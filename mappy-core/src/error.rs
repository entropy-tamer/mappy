//! Error rate control and strong maplet property
//!
//! Implements error rate control and validation of the strong maplet property
//! as described in Section 2 of the research paper.

#![allow(clippy::cast_precision_loss)] // Acceptable for error rate calculations

use crate::{MapletError, MapletResult};

/// Error rate controller for maplets
#[derive(Debug, Clone)]
pub struct ErrorRateController {
    /// Target false-positive rate
    target_error_rate: f64,
    /// Current estimated error rate
    current_error_rate: f64,
    /// Number of queries performed
    query_count: u64,
    /// Number of false positives detected
    false_positive_count: u64,
    /// Collision tracking for error analysis
    collision_tracker: CollisionTracker,
}

impl ErrorRateController {
    /// Create a new error rate controller
    pub fn new(target_error_rate: f64) -> MapletResult<Self> {
        if target_error_rate <= 0.0 || target_error_rate >= 1.0 {
            return Err(MapletError::InvalidErrorRate(target_error_rate));
        }

        Ok(Self {
            target_error_rate,
            current_error_rate: 0.0,
            query_count: 0,
            false_positive_count: 0,
            collision_tracker: CollisionTracker::new(),
        })
    }

    /// Record a query result
    pub fn record_query(&mut self, was_false_positive: bool) {
        self.query_count += 1;
        if was_false_positive {
            self.false_positive_count += 1;
        }

        // Update current error rate estimate
        #[allow(clippy::cast_precision_loss)] // Acceptable for ratio calculation
        {
            self.current_error_rate = self.false_positive_count as f64 / self.query_count as f64;
        }
    }

    /// Record a hash collision
    pub fn record_collision(&mut self, fingerprint: u64, slot: usize) {
        self.collision_tracker.record_collision(fingerprint, slot);
    }

    /// Get the current error rate
    #[must_use]
    pub const fn current_error_rate(&self) -> f64 {
        self.current_error_rate
    }

    /// Get the target error rate
    #[must_use]
    pub const fn target_error_rate(&self) -> f64 {
        self.target_error_rate
    }

    /// Check if the error rate is within acceptable bounds
    #[must_use]
    pub fn is_error_rate_acceptable(&self) -> bool {
        self.current_error_rate <= self.target_error_rate * 1.5 // Allow 50% tolerance
    }

    /// Get error rate statistics
    #[must_use]
    pub fn stats(&self) -> ErrorRateStats {
        ErrorRateStats {
            target_error_rate: self.target_error_rate,
            current_error_rate: self.current_error_rate,
            query_count: self.query_count,
            false_positive_count: self.false_positive_count,
            collision_count: self.collision_tracker.collision_count(),
            max_chain_length: self.collision_tracker.max_chain_length(),
        }
    }
}

/// Collision tracker for monitoring hash collisions
#[derive(Debug, Clone)]
pub struct CollisionTracker {
    /// Map from fingerprint to list of slots it maps to
    fingerprint_to_slots: std::collections::HashMap<u64, Vec<usize>>,
    /// Map from slot to list of fingerprints that map to it
    slot_to_fingerprints: std::collections::HashMap<usize, Vec<u64>>,
}

impl Default for CollisionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CollisionTracker {
    /// Create a new collision tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            fingerprint_to_slots: std::collections::HashMap::new(),
            slot_to_fingerprints: std::collections::HashMap::new(),
        }
    }

    /// Record a collision
    pub fn record_collision(&mut self, fingerprint: u64, slot: usize) {
        self.fingerprint_to_slots
            .entry(fingerprint)
            .or_default()
            .push(slot);
        self.slot_to_fingerprints
            .entry(slot)
            .or_default()
            .push(fingerprint);
    }

    /// Get the number of collisions
    #[must_use]
    pub fn collision_count(&self) -> usize {
        self.slot_to_fingerprints
            .values()
            .filter(|fingerprints| fingerprints.len() > 1)
            .count()
    }

    /// Get the maximum chain length
    #[must_use]
    pub fn max_chain_length(&self) -> usize {
        self.slot_to_fingerprints
            .values()
            .map(std::vec::Vec::len)
            .max()
            .unwrap_or(0)
    }

    /// Get fingerprints that collide with a given fingerprint
    #[must_use]
    pub fn get_colliding_fingerprints(&self, fingerprint: u64) -> Vec<u64> {
        self.fingerprint_to_slots
            .get(&fingerprint)
            .map_or_else(Vec::new, |slots| {
                let mut colliding = Vec::new();
                for &slot in slots {
                    if let Some(fingerprints) = self.slot_to_fingerprints.get(&slot) {
                        for &fp in fingerprints {
                            if fp != fingerprint {
                                colliding.push(fp);
                            }
                        }
                    }
                }
                colliding.sort_unstable();
                colliding.dedup();
                colliding
            })
    }
}

/// Strong maplet property validator
#[derive(Debug, Clone)]
pub struct StrongMapletValidator {
    /// Maximum allowed chain length for strong property
    max_chain_length: usize,
    /// Error rate threshold for strong property
    error_threshold: f64,
}

impl StrongMapletValidator {
    /// Create a new strong maplet validator
    #[must_use]
    pub const fn new(max_chain_length: usize, error_threshold: f64) -> Self {
        Self {
            max_chain_length,
            error_threshold,
        }
    }

    /// Validate the strong maplet property
    ///
    /// The strong maplet property states that:
    /// m[k] = M[k] ⊕ (⊕ᵢ₌₁ˡ M[kᵢ])
    /// where Pr[ℓ ≥ L] ≤ ε^L
    pub fn validate_strong_property<V, Op>(
        &self,
        collision_tracker: &CollisionTracker,
        error_rate: f64,
    ) -> MapletResult<StrongMapletValidation>
    where
        V: Clone,
        Op: crate::operators::MergeOperator<V>,
    {
        let max_chain = collision_tracker.max_chain_length();
        let collision_count = collision_tracker.collision_count();

        // Check if chain length exceeds maximum
        let chain_length_ok = max_chain <= self.max_chain_length;

        // Check if error rate is within bounds
        let error_rate_ok = error_rate <= self.error_threshold;

        // Calculate probability of exceeding chain length
        let prob_exceed_chain = if max_chain > 0 {
            error_rate.powi(max_chain as i32)
        } else {
            0.0
        };

        let validation = StrongMapletValidation {
            chain_length_ok,
            error_rate_ok,
            max_chain_length: max_chain,
            collision_count,
            error_rate,
            prob_exceed_chain,
            is_valid: chain_length_ok && error_rate_ok,
        };

        if !validation.is_valid {
            tracing::warn!(
                "Strong maplet property validation failed: chain_length_ok={}, error_rate_ok={}",
                chain_length_ok,
                error_rate_ok
            );
        }

        Ok(validation)
    }
}

/// Result of strong maplet property validation
#[derive(Debug, Clone)]
pub struct StrongMapletValidation {
    /// Whether chain length is within bounds
    pub chain_length_ok: bool,
    /// Whether error rate is within bounds
    pub error_rate_ok: bool,
    /// Maximum chain length observed
    pub max_chain_length: usize,
    /// Total number of collisions
    pub collision_count: usize,
    /// Current error rate
    pub error_rate: f64,
    /// Probability of exceeding chain length
    pub prob_exceed_chain: f64,
    /// Overall validation result
    pub is_valid: bool,
}

/// Error rate statistics
#[derive(Debug, Clone)]
pub struct ErrorRateStats {
    pub target_error_rate: f64,
    pub current_error_rate: f64,
    pub query_count: u64,
    pub false_positive_count: u64,
    pub collision_count: usize,
    pub max_chain_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_rate_controller() {
        let mut controller = ErrorRateController::new(0.01).unwrap();

        // Record some queries
        for _ in 0..100 {
            controller.record_query(false);
        }

        // Record a few false positives
        for _ in 0..1 {
            controller.record_query(true);
        }

        assert_eq!(controller.query_count, 101);
        assert_eq!(controller.false_positive_count, 1);
        assert!((controller.current_error_rate() - 1.0 / 101.0).abs() < 1e-10);
        assert!(controller.is_error_rate_acceptable());
    }

    #[test]
    fn test_collision_tracker() {
        let mut tracker = CollisionTracker::new();

        // Record some collisions
        tracker.record_collision(0x1234, 0);
        tracker.record_collision(0x5678, 0); // Collision at slot 0
        tracker.record_collision(0x9ABC, 1);

        assert_eq!(tracker.collision_count(), 1);
        assert_eq!(tracker.max_chain_length(), 2);

        let colliding = tracker.get_colliding_fingerprints(0x1234);
        assert_eq!(colliding, vec![0x5678]);
    }

    #[test]
    fn test_strong_maplet_validator() {
        let validator = StrongMapletValidator::new(5, 0.01);
        let mut tracker = CollisionTracker::new();

        // Add some collisions
        tracker.record_collision(0x1234, 0);
        tracker.record_collision(0x5678, 0);

        let validation = validator
            .validate_strong_property::<u64, crate::operators::CounterOperator>(&tracker, 0.005)
            .unwrap();

        assert!(validation.chain_length_ok);
        assert!(validation.error_rate_ok);
        assert!(validation.is_valid);
        assert_eq!(validation.max_chain_length, 2);
        assert_eq!(validation.collision_count, 1);
    }
}
