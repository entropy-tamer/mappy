//! Dynamic resizing and merging
//!
//! Implements dynamic resizing and merging capabilities for maplets.

use crate::{MapletError, MapletResult};

/// Resize manager for maplets
#[derive(Debug, Clone)]
pub struct ResizeManager {
    /// Current capacity
    capacity: usize,
    /// Growth factor for resizing
    growth_factor: f64,
    /// Maximum capacity
    max_capacity: usize,
}

impl ResizeManager {
    /// Create a new resize manager
    #[must_use]
    pub const fn new(initial_capacity: usize) -> Self {
        Self {
            capacity: initial_capacity,
            growth_factor: 2.0,
            max_capacity: usize::MAX,
        }
    }

    /// Calculate new capacity for resizing
    #[must_use]
    pub fn calculate_new_capacity(&self, _current_load: usize) -> usize {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        {
            let new_capacity = (self.capacity as f64 * self.growth_factor) as usize;
            new_capacity.min(self.max_capacity)
        }
    }

    /// Check if resizing is needed
    #[must_use]
    pub fn should_resize(&self, current_load: usize, max_load_factor: f64) -> bool {
        #[allow(clippy::cast_precision_loss)]
        {
            let load_factor = current_load as f64 / self.capacity as f64;
            load_factor > max_load_factor
        }
    }

    /// Update capacity after resize
    pub const fn update_capacity(&mut self, new_capacity: usize) {
        self.capacity = new_capacity;
    }
}

/// Merge manager for combining maplets
#[derive(Debug, Clone)]
pub struct MergeManager {
    /// Maximum merge operations allowed
    max_merges: usize,
    /// Current merge count
    merge_count: usize,
}

impl MergeManager {
    /// Create a new merge manager
    #[must_use]
    pub const fn new(max_merges: usize) -> Self {
        Self {
            max_merges,
            merge_count: 0,
        }
    }

    /// Check if merge is allowed
    #[must_use]
    pub const fn can_merge(&self) -> bool {
        self.merge_count < self.max_merges
    }

    /// Record a merge operation
    ///
    /// # Errors
    ///
    /// Returns an error if the merge limit is exceeded
    pub fn record_merge(&mut self) -> MapletResult<()> {
        if !self.can_merge() {
            return Err(MapletError::MergeFailed(
                "Maximum merges exceeded".to_string(),
            ));
        }
        self.merge_count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_manager() {
        let mut manager = ResizeManager::new(100);

        assert_eq!(manager.capacity, 100);
        assert!(!manager.should_resize(50, 0.8));
        assert!(manager.should_resize(90, 0.8));

        let new_capacity = manager.calculate_new_capacity(90);
        assert_eq!(new_capacity, 200);

        manager.update_capacity(200);
        assert_eq!(manager.capacity, 200);
    }

    #[test]
    fn test_resize_manager_clone() {
        let manager = ResizeManager::new(100);
        let cloned = manager;
        assert_eq!(cloned.capacity, 100);
    }

    #[test]
    fn test_resize_manager_zero_capacity() {
        let manager = ResizeManager::new(0);
        assert_eq!(manager.capacity, 0);
    }

    #[test]
    fn test_resize_manager_large_capacity() {
        let manager = ResizeManager::new(10000);
        assert_eq!(manager.capacity, 10000);
    }

    #[test]
    fn test_resize_manager_calculate_new_capacity_max() {
        let manager = ResizeManager::new(usize::MAX / 2);
        let new_capacity = manager.calculate_new_capacity(0);
        assert_eq!(new_capacity, usize::MAX);
    }

    #[test]
    fn test_resize_manager_should_resize_edge() {
        let manager = ResizeManager::new(100);
        assert!(!manager.should_resize(80, 0.8));
        assert!(manager.should_resize(81, 0.8));
    }

    #[test]
    fn test_resize_manager_should_resize_zero_load() {
        let manager = ResizeManager::new(100);
        assert!(!manager.should_resize(0, 0.8));
    }

    #[test]
    fn test_merge_manager() {
        let mut manager = MergeManager::new(5);

        assert!(manager.can_merge());

        for _ in 0..5 {
            assert!(manager.record_merge().is_ok());
        }

        assert!(!manager.can_merge());
        assert!(manager.record_merge().is_err());
    }

    #[test]
    fn test_merge_manager_clone() {
        let mut manager = MergeManager::new(5);
        manager.record_merge().ok();

        let cloned = manager.clone();
        assert_eq!(cloned.merge_count, 1);
    }

    #[test]
    fn test_merge_manager_zero_max() {
        let manager = MergeManager::new(0);
        assert!(!manager.can_merge());
    }

    #[test]
    fn test_merge_manager_large_max() {
        let manager = MergeManager::new(1000);
        assert!(manager.can_merge());
    }

    #[test]
    fn test_merge_manager_partial_merges() {
        let mut manager = MergeManager::new(10);

        for _ in 0..5 {
            assert!(manager.record_merge().is_ok());
        }

        assert_eq!(manager.merge_count, 5);
        assert!(manager.can_merge());
    }
}
