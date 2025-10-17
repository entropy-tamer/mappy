//! Fingerprint hashing and perfect hash foundation
//! 
//! Implements the hashing layer for maplets, providing fingerprint-based
//! hashing with configurable hash functions and collision detection.

use ahash::RandomState;
use std::hash::{Hash, Hasher, BuildHasher};
use crate::{MapletError, MapletResult};

/// Hash function types supported by the maplet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashFunction {
    /// AHash - fast, high-quality hashing
    AHash,
    /// TwoX - deterministic, good distribution
    TwoX,
    /// FNV - simple, fast for small keys
    Fnv,
}

impl Default for HashFunction {
    fn default() -> Self {
        Self::AHash
    }
}

/// Fingerprint-based hasher for maplets
#[derive(Debug, Clone)]
pub struct FingerprintHasher {
    /// Hash function to use
    hash_fn: HashFunction,
    /// Random state for AHash
    random_state: RandomState,
    /// Fingerprint size in bits
    fingerprint_bits: u32,
    /// Mask for fingerprint extraction
    fingerprint_mask: u64,
}

impl FingerprintHasher {
    /// Create a new fingerprint hasher
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
            HashFunction::AHash => {
                let mut hasher = self.random_state.build_hasher();
                key.hash(&mut hasher);
                hasher.finish()
            }
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
    
    /// Get the fingerprint size in bits
    pub fn fingerprint_bits(&self) -> u32 {
        self.fingerprint_bits
    }
    
    /// Get the fingerprint mask
    pub fn fingerprint_mask(&self) -> u64 {
        self.fingerprint_mask
    }
    
    /// Calculate the optimal fingerprint size for a given false-positive rate
    pub fn optimal_fingerprint_size(false_positive_rate: f64) -> u32 {
        if false_positive_rate <= 0.0 || false_positive_rate >= 1.0 {
            return 8; // Default to 8 bits
        }
        
        // From the paper: fingerprint size = ⌈log₂(1/ε)⌉ + 3
        let bits = (-false_positive_rate.log2()).ceil() as u32 + 3;
        bits.min(64).max(4) // Clamp between 4 and 64 bits
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
    pub fn slot_index(&self, fingerprint: u64) -> usize {
        let hash = self.slot_hasher.hash_key(&fingerprint);
        (hash as usize) % self.num_slots
    }
    
    /// Get the number of slots
    pub fn num_slots(&self) -> usize {
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
    pub fn new(max_collisions: usize) -> Self {
        Self {
            max_collisions,
            collision_count: 0,
            warning_threshold: max_collisions / 2,
        }
    }
    
    /// Record a collision
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
    pub fn collision_count(&self) -> usize {
        self.collision_count
    }
    
    /// Reset the collision count
    pub fn reset(&mut self) {
        self.collision_count = 0;
    }
    
    /// Check if we're approaching the collision limit
    pub fn is_approaching_limit(&self) -> bool {
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
        
        // Same key should produce same fingerprint
        let fingerprint2 = hasher.fingerprint(&key);
        assert_eq!(fingerprint, fingerprint2);
    }

    #[test]
    fn test_perfect_hash() {
        let perfect_hash = PerfectHash::new(100, HashFunction::AHash);
        let fingerprint = 12345u64;
        
        let slot = perfect_hash.slot_index(fingerprint);
        assert!(slot < 100);
        
        // Same fingerprint should map to same slot
        let slot2 = perfect_hash.slot_index(fingerprint);
        assert_eq!(slot, slot2);
    }

    #[test]
    fn test_collision_detector() {
        let mut detector = CollisionDetector::new(10);
        
        // Record some collisions
        for _ in 0..5 {
            assert!(detector.record_collision().is_ok());
        }
        
        assert_eq!(detector.collision_count(), 5);
        assert!(!detector.is_approaching_limit());
        
        // Record more collisions to trigger warning
        for _ in 0..5 {
            assert!(detector.record_collision().is_ok());
        }
        
        assert_eq!(detector.collision_count(), 10);
        assert!(detector.is_approaching_limit());
        
        // Try to record one more collision - should fail
        assert!(detector.record_collision().is_err());
    }

    #[test]
    fn test_optimal_fingerprint_size() {
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(0.01), 10); // log2(100) + 3 = 10
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(0.001), 13); // log2(1000) + 3 = 13
        assert_eq!(FingerprintHasher::optimal_fingerprint_size(0.1), 7); // log2(10) + 3 = 7
    }
}
