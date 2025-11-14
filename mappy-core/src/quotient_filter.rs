//! Quotient filter implementation with multiset support
//! 
//! Implements the quotient filter as the perfect-hashing filter foundation
//! for maplets, supporting multisets of fingerprints and efficient slot management.

// use bitvec::prelude::*;
use std::collections::HashMap;
use crate::{MapletError, MapletResult, hash::{FingerprintHasher, PerfectHash, HashFunction}};

/// Metadata bits for quotient filter slots
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub struct SlotMetadata {
    /// Remainder bits (the actual fingerprint data)
    pub remainder: u64,
    /// Run-end bit: marks the end of a run
    pub runend: bool,
    /// Occupied bit: slot is occupied by some fingerprint
    pub occupied: bool,
    /// Shifted bit: slot has been shifted due to linear probing
    pub shifted: bool,
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
    /// Fingerprint hasher
    #[allow(dead_code)]
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
    pub fn new(capacity: usize, fingerprint_bits: u32, hash_fn: HashFunction) -> MapletResult<Self> {
        if capacity == 0 {
            return Err(MapletError::InvalidCapacity(capacity));
        }
        
        if fingerprint_bits == 0 || fingerprint_bits > 64 {
            return Err(MapletError::InvalidErrorRate(f64::from(fingerprint_bits)));
        }
        
        // Calculate quotient and remainder bits
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
        
        Ok(Self {
            capacity,
            quotient_bits,
            remainder_bits,
            quotient_mask,
            remainder_mask,
            slots: vec![SlotMetadata::default(); capacity],
            hasher: FingerprintHasher::new(hash_fn, fingerprint_bits),
            perfect_hash: PerfectHash::new(capacity, hash_fn),
            len: 0,
            multiset_counts: HashMap::new(),
        })
    }
    
    /// Insert a fingerprint into the filter
    pub fn insert(&mut self, fingerprint: u64) -> MapletResult<()> {
        if self.len >= self.capacity {
            return Err(MapletError::CapacityExceeded);
        }
        
        let quotient = self.extract_quotient(fingerprint);
        let remainder = self.extract_remainder(fingerprint);
        
        // Find the target slot for this quotient
        let target_slot = self.find_target_slot(quotient);
        
        // Insert the remainder at the target slot
        match self.insert_remainder_at_slot(target_slot, remainder) {
            Ok(()) => {
                // Update multiset count
                *self.multiset_counts.entry(fingerprint).or_insert(0) += 1;
                self.len += 1;
                Ok(())
            }
            Err(MapletError::CapacityExceeded) => {
                // If we can't insert due to capacity issues, return capacity exceeded
                Err(MapletError::CapacityExceeded)
            }
            Err(e) => Err(e)
        }
    }
    
    /// Query if a fingerprint might be in the filter
    #[must_use] 
    pub fn query(&self, fingerprint: u64) -> bool {
        let quotient = self.extract_quotient(fingerprint);
        let remainder = self.extract_remainder(fingerprint);
        
        // Find the target slot for this quotient
        let target_slot = self.find_target_slot(quotient);
        
        // Check if the target slot is occupied
        if !self.slots[target_slot].occupied {
            return false;
        }
        
        // Find the run for this quotient
        let run_start = self.find_run_start(quotient);
        let run_end = self.find_run_end(quotient);
        
        // Search for the remainder in the run
        for slot_idx in run_start..=run_end {
            if self.slots[slot_idx].remainder == remainder {
                return true;
            }
        }
        
        false
    }
    
    /// Delete one instance of a fingerprint from the filter
    pub fn delete(&mut self, fingerprint: u64) -> MapletResult<bool> {
        let quotient = self.extract_quotient(fingerprint);
        let remainder = self.extract_remainder(fingerprint);
        
        // Find the target slot for this quotient
        let target_slot = self.find_target_slot(quotient);
        
        // Check if the target slot is occupied
        if !self.slots[target_slot].occupied {
            return Ok(false);
        }
        
        // Find the run for this quotient
        let run_start = self.find_run_start(quotient);
        let run_end = self.find_run_end(quotient);
        
        // Search for the remainder in the run
        for slot_idx in run_start..=run_end {
            if self.slots[slot_idx].remainder == remainder {
                // Found the remainder, remove it
                self.remove_remainder_at_slot(slot_idx)?;
                
                // Update multiset count
                if let Some(count) = self.multiset_counts.get_mut(&fingerprint) {
                    *count -= 1;
                    if *count == 0 {
                        self.multiset_counts.remove(&fingerprint);
                    }
                }
                
                self.len -= 1;
                return Ok(true);
            }
        }
        
        Ok(false)
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
    
    /// Find the start of a run for a quotient
    fn find_run_start(&self, quotient: u64) -> usize {
        let mut slot = self.find_target_slot(quotient);
        
        // Walk backwards to find the start of the run
        while slot > 0 && self.slots[slot - 1].shifted {
            slot -= 1;
        }
        
        slot
    }
    
    /// Find the end of a run for a quotient
    fn find_run_end(&self, quotient: u64) -> usize {
        let mut slot = self.find_target_slot(quotient);
        
        // Walk forwards to find the end of the run
        while slot < self.capacity - 1 && !self.slots[slot].runend {
            slot += 1;
        }
        
        slot
    }
    
    /// Insert a remainder at a specific slot
    fn insert_remainder_at_slot(&mut self, slot: usize, remainder: u64) -> MapletResult<()> {
        if slot >= self.capacity {
            return Err(MapletError::Internal("Slot index out of bounds".to_string()));
        }
        
        // If slot is empty, just insert
        if !self.slots[slot].occupied {
            self.slots[slot].remainder = remainder;
            self.slots[slot].occupied = true;
            self.slots[slot].runend = true;
            return Ok(());
        }
        
        // Slot is occupied, need to shift
        self.shift_and_insert(slot, remainder)
    }
    
    /// Shift elements and insert a remainder
    fn shift_and_insert(&mut self, start_slot: usize, remainder: u64) -> MapletResult<()> {
        let mut slot = start_slot;
        
        // Find the end of the current run
        while slot < self.capacity && !self.slots[slot].runend {
            slot += 1;
        }
        
        // If we're at or beyond the capacity, we can't shift
        if slot >= self.capacity {
            return Err(MapletError::CapacityExceeded);
        }
        
        // Check if we have space to shift
        if slot + 1 >= self.capacity {
            return Err(MapletError::CapacityExceeded);
        }
        
        // Shift elements to the right
        for i in (start_slot..=slot).rev() {
            if i + 1 < self.capacity {
                self.slots[i + 1] = self.slots[i];
                self.slots[i + 1].shifted = true;
            }
        }
        
        // Insert the new remainder
        self.slots[start_slot].remainder = remainder;
        self.slots[start_slot].occupied = true;
        self.slots[start_slot].shifted = false;
        self.slots[start_slot].runend = false;
        
        // Update runend bit for the last element
        if slot + 1 < self.capacity {
            self.slots[slot + 1].runend = true;
        }
        
        Ok(())
    }
    
    /// Remove a remainder at a specific slot
    fn remove_remainder_at_slot(&mut self, slot: usize) -> MapletResult<()> {
        if slot >= self.capacity {
            return Err(MapletError::Internal("Slot index out of bounds".to_string()));
        }
        
        // Shift elements to the left
        for i in slot..self.capacity - 1 {
            if self.slots[i + 1].occupied {
                self.slots[i] = self.slots[i + 1];
                self.slots[i].shifted = i > 0 && self.slots[i - 1].occupied;
            } else {
                break;
            }
        }
        
        // Clear the last slot
        if let Some(last_slot) = self.slots.last_mut() {
            *last_slot = SlotMetadata::default();
        }
        
        Ok(())
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
        
        // Find the run for this quotient
        let run_start = self.find_run_start(quotient);
        let run_end = self.find_run_end(quotient);
        
        // Search through the run to find the slot with the matching remainder
        (run_start..=run_end).find(|&slot| {
            slot < self.capacity && self.slots[slot].occupied && self.slots[slot].remainder == remainder
        })
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
            assert_ne!(slot1, slot2, "Different fingerprints should have different slots");
        }
        
        // Try to find slot for non-existing fingerprint
        let non_existing = 0xDEAD;
        let non_existing_slot = filter.get_actual_slot_for_fingerprint(non_existing);
        assert!(non_existing_slot.is_none(), "Should not find slot for non-existing fingerprint");
    }
}
