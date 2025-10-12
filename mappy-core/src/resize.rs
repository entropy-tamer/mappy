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
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            capacity: initial_capacity,
            growth_factor: 2.0,
            max_capacity: usize::MAX,
        }
    }
    
    /// Calculate new capacity for resizing
    pub fn calculate_new_capacity(&self, _current_load: usize) -> usize {
        let new_capacity = (self.capacity as f64 * self.growth_factor) as usize;
        new_capacity.min(self.max_capacity)
    }
    
    /// Check if resizing is needed
    pub fn should_resize(&self, current_load: usize, max_load_factor: f64) -> bool {
        let load_factor = current_load as f64 / self.capacity as f64;
        load_factor > max_load_factor
    }
    
    /// Update capacity after resize
    pub fn update_capacity(&mut self, new_capacity: usize) {
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
    pub fn new(max_merges: usize) -> Self {
        Self {
            max_merges,
            merge_count: 0,
        }
    }
    
    /// Check if merge is allowed
    pub fn can_merge(&self) -> bool {
        self.merge_count < self.max_merges
    }
    
    /// Record a merge operation
    pub fn record_merge(&mut self) -> MapletResult<()> {
        if !self.can_merge() {
            return Err(MapletError::MergeFailed("Maximum merges exceeded".to_string()));
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
    fn test_merge_manager() {
        let mut manager = MergeManager::new(5);
        
        assert!(manager.can_merge());
        
        for _ in 0..5 {
            assert!(manager.record_merge().is_ok());
        }
        
        assert!(!manager.can_merge());
        assert!(manager.record_merge().is_err());
    }
}
