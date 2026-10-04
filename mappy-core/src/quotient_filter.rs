//! Quotient filter implementation with multiset support
//!
//! Implements the quotient filter as the perfect-hashing filter foundation
//! for maplets, supporting multisets of fingerprints and efficient slot management.

// use bitvec::prelude::*;
use crate::{
    MapletError, MapletResult,
    hash::{FingerprintHasher, HashFunction, PerfectHash},
};
use std::collections::HashMap;

/// Largest quotient width the `u32` owner tag can represent.
///
/// `quotient` is masked to `quotient_bits` before it is stored, so this caps the
/// filter at `2^32` slots rather than `usize::MAX`.
const MAX_QUOTIENT_BITS: u32 = 32;

/// `run_starts` entry meaning "this quotient has no run".
const NO_RUN: u32 = u32::MAX;

/// How many times `insert` may double the table before giving up on placement.
const MAX_GROW_RETRIES: u32 = 8;

/// Owner tag stored in a slot for members of `quotient`.
fn tag_of(quotient: u64) -> u32 {
    debug_assert!(
        quotient <= u64::from(u32::MAX),
        "quotient exceeds owner tag width"
    );
    #[allow(clippy::cast_possible_truncation)] // width enforced by MAX_QUOTIENT_BITS
    {
        quotient as u32
    }
}

/// Metadata bits for quotient filter slots
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotMetadata {
    /// Remainder bits (the actual fingerprint data)
    pub remainder: u64,
    /// Quotient that owns this slot; meaningful only while `occupied` is set.
    ///
    /// This is what identifies a run. The canonical slot cannot: `PerfectHash::slot_index`
    /// is `hash % capacity` rather than a permutation, so distinct quotients collide onto
    /// one slot, and a cluster shift can move a run's start past its own canonical slot.
    /// Neither case is recoverable from `runend` and `shifted` alone.
    pub owner: u32,
    /// Run-end bit: marks the end of a run
    pub runend: bool,
    /// Occupied bit: slot is occupied by some fingerprint
    pub occupied: bool,
    /// Shifted bit: slot has been shifted due to linear probing
    pub shifted: bool,
}

impl Default for SlotMetadata {
    fn default() -> Self {
        Self {
            remainder: 0,
            owner: NO_RUN,
            runend: false,
            occupied: false,
            shifted: false,
        }
    }
}

/// Quotient filter implementation with multiset support
#[derive(Debug, Clone)]
pub struct QuotientFilter {
    /// Number of slots in the filter
    capacity: usize,
    /// Number of quotient bits (determines number of runs)
    quotient_bits: u32,
    /// Number of remainder bits
    remainder_bits: u32,
    /// Quotient mask for extracting quotient
    quotient_mask: u64,
    /// Remainder mask for extracting remainder
    remainder_mask: u64,
    /// Array of slot metadata
    slots: Vec<SlotMetadata>,
    /// Start slot of each quotient's run, indexed by quotient; `NO_RUN` when absent.
    ///
    /// Sized `1 << quotient_bits` rather than `capacity`, because the index is the
    /// quotient and quotients outnumber slots.
    run_starts: Vec<u32>,
    /// Fingerprint hasher
    hasher: FingerprintHasher,
    /// Perfect hash for slot mapping
    perfect_hash: PerfectHash,
    /// Current number of items stored
    len: usize,
    /// Multiset support: track multiple instances per fingerprint
    multiset_counts: HashMap<u64, usize>,
}

impl QuotientFilter {
    /// Create a new quotient filter
    ///
    /// # Errors
    /// Returns an error if capacity is zero, `fingerprint_bits` is invalid (0 or > 64),
    /// or the capacity needs more quotient bits than an owner tag can carry.
    pub fn new(
        capacity: usize,
        fingerprint_bits: u32,
        hash_fn: HashFunction,
    ) -> MapletResult<Self> {
        if capacity == 0 {
            return Err(MapletError::InvalidCapacity(capacity));
        }

        if fingerprint_bits == 0 || fingerprint_bits > 64 {
            return Err(MapletError::InvalidErrorRate(f64::from(fingerprint_bits)));
        }

        let (quotient_bits, remainder_bits, quotient_mask, remainder_mask) =
            Self::slot_layout(capacity, fingerprint_bits);

        if quotient_bits > MAX_QUOTIENT_BITS {
            return Err(MapletError::InvalidCapacity(capacity));
        }

        Ok(Self {
            capacity,
            quotient_bits,
            remainder_bits,
            quotient_mask,
            remainder_mask,
            slots: vec![SlotMetadata::default(); capacity],
            run_starts: vec![NO_RUN; 1 << quotient_bits],
            hasher: FingerprintHasher::new(hash_fn, fingerprint_bits),
            perfect_hash: PerfectHash::new(capacity, hash_fn),
            len: 0,
            multiset_counts: HashMap::new(),
        })
    }

    /// Split a capacity and fingerprint width into quotient/remainder widths and masks.
    ///
    /// Shared by construction and growth so a grown filter is laid out identically to
    /// one built at that capacity directly.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )] // Acceptable for bit calculation
    fn slot_layout(capacity: usize, fingerprint_bits: u32) -> (u32, u32, u64, u64) {
        let quotient_bits = (capacity as f64).log2().ceil() as u32;
        let remainder_bits = fingerprint_bits.saturating_sub(quotient_bits).max(1);

        let quotient_mask = if quotient_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << quotient_bits) - 1
        };

        let remainder_mask = if remainder_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << remainder_bits) - 1
        };

        (quotient_bits, remainder_bits, quotient_mask, remainder_mask)
    }

    /// Insert a fingerprint into the filter
    ///
    /// # Errors
    /// Returns an error if capacity is exceeded or insertion fails.
    pub fn insert(&mut self, fingerprint: u64) -> MapletResult<()> {
        if self.len >= self.capacity {
            return Err(MapletError::CapacityExceeded);
        }

        // `len < capacity` holds here, so a `CapacityExceeded` from the shift path can
        // only mean this run has no free slot after it, never that the table is full.
        // Rehashing separates the clusters, but one doubling is not itself a guarantee:
        // the run for this particular canonical slot can still land against the end of a
        // fresh array. Doubling makes that exponentially less likely, so a few retries
        // settle it; exceeding the cap means the placement path is wrong, not the load.
        let mut retries = 0u32;
        loop {
            match self.place(fingerprint) {
                Ok(()) => break,
                Err(MapletError::CapacityExceeded) => {
                    retries += 1;
                    if retries > MAX_GROW_RETRIES {
                        return Err(MapletError::CapacityExceeded);
                    }
                    self.grow()?;
                }
                Err(other) => return Err(other),
            }
        }

        // Update multiset count
        *self.multiset_counts.entry(fingerprint).or_insert(0) += 1;
        self.len += 1;
        Ok(())
    }

    /// Place a fingerprint's remainder into its run, shifting neighbours as needed.
    ///
    /// Leaves `len` and `multiset_counts` alone; callers own the bookkeeping.
    fn place(&mut self, fingerprint: u64) -> MapletResult<()> {
        let quotient = self.extract_quotient(fingerprint);
        let remainder = self.extract_remainder(fingerprint);

        match self.run_of(quotient) {
            Some((start, end)) => self.extend_run(start, end, quotient, remainder),
            None => self.start_run(quotient, remainder),
        }
    }

    /// Create a run for `quotient` in the first free slot at or after its canonical slot.
    ///
    /// # Errors
    /// Returns an error when no slot at all is free, which the caller resolves by growing.
    fn start_run(&mut self, quotient: u64, remainder: u64) -> MapletResult<()> {
        let target = self.find_target_slot(quotient);

        // Prefer the first free slot at or after the canonical slot so runs stay in
        // quotient order wherever the array allows it. A packed tail is not a full table —
        // `insert` guarantees `len < capacity` — so fall back to any free slot rather than
        // report `CapacityExceeded` for a table that demonstrably has room.
        let Some(slot) = (target..self.capacity)
            .chain(0..target)
            .find(|&s| !self.slots[s].occupied)
        else {
            return Err(MapletError::CapacityExceeded);
        };

        let owner = tag_of(quotient);
        self.slots[slot] = SlotMetadata {
            remainder,
            owner,
            runend: true,
            occupied: true,
            shifted: false,
        };
        #[allow(clippy::cast_possible_truncation)] // slot is bounded by capacity <= 2^32
        {
            self.run_starts[Self::run_index(quotient)] = slot as u32;
        }
        self.refresh_flags(slot);
        Ok(())
    }

    /// Append `remainder` to the run occupying `start..=end`, pushing later runs right.
    ///
    /// # Errors
    /// Returns an error when the run reaches the final slot and no free slot follows.
    fn extend_run(
        &mut self,
        start: usize,
        end: usize,
        quotient: u64,
        remainder: u64,
    ) -> MapletResult<()> {
        let mut free = end + 1;
        while free < self.capacity && self.slots[free].occupied {
            free += 1;
        }

        if free >= self.capacity {
            return Err(MapletError::CapacityExceeded);
        }

        // Shift the occupied tail one slot right. Only this run's own elements keep their slots,
        // so every run that moves has to record its new start or it becomes unreachable.
        // Slot `i` starts a run when its predecessor belongs to another one; every run in
        // the tail therefore gets its entry bumped exactly once, and this run's does not
        // move at all. `free` is excluded: it absorbs the last moved element and is itself
        // overwritten, so reading its owner would be reading an empty slot.
        #[allow(clippy::cast_possible_truncation)] // i + 1 is bounded by capacity <= 2^32
        for i in (end + 1..free).rev() {
            if self.slots[i - 1].owner != self.slots[i].owner {
                // u32 -> usize is widening; the tag indexes `run_starts` directly.
                let moved = self.slots[i].owner as usize;
                self.run_starts[moved] = (i + 1) as u32;
            }
            self.slots[i + 1] = self.slots[i];
        }

        self.slots[end + 1] = SlotMetadata {
            remainder,
            owner: tag_of(quotient),
            runend: false,
            occupied: true,
            shifted: true,
        };
        self.refresh_flags(start);
        Ok(())
    }

    /// Index into `run_starts` for a quotient, which `extract_quotient` has already masked.
    #[allow(clippy::cast_possible_truncation)] // quotient is bounded by MAX_QUOTIENT_BITS
    const fn run_index(quotient: u64) -> usize {
        quotient as usize
    }

    /// The run holding `quotient`, as `(start, end)` inclusive, or `None` if it has none.
    ///
    /// The start comes from `run_starts` and the end from scanning consecutive slots
    /// carrying the same owner tag. Both are exact: neither depends on the `runend` or
    /// `shifted` bits, which cannot represent a run that starts past its canonical slot.
    fn run_of(&self, quotient: u64) -> Option<(usize, usize)> {
        let start = self.run_starts[Self::run_index(quotient)];
        if start == NO_RUN {
            return None;
        }

        let owner = tag_of(quotient);
        #[allow(clippy::cast_possible_truncation)] // NO_RUN is filtered above
        let start = start as usize;
        let mut end = start;
        while end + 1 < self.capacity
            && self.slots[end + 1].occupied
            && self.slots[end + 1].owner == owner
        {
            end += 1;
        }

        Some((start, end))
    }

    /// Recompute the `runend` and `shifted` metadata for the cluster containing `from`.
    ///
    /// Both are diagnostics: run membership is read from the owner tag and `run_starts`,
    /// so a stale bit here can no longer hide a stored fingerprint.
    fn refresh_flags(&mut self, from: usize) {
        let mut i = from;

        while i < self.capacity {
            if !self.slots[i].occupied {
                // `from` itself is always occupied; a gap after it ends the cluster.
                if i > from {
                    return;
                }
                i += 1;
                continue;
            }

            let owner = self.slots[i].owner;
            let mut j = i;
            while j + 1 < self.capacity
                && self.slots[j + 1].occupied
                && self.slots[j + 1].owner == owner
            {
                j += 1;
            }

            for k in i..=j {
                self.slots[k].runend = k == j;
                self.slots[k].shifted = k != i;
            }

            if j + 1 < self.capacity && !self.slots[j + 1].occupied {
                return;
            }
            i = j + 1;
        }
    }

    /// Double the capacity, rehashing every stored fingerprint.
    ///
    /// # Errors
    /// Returns an error if the capacity cannot be doubled or the replay overflows.
    fn grow(&mut self) -> MapletResult<()> {
        let new_capacity = self
            .capacity
            .checked_mul(2)
            .ok_or(MapletError::CapacityExceeded)?;
        self.grow_to(new_capacity)
    }

    /// Rebuild the filter at `new_capacity`, rehashing every stored fingerprint.
    ///
    /// Slot indices are `hash % capacity`, so growing moves every fingerprint to a new
    /// slot. `multiset_counts` is the only record of what is present and carries the full
    /// fingerprints, so it is replayed slot for slot rather than merely trusted.
    ///
    /// # Errors
    /// Returns an error if `new_capacity` is zero or no capacity can hold the contents.
    fn grow_to(&mut self, new_capacity: usize) -> MapletResult<()> {
        if new_capacity == 0 {
            return Err(MapletError::InvalidCapacity(new_capacity));
        }

        // One entry per stored instance: a repeated fingerprint occupies a slot per
        // instance, so the replay must place it that many times.
        let mut pending: Vec<u64> = Vec::with_capacity(self.len);
        for (&fingerprint, &count) in &self.multiset_counts {
            pending.extend(std::iter::repeat_n(fingerprint, count));
        }

        let mut capacity = new_capacity;
        loop {
            self.reset_layout(capacity)?;
            if pending.iter().all(|&f| self.place(f).is_ok()) {
                self.len = pending.len();
                return Ok(());
            }
            // The replay hit the same edge condition; doubling eventually separates the
            // runs, so retry from scratch rather than leaving the filter half rebuilt.
            capacity = capacity
                .checked_mul(2)
                .ok_or(MapletError::CapacityExceeded)?;
        }
    }

    /// Reallocate slots and rederive every capacity-dependent field at `capacity`.
    ///
    /// # Errors
    /// Returns an error if `capacity` needs more quotient bits than an owner tag holds.
    fn reset_layout(&mut self, capacity: usize) -> MapletResult<()> {
        let (quotient_bits, remainder_bits, quotient_mask, remainder_mask) =
            Self::slot_layout(capacity, self.hasher.fingerprint_bits());

        if quotient_bits > MAX_QUOTIENT_BITS {
            return Err(MapletError::InvalidCapacity(capacity));
        }

        self.capacity = capacity;
        self.quotient_bits = quotient_bits;
        self.remainder_bits = remainder_bits;
        self.quotient_mask = quotient_mask;
        self.remainder_mask = remainder_mask;
        self.slots = vec![SlotMetadata::default(); capacity];
        self.run_starts = vec![NO_RUN; 1 << quotient_bits];
        self.perfect_hash = PerfectHash::new(capacity, self.hasher.hash_fn());
        self.len = 0;
        Ok(())
    }

    /// Query if a fingerprint might be in the filter
    #[must_use]
    pub fn query(&self, fingerprint: u64) -> bool {
        let quotient = self.extract_quotient(fingerprint);
        let remainder = self.extract_remainder(fingerprint);

        match self.run_of(quotient) {
            Some((start, end)) => (start..=end).any(|slot| self.slots[slot].remainder == remainder),
            None => false,
        }
    }

    /// Delete one instance of a fingerprint from the filter
    ///
    /// # Errors
    /// Returns an error if deletion fails.
    pub fn delete(&mut self, fingerprint: u64) -> MapletResult<bool> {
        // The slots hold truncated remainders, so a different full fingerprint can share
        // this one's quotient and remainder. `multiset_counts` holds full fingerprints
        // and is the only authority on membership; without this guard a delete would
        // remove a slot belonging to a fingerprint that was never inserted.
        if !self.multiset_counts.contains_key(&fingerprint) {
            return Ok(false);
        }

        let quotient = self.extract_quotient(fingerprint);
        let remainder = self.extract_remainder(fingerprint);

        // The multiset counts one slot per stored instance, so a present fingerprint whose
        // slot is missing means the two disagree. Re-placing repairs the invariant rather
        // than decrementing bookkeeping that was never matched by a slot.
        let Some((start, end)) = self.run_of(quotient) else {
            self.place(fingerprint)?;
            return Ok(false);
        };

        let Some(offset) = (start..=end).find(|&s| self.slots[s].remainder == remainder) else {
            self.place(fingerprint)?;
            return Ok(false);
        };

        // Close the gap inside this run only. Shifting across the terminator would move a
        // later run's elements left of their own slots; the run simply ends one earlier.
        for i in offset..end {
            self.slots[i] = self.slots[i + 1];
        }
        self.slots[end] = SlotMetadata::default();

        if start == end {
            self.run_starts[Self::run_index(quotient)] = NO_RUN;
        }
        self.refresh_flags(start);

        if let Some(count) = self.multiset_counts.get_mut(&fingerprint) {
            *count -= 1;
            if *count == 0 {
                self.multiset_counts.remove(&fingerprint);
            }
        }

        self.len -= 1;
        Ok(true)
    }

    /// Get the number of instances of a fingerprint
    #[must_use]
    pub fn count(&self, fingerprint: u64) -> usize {
        self.multiset_counts.get(&fingerprint).copied().unwrap_or(0)
    }

    /// Get the current number of items stored
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if the filter is empty
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity of the filter
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the load factor
    #[must_use]
    pub fn load_factor(&self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        {
            self.len as f64 / self.capacity as f64
        }
    }

    /// Get the number of quotient bits
    #[must_use]
    pub const fn quotient_bits(&self) -> u32 {
        self.quotient_bits
    }

    /// Clear all data from the filter
    pub fn clear(&mut self) {
        self.slots = vec![SlotMetadata::default(); self.capacity];
        self.run_starts = vec![NO_RUN; 1 << self.quotient_bits];
        self.len = 0;
        self.multiset_counts.clear();
    }

    /// Extract quotient from fingerprint
    const fn extract_quotient(&self, fingerprint: u64) -> u64 {
        fingerprint & self.quotient_mask
    }

    /// Extract remainder from fingerprint
    const fn extract_remainder(&self, fingerprint: u64) -> u64 {
        (fingerprint >> self.quotient_bits) & self.remainder_mask
    }

    /// Find the target slot for a quotient
    fn find_target_slot(&self, quotient: u64) -> usize {
        // Use perfect hash to find the target slot
        self.perfect_hash.slot_index(quotient)
    }

    /// Get the slot index for a fingerprint (for maplet coordination)
    #[must_use]
    pub fn get_slot_for_fingerprint(&self, fingerprint: u64) -> usize {
        let quotient = self.extract_quotient(fingerprint);
        self.find_target_slot(quotient)
    }

    /// Get the actual slot where a fingerprint is stored (considering runs and shifting)
    #[must_use]
    pub fn get_actual_slot_for_fingerprint(&self, fingerprint: u64) -> Option<usize> {
        let quotient = self.extract_quotient(fingerprint);
        let remainder = self.extract_remainder(fingerprint);

        let (start, end) = self.run_of(quotient)?;
        (start..=end).find(|&slot| self.slots[slot].remainder == remainder)
    }

    /// Get statistics about the filter
    #[must_use]
    pub fn stats(&self) -> QuotientFilterStats {
        let mut runs = 0;
        let mut shifted_slots = 0;

        for slot in &self.slots {
            if slot.runend {
                runs += 1;
            }
            if slot.shifted {
                shifted_slots += 1;
            }
        }

        QuotientFilterStats {
            capacity: self.capacity,
            len: self.len,
            load_factor: self.load_factor(),
            runs,
            shifted_slots,
            quotient_bits: self.quotient_bits,
            remainder_bits: self.remainder_bits,
        }
    }
}

/// Statistics for a quotient filter
#[derive(Debug, Clone)]
pub struct QuotientFilterStats {
    pub capacity: usize,
    pub len: usize,
    pub load_factor: f64,
    pub runs: usize,
    pub shifted_slots: usize,
    pub quotient_bits: u32,
    pub remainder_bits: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quotient_filter_basic() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        // Insert some fingerprints
        assert!(filter.insert(0x1234).is_ok());
        assert!(filter.insert(0x5678).is_ok());
        assert!(filter.insert(0x9ABC).is_ok());

        assert_eq!(filter.len(), 3);

        // Query existing fingerprints
        assert!(filter.query(0x1234));
        assert!(filter.query(0x5678));
        assert!(filter.query(0x9ABC));

        // Query non-existing fingerprint
        assert!(!filter.query(0xDEAD));
    }

    #[test]
    fn test_quotient_filter_multiset() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        // Insert the same fingerprint multiple times
        assert!(filter.insert(0x1234).is_ok());
        assert!(filter.insert(0x1234).is_ok());
        assert!(filter.insert(0x1234).is_ok());

        assert_eq!(filter.len(), 3);
        assert_eq!(filter.count(0x1234), 3);

        // Delete one instance
        assert!(filter.delete(0x1234).unwrap());
        assert_eq!(filter.len(), 2);
        assert_eq!(filter.count(0x1234), 2);

        // Query should still return true
        assert!(filter.query(0x1234));
    }

    #[test]
    fn test_quotient_filter_capacity() {
        let mut filter = QuotientFilter::new(3, 8, HashFunction::AHash).unwrap();

        // Fill to capacity
        assert!(filter.insert(0x1111).is_ok());
        assert!(filter.insert(0x2222).is_ok());
        assert!(filter.insert(0x3333).is_ok());

        // Should fail to insert more
        assert!(filter.insert(0x4444).is_err());
    }

    #[test]
    fn test_quotient_filter_stats() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        filter.insert(0x1234).unwrap();
        filter.insert(0x5678).unwrap();

        let stats = filter.stats();
        assert_eq!(stats.capacity, 100);
        assert_eq!(stats.len, 2);
        assert!(stats.load_factor > 0.0);
        assert!(stats.runs > 0);
    }

    #[test]
    fn test_actual_slot_finding() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        // Insert some fingerprints
        let fingerprint1 = 0x1234;
        let fingerprint2 = 0x5678;

        assert!(filter.insert(fingerprint1).is_ok());
        assert!(filter.insert(fingerprint2).is_ok());

        // Find actual slots for the fingerprints
        let slot1 = filter.get_actual_slot_for_fingerprint(fingerprint1);
        let slot2 = filter.get_actual_slot_for_fingerprint(fingerprint2);

        assert!(slot1.is_some(), "Should find actual slot for fingerprint1");
        assert!(slot2.is_some(), "Should find actual slot for fingerprint2");

        // Slots should be different (unless there's a collision)
        if slot1 != slot2 {
            assert_ne!(
                slot1, slot2,
                "Different fingerprints should have different slots"
            );
        }

        // Try to find slot for non-existing fingerprint
        let non_existing = 0xDEAD;
        let non_existing_slot = filter.get_actual_slot_for_fingerprint(non_existing);
        assert!(
            non_existing_slot.is_none(),
            "Should not find slot for non-existing fingerprint"
        );
    }

    #[test]
    fn test_quotient_filter_delete_nonexistent() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        let deleted = filter.delete(0xDEAD).unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_quotient_filter_is_empty() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        assert!(filter.is_empty());

        filter.insert(0x1234).unwrap();

        assert!(!filter.is_empty());
    }

    #[test]
    fn test_quotient_filter_load_factor() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        assert_eq!(filter.load_factor(), 0.0);

        filter.insert(0x1234).unwrap();
        filter.insert(0x5678).unwrap();

        assert!(filter.load_factor() > 0.0);
    }

    #[test]
    fn test_quotient_filter_quotient_bits() {
        let filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        assert!(filter.quotient_bits() > 0);
    }

    #[test]
    fn test_quotient_filter_get_slot_for_fingerprint() {
        let filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        let slot = filter.get_slot_for_fingerprint(0x1234);
        assert!(slot < 100);
    }

    #[test]
    fn test_quotient_filter_capacity_getter() {
        let filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        assert_eq!(filter.capacity(), 100);
    }

    #[test]
    fn test_slot_metadata_default() {
        let metadata = SlotMetadata::default();

        assert_eq!(metadata.remainder, 0);
        assert_eq!(metadata.owner, NO_RUN);
        assert!(!metadata.runend);
        assert!(!metadata.occupied);
        assert!(!metadata.shifted);
    }

    #[test]
    fn test_slot_metadata_clone() {
        let metadata = SlotMetadata {
            remainder: 123,
            owner: 7,
            runend: true,
            occupied: true,
            shifted: false,
        };

        let cloned = metadata.clone();
        assert_eq!(cloned.remainder, 123);
        assert!(cloned.runend);
        assert!(cloned.occupied);
        assert!(!cloned.shifted);
    }

    #[test]
    fn test_slot_metadata_eq() {
        let metadata1 = SlotMetadata {
            remainder: 123,
            owner: 7,
            runend: true,
            occupied: true,
            shifted: false,
        };

        let metadata2 = SlotMetadata {
            remainder: 123,
            owner: 7,
            runend: true,
            occupied: true,
            shifted: false,
        };

        assert_eq!(metadata1, metadata2);
    }

    #[test]
    fn test_quotient_filter_twox_hash() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::TwoX).unwrap();

        assert!(filter.insert(0x1234).is_ok());
        assert!(filter.query(0x1234));
    }

    #[test]
    fn test_quotient_filter_fnv_hash() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::Fnv).unwrap();

        assert!(filter.insert(0x1234).is_ok());
        assert!(filter.query(0x1234));
    }

    #[test]
    fn test_quotient_filter_invalid_capacity() {
        let filter = QuotientFilter::new(0, 8, HashFunction::AHash);
        assert!(filter.is_err());
    }

    #[test]
    fn test_quotient_filter_invalid_fingerprint_bits() {
        let filter = QuotientFilter::new(100, 0, HashFunction::AHash);
        assert!(filter.is_err());

        let filter = QuotientFilter::new(100, 65, HashFunction::AHash);
        assert!(filter.is_err());
    }

    #[test]
    fn test_quotient_filter_stats_clone() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();
        filter.insert(0x1234).unwrap();

        let stats = filter.stats();
        let cloned = stats.clone();

        assert_eq!(cloned.capacity, stats.capacity);
        assert_eq!(cloned.len, stats.len);
    }

    #[test]
    fn test_quotient_filter_count_nonexistent() {
        let filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        assert_eq!(filter.count(0xDEAD), 0);
    }

    #[test]
    fn test_quotient_filter_delete_all_instances() {
        let mut filter = QuotientFilter::new(100, 8, HashFunction::AHash).unwrap();

        filter.insert(0x1234).unwrap();
        filter.insert(0x1234).unwrap();

        assert_eq!(filter.count(0x1234), 2);

        filter.delete(0x1234).unwrap();
        filter.delete(0x1234).unwrap();

        assert_eq!(filter.count(0x1234), 0);
    }

    /// Deterministic pseudo-random source, so any failure reproduces exactly without
    /// pulling in a dependency.
    fn splitmix(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// The owner tag was chosen over a `u64` because it fits the padding `SlotMetadata`
    /// already had. That is the whole reason the layout change is free, so it is pinned here:
    /// a future field that breaks the size turns the design rationale into a false claim.
    #[test]
    fn test_owner_tag_costs_no_extra_slot_bytes() {
        assert_eq!(
            std::mem::size_of::<SlotMetadata>(),
            16,
            "owner must occupy existing padding, not grow the slot"
        );
        // The run-start table is the one real cost: one u32 per quotient.
        let filter = QuotientFilter::new(100, 10, HashFunction::AHash).unwrap();
        assert_eq!(filter.run_starts.len(), 1 << filter.quotient_bits());
        assert!(
            filter.run_starts.len() <= 2 * filter.capacity(),
            "run table should stay within a small constant multiple of capacity"
        );
    }

    /// The premise a quotient filter needs and this implementation never had: an injective
    /// quotient-to-canonical-slot map. `PerfectHash::slot_index` is `hash % capacity`, and
    /// there are `2^quotient_bits` quotients over `capacity` slots, so two distinct
    /// quotients always collide onto one canonical slot. Any scheme that reconstructs a
    /// run by walking back from the canonical slot is therefore unsound, independently of
    /// how it handles a run displaced past its own slot. Owner tags are the replacement.
    #[test]
    fn test_canonical_slot_map_is_not_injective() {
        let filter = QuotientFilter::new(100, 10, HashFunction::AHash).unwrap();
        let table_len = 1usize << filter.quotient_bits();
        assert!(
            table_len > filter.capacity(),
            "quotients must outnumber slots"
        );

        let mut seen = HashMap::new();
        let mut collided = 0usize;
        for quotient in 0..table_len as u64 {
            let slot = filter.find_target_slot(quotient);
            if let Some(&other) = seen.get(&slot) {
                assert_ne!(other, quotient);
                collided += 1;
            } else {
                seen.insert(slot, quotient);
            }
        }

        assert!(
            collided > 0,
            "expected canonical-slot collisions over {table_len} quotients in {} slots",
            filter.capacity()
        );
    }

    /// Two quotients sharing a canonical slot must keep separate runs, and each must stay
    /// reachable after the other grows. The old layout conflated them into one run.
    #[test]
    fn test_runs_split_across_a_shared_canonical_slot() {
        let mut filter = QuotientFilter::new(100, 10, HashFunction::AHash).unwrap();
        let table_len = 1usize << filter.quotient_bits();

        let mut seen: HashMap<usize, u64> = HashMap::new();
        let mut pair = None;
        for quotient in 0..table_len as u64 {
            let slot = filter.find_target_slot(quotient);
            match seen.get(&slot) {
                Some(&other) => {
                    pair = Some((other, quotient));
                    break;
                }
                None => {
                    seen.insert(slot, quotient);
                }
            }
        }

        let (a, b) = pair.expect("canonical-slot collision is guaranteed");
        assert_eq!(
            filter.find_target_slot(a),
            filter.find_target_slot(b),
            "the pair must share a canonical slot for this test to mean anything"
        );

        // Alternate the two colliding quotients so each run grows after the other exists.
        let mut stored = Vec::new();
        for remainder in 1..5u64 {
            for &quotient in &[a, b] {
                let fingerprint = quotient | (remainder << filter.quotient_bits());
                filter.insert(fingerprint).unwrap();
                stored.push(fingerprint);
            }
        }

        assert_eq!(
            stored.len(),
            8,
            "expected four remainders for each quotient"
        );
        for &fingerprint in &stored {
            assert!(
                filter.query(fingerprint),
                "lost {fingerprint:#x} (quotients {a} and {b} share a canonical slot)"
            );
        }
    }

    /// The boundary contract: while the filter is provably not full, an insert must never
    /// report `CapacityExceeded`, and every accepted insert must be counted. This is
    /// precisely what growing on boundary overflow is responsible for.
    #[test]
    fn test_insert_below_capacity_never_errors() {
        let mask = (1u64 << 10) - 1;

        for seed in 0..256u64 {
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut filter = QuotientFilter::new(100, 10, HashFunction::AHash).unwrap();
            let mut attempts = 0usize;

            for _ in 0..40 {
                if filter.len() == filter.capacity() {
                    break;
                }
                attempts += 1;
                let fp = splitmix(&mut state) & mask;
                filter
                    .insert(fp)
                    .unwrap_or_else(|e| panic!("seed {seed}: insert below capacity failed: {e:?}"));
            }

            assert_eq!(
                filter.len(),
                attempts,
                "seed {seed}: accepted inserts uncounted"
            );
        }
    }

    /// An insert must never lose an earlier fingerprint.
    #[test]
    fn test_insert_below_capacity_preserves_fingerprints() {
        let mask = (1u64 << 10) - 1;

        for seed in 0..256u64 {
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut filter = QuotientFilter::new(100, 10, HashFunction::AHash).unwrap();
            let mut inserted = Vec::new();

            for _ in 0..40 {
                if filter.len() == filter.capacity() {
                    break;
                }
                let fp = splitmix(&mut state) & mask;
                filter
                    .insert(fp)
                    .unwrap_or_else(|e| panic!("seed {seed}: insert below capacity failed: {e:?}"));
                inserted.push(fp);
            }

            assert_eq!(filter.len(), inserted.len(), "seed {seed}: len drift");
            for &fp in &inserted {
                assert!(filter.query(fp), "seed {seed}: lost {fp:#x}");
            }
        }
    }

    /// Growth must rehash, not merely reallocate: every stored instance stays queryable
    /// and `len` stays exact. Swept over enough inputs that the final-slot boundary is
    /// actually reached, so the growth path is exercised rather than never taken.
    #[test]
    fn test_growth_preserves_every_stored_instance() {
        let mask = (1u64 << 10) - 1;
        let mut grew = 0;

        for seed in 0..256u64 {
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut filter = QuotientFilter::new(64, 10, HashFunction::AHash).unwrap();
            let mut inserted = Vec::new();

            for _ in 0..60 {
                if filter.len() == filter.capacity() {
                    break;
                }
                let fp = splitmix(&mut state) & mask;
                filter.insert(fp).unwrap();
                inserted.push(fp);
            }

            if filter.capacity() > 64 {
                grew += 1;
            }
            assert_eq!(filter.len(), inserted.len(), "seed {seed}: len drift");
            for &fp in &inserted {
                assert!(filter.query(fp), "seed {seed}: growth lost {fp:#x}");
            }
        }

        assert!(
            grew > 0,
            "no seed reached the final-slot boundary; the growth path is untested"
        );
    }

    /// Model-based property test: for any interleaving of inserts and deletes the filter
    /// must have no false negatives and `len` must equal the stored instance count.
    #[test]
    fn test_filter_matches_reference_under_mixed_operations() {
        let mask = (1u64 << 10) - 1;

        for seed in 0..64u64 {
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut filter = QuotientFilter::new(100, 10, HashFunction::AHash).unwrap();
            let mut model: HashMap<u64, usize> = HashMap::new();
            let mut total = 0usize;

            for step in 0..300 {
                let fp = splitmix(&mut state) & mask;

                if splitmix(&mut state) % 2 == 0 {
                    let present = model.contains_key(&fp);
                    let deleted = filter.delete(fp).unwrap();
                    assert_eq!(
                        deleted, present,
                        "seed {seed} step {step}: delete disagreed"
                    );

                    if present {
                        let count = model.get_mut(&fp).unwrap();
                        *count -= 1;
                        if *count == 0 {
                            model.remove(&fp);
                        }
                        total -= 1;
                    }
                } else {
                    if filter.len() == filter.capacity() {
                        continue;
                    }
                    filter.insert(fp).unwrap();
                    *model.entry(fp).or_insert(0) += 1;
                    total += 1;
                }

                assert_eq!(filter.len(), total, "seed {seed} step {step}: len drift");
                for (&stored, &count) in &model {
                    assert!(
                        filter.query(stored),
                        "seed {seed} step {step}: false negative for {stored:#x}"
                    );
                    assert_eq!(
                        filter.count(stored),
                        count,
                        "seed {seed} step {step}: count drift for {stored:#x}"
                    );
                }
            }
        }
    }

    /// Regression: removal used to clear the final slot of the array unconditionally,
    /// destroying an unrelated fingerprint. Every surviving fingerprint must stay
    /// queryable after any deletion.
    #[test]
    fn test_delete_preserves_unrelated_fingerprints() {
        let mask = (1u64 << 10) - 1;

        for seed in 0..128u64 {
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut filter = QuotientFilter::new(100, 10, HashFunction::AHash).unwrap();
            let mut live = Vec::new();

            for _ in 0..30 {
                if filter.len() == filter.capacity() {
                    break;
                }
                let fp = splitmix(&mut state) & mask;
                filter.insert(fp).unwrap();
                live.push(fp);
            }

            let victim = live[live.len() / 2];
            assert!(
                filter.delete(victim).unwrap(),
                "seed {seed}: victim not deleted"
            );

            live.retain(|&fp| fp != victim);
            for &fp in &live {
                assert!(filter.query(fp), "seed {seed}: delete destroyed {fp:#x}");
            }
        }
    }

    /// A full drain and refill must be indistinguishable from a fresh filter: no stale
    /// `run_starts` entry may survive to make an absent fingerprint look present.
    #[test]
    fn test_clear_resets_every_run() {
        let mask = (1u64 << 10) - 1;
        let mut filter = QuotientFilter::new(100, 10, HashFunction::AHash).unwrap();

        let mut state = 7u64;
        let mut populated = Vec::new();
        for _ in 0..40 {
            let fp = splitmix(&mut state) & mask;
            filter.insert(fp).unwrap();
            populated.push(fp);
        }
        assert_eq!(filter.len(), 40);

        filter.clear();
        assert!(filter.is_empty());

        for fp in &populated {
            assert!(!filter.query(*fp), "cleared filter still holds {fp:#x}");
            assert!(!filter.delete(*fp).unwrap(), "deleted from an empty filter");
        }

        for fp in populated.iter().take(20) {
            filter.insert(*fp).unwrap();
        }
        assert_eq!(filter.len(), 20);
        for fp in populated.iter().take(20) {
            assert!(filter.query(*fp), "refill lost {fp:#x}");
        }
    }
}
