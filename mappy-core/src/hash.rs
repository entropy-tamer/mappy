//! Fingerprint hashing and perfect hash foundation
//!
//! Implements the hashing layer for maplets, providing fingerprint-based
//! hashing with configurable hash functions and collision detection.

use crate::{MapletError, MapletResult};
use ahash::RandomState;
use std::hash::{Hash, Hasher};

/// Hash function types supported by the maplet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HashFunction {
    /// `AHash` - fast, high-quality hashing
    #[default]
    AHash,
    /// `TwoX` - deterministic, good distribution
    TwoX,
    /// FNV - simple, fast for small keys
    Fnv,
}

/// Fingerprint-based hasher for maplets
#[derive(Debug, Clone)]
pub struct FingerprintHasher {
    /// Hash function to use
    hash_fn: HashFunction,
    /// Random state for `AHash`
    random_state: RandomState,
    /// Fingerprint size in bits
    fingerprint_bits: u32,
    /// Mask for fingerprint extraction
    fingerprint_mask: u64,
}

impl FingerprintHasher {
    /// Create a new fingerprint hasher
    #[must_use]
    pub fn new(hash_fn: HashFunction, fingerprint_bits: u32) -> Self {
        let fingerprint_mask = if fingerprint_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << fingerprint_bits) - 1
        };

        Self {
            hash_fn,
            random_state: RandomState::with_seed(42), // Use consistent seed
            fingerprint_bits,
            fingerprint_mask,
        }
    }

    /// Calculate fingerprint for a key
    pub fn fingerprint<T: Hash>(&self, key: &T) -> u64 {
        let hash = self.hash_key(key);
        hash & self.fingerprint_mask
    }

    /// Calculate full hash for a key
    pub fn hash_key<T: Hash>(&self, key: &T) -> u64 {
        match self.hash_fn {
            HashFunction::AHash => self.random_state.hash_one(key),
            HashFunction::TwoX => {
                use twox_hash::XxHash64;
                let mut hasher = XxHash64::default();
                key.hash(&mut hasher);
                hasher.finish()
            }
            HashFunction::Fnv => {
                use std::collections::hash_map::DefaultHasher;
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                hasher.finish()
            }
        }
    }

    /// Get the configured hash function
    #[must_use]
    pub const fn hash_fn(&self) -> HashFunction {
        self.hash_fn
    }

    /// Get the fingerprint size in bits
    #[must_use]
    pub const fn fingerprint_bits(&self) -> u32 {
        self.fingerprint_bits
    }

    /// Get the fingerprint mask
    #[must_use]
    pub const fn fingerprint_mask(&self) -> u64 {
        self.fingerprint_mask
    }

    /// Calculate the optimal fingerprint size for a given false-positive rate
    #[must_use]
    pub fn optimal_fingerprint_size(false_positive_rate: f64) -> u32 {
        if false_positive_rate <= 0.0 || false_positive_rate >= 1.0 {
            return 8; // Default to 8 bits
        }

        // From the paper: fingerprint size = ⌈log₂(1/ε)⌉ + 3
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // Acceptable for bit calculation
        let bits = (-false_positive_rate.log2()).ceil() as u32 + 3;
        bits.clamp(4, 64) // Clamp between 4 and 64 bits
    }
}

/// Perfect hash function for mapping fingerprints to slots
#[derive(Debug, Clone)]
pub struct PerfectHash {
    /// Number of slots available
    num_slots: usize,
    /// Hash function for slot mapping
    slot_hasher: FingerprintHasher,
}

impl PerfectHash {
    /// Create a new perfect hash function
    #[must_use]
    pub fn new(num_slots: usize, hash_fn: HashFunction) -> Self {
        // Use a different hash function for slot mapping to avoid correlation
        let slot_hash_fn = match hash_fn {
            HashFunction::AHash => HashFunction::TwoX,
            HashFunction::TwoX => HashFunction::Fnv,
            HashFunction::Fnv => HashFunction::AHash,
        };

        Self {
            num_slots,
            slot_hasher: FingerprintHasher::new(slot_hash_fn, 32), // 32 bits for slot hashing
        }
    }

    /// Map a fingerprint to a slot index
    #[must_use]
    pub fn slot_index(&self, fingerprint: u64) -> usize {
        let hash = self.slot_hasher.hash_key(&fingerprint);
        #[allow(clippy::cast_possible_truncation)] // Acceptable for slot calculation
        let hash_usize = hash as usize;
        hash_usize % self.num_slots
    }

    /// Get the number of slots
    #[must_use]
    pub const fn num_slots(&self) -> usize {
        self.num_slots
    }
}

/// Collision detector for tracking hash collisions
#[derive(Debug, Clone)]
pub struct CollisionDetector {
    /// Maximum number of collisions to track
    max_collisions: usize,
    /// Current collision count
    collision_count: usize,
    /// Collision threshold for warnings
    warning_threshold: usize,
}

impl CollisionDetector {
    /// Create a new collision detector
    #[must_use]
    pub const fn new(max_collisions: usize) -> Self {
        Self {
            max_collisions,
            collision_count: 0,
            warning_threshold: max_collisions / 2,
        }
    }

    /// Record a collision
    ///
    /// # Errors
    /// Returns an error if the collision limit is exceeded.
    pub fn record_collision(&mut self) -> MapletResult<()> {
        self.collision_count += 1;

        if self.collision_count > self.max_collisions {
            return Err(MapletError::CollisionLimitExceeded);
        }

        if self.collision_count > self.warning_threshold {
            tracing::warn!(
                "High collision count: {} (threshold: {})",
                self.collision_count,
                self.warning_threshold
            );
        }

        Ok(())
    }

    /// Get the current collision count
    #[must_use]
    pub const fn collision_count(&self) -> usize {
        self.collision_count
    }

    /// Reset the collision count
    pub const fn reset(&mut self) {
        self.collision_count = 0;
    }

    /// Check if we're approaching the collision limit
    #[must_use]
    pub const fn is_approaching_limit(&self) -> bool {
        self.collision_count > self.warning_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_hasher() {
        let hasher = FingerprintHasher::new(HashFunction::AHash, 8);
        let key = "test_key";

        let fingerprint = hasher.fingerprint(&key);
        assert!(fingerprint < (1u64 << 8));

        let fingerprint2 = hasher.fingerprint(&key);
        assert_eq!(fingerprint, fingerprint2);
    }

    #[test]
    fn test_fingerprint_hasher_twox() {
        let hasher = FingerprintHasher::new(HashFunction::TwoX, 16);
        let key = "test_key";

        let fingerprint = hasher.fingerprint(&key);
        assert!(fingerprint < (1u64 << 16));

        let fingerprint2 = hasher.fingerprint(&key);
        assert_eq!(fingerprint, fingerprint2);
    }

    #[test]
    fn test_fingerprint_hasher_fnv() {
        let hasher = FingerprintHasher::new(HashFunction::Fnv, 12);
        let key = "test_key";

        let fingerprint = hasher.fingerprint(&key);
        assert!(fingerprint < (1u64 << 12));
    }

    #[test]
    fn test_fingerprint_hasher_64_bits() {
        let hasher = FingerprintHasher::new(HashFunction::AHash, 64);
        let key = "test_key";

        let fingerprint = hasher.fingerprint(&key);

        // This asserted `fingerprint <= u64::MAX` on a `u64`, which cannot
        // fail. What makes "64 bits" a claim worth testing is that the hasher
        // reaches past the width the sibling 12-bit test pins, and that it is
        // deterministic for a fixed key.
        assert!(
            fingerprint >= (1u64 << 12),
            "a 64-bit hasher must be able to exceed 12 bits, got {fingerprint}"
        );
        assert_eq!(
            fingerprint,
            hasher.fingerprint(&key),
            "fingerprinting must be deterministic for a fixed key"
        );
    }

    #[test]
    fn test_fingerprint_hasher_different_keys() {
        let hasher = FingerprintHasher::new(HashFunction::AHash, 16);
        let key1 = "key1";
        let key2 = "key2";

        let fp1 = hasher.fingerprint(&key1);
        let fp2 = hasher.fingerprint(&key2);

        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_hasher_clone() {
        let hasher = FingerprintHasher::new(HashFunction::AHash, 16);
        let cloned = hasher.clone();
        let key = "test_key";

        assert_eq!(hasher.fingerprint(&key), cloned.fingerprint(&key));
    }

    #[test]
    fn test_fingerprint_hasher_default() {
        let hasher = FingerprintHasher::new(HashFunction::default(), 16);
        assert_eq!(hasher.hash_fn, HashFunction::AHash);
    }

    #[test]
    fn test_perfect_hash() {
        let perfect_hash = PerfectHash::new(100, HashFunction::AHash);
        let fingerprint = 12345u64;

        let slot = perfect_hash.slot_index(fingerprint);
        assert!(slot < 100);

        let slot2 = perfect_hash.slot_index(fingerprint);
        assert_eq!(slot, slot2);
    }

    #[test]
    fn test_perfect_hash_different_fingerprints() {
        let perfect_hash = PerfectHash::new(100, HashFunction::AHash);

        let slot1 = perfect_hash.slot_index(12345);
        let slot2 = perfect_hash.slot_index(54321);

        assert!(slot1 < 100);
        assert!(slot2 < 100);
    }

    #[test]
    fn test_perfect_hash_clone() {
        let perfect_hash = PerfectHash::new(100, HashFunction::AHash);
        let cloned = perfect_hash.clone();

        let fingerprint = 12345u64;
        assert_eq!(
            perfect_hash.slot_index(fingerprint),
            cloned.slot_index(fingerprint)
        );
    }

    #[test]
    fn test_collision_detector() {
        let mut detector = CollisionDetector::new(10);

        for _ in 0..5 {
            assert!(detector.record_collision().is_ok());
        }

        assert_eq!(detector.collision_count(), 5);
        assert!(!detector.is_approaching_limit());

        for _ in 0..5 {
            assert!(detector.record_collision().is_ok());
        }

        assert_eq!(detector.collision_count(), 10);
        assert!(detector.is_approaching_limit());

        assert!(detector.record_collision().is_err());
    }

    #[test]
    fn test_collision_detector_reset() {
        let mut detector = CollisionDetector::new(10);
        detector.record_collision().ok();

        assert_eq!(detector.collision_count(), 1);

        detector.reset();
        assert_eq!(detector.collision_count(), 0);
    }

    #[test]
    fn test_collision_detector_clone() {
        let mut detector = CollisionDetector::new(10);
        detector.record_collision().ok();

        let cloned = detector.clone();
        assert_eq!(cloned.collision_count(), 1);
    }

    #[test]
    fn test_collision_detector_max_zero() {
        let mut detector = CollisionDetector::new(0);
        assert!(detector.record_collision().is_err());
    }

    #[test]
    fn test_optimal_fingerprint_size() {
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(0.01), 10);
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(0.001), 13);
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(0.1), 7);
    }

    #[test]
    fn test_optimal_fingerprint_size_edge_cases() {
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(0.0), 8);
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(1.0), 8);
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(-0.5), 8);
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(2.0), 8);
    }

    #[test]
    fn test_hash_function_eq() {
        assert_eq!(HashFunction::AHash, HashFunction::AHash);
        assert_ne!(HashFunction::AHash, HashFunction::TwoX);
    }

    #[test]
    fn test_fingerprint_hasher_getters() {
        let hasher = FingerprintHasher::new(HashFunction::AHash, 16);
        assert_eq!(hasher.fingerprint_bits(), 16);
        assert_eq!(hasher.fingerprint_mask(), (1u64 << 16) - 1);
    }

    #[test]
    fn test_perfect_hash_getters() {
        let perfect_hash = PerfectHash::new(100, HashFunction::AHash);
        assert_eq!(perfect_hash.num_slots(), 100);
    }
}
