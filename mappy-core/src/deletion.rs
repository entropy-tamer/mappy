//! Multiset-based deletion support
//!
//! Implements deletion support for maplets using the multiset approach
//! described in Section 2 of the research paper.

use crate::MapletResult;

/// Multiset deletion manager
#[derive(Debug, Clone)]
pub struct DeletionManager {
    /// Track multiple instances per fingerprint
    instance_counts: std::collections::HashMap<u64, usize>,
    /// Track which slots contain which instances
    slot_instances: std::collections::HashMap<usize, Vec<u64>>,
}

impl Default for DeletionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeletionManager {
    /// Create a new deletion manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            instance_counts: std::collections::HashMap::new(),
            slot_instances: std::collections::HashMap::new(),
        }
    }

    /// Add an instance of a fingerprint
    pub fn add_instance(&mut self, fingerprint: u64, slot: usize) {
        *self.instance_counts.entry(fingerprint).or_insert(0) += 1;
        self.slot_instances
            .entry(slot)
            .or_default()
            .push(fingerprint);
    }

    /// Remove an instance of a fingerprint
    /// # Errors
    ///
    /// Returns an error if the removal operation fails
    pub fn remove_instance(&mut self, fingerprint: u64, slot: usize) -> MapletResult<bool> {
        if let Some(count) = self.instance_counts.get_mut(&fingerprint)
            && *count > 0
        {
            *count -= 1;
            if *count == 0 {
                self.instance_counts.remove(&fingerprint);
            }

            // Remove from slot instances
            if let Some(instances) = self.slot_instances.get_mut(&slot) {
                instances.retain(|&fp| fp != fingerprint);
            }

            return Ok(true);
        }
        Ok(false)
    }

    /// Get the number of instances for a fingerprint
    #[must_use]
    pub fn instance_count(&self, fingerprint: u64) -> usize {
        self.instance_counts.get(&fingerprint).copied().unwrap_or(0)
    }

    /// Check if a fingerprint has any instances
    #[must_use]
    pub fn has_instances(&self, fingerprint: u64) -> bool {
        self.instance_count(fingerprint) > 0
    }

    /// Get all fingerprints for a slot
    #[must_use]
    pub fn get_slot_fingerprints(&self, slot: usize) -> Vec<u64> {
        self.slot_instances.get(&slot).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deletion_manager() {
        let mut manager = DeletionManager::new();

        manager.add_instance(0x1234, 0);
        manager.add_instance(0x1234, 1);
        manager.add_instance(0x5678, 2);

        assert_eq!(manager.instance_count(0x1234), 2);
        assert_eq!(manager.instance_count(0x5678), 1);
        assert!(manager.has_instances(0x1234));

        assert!(manager.remove_instance(0x1234, 0).unwrap());
        assert_eq!(manager.instance_count(0x1234), 1);

        assert!(manager.remove_instance(0x1234, 1).unwrap());
        assert_eq!(manager.instance_count(0x1234), 0);
        assert!(!manager.has_instances(0x1234));
    }

    #[test]
    fn test_deletion_manager_default() {
        let manager = DeletionManager::default();
        assert_eq!(manager.instance_count(0x1234), 0);
        assert!(!manager.has_instances(0x1234));
    }

    #[test]
    fn test_deletion_manager_empty() {
        let manager = DeletionManager::new();
        assert_eq!(manager.instance_count(0x1234), 0);
        assert!(!manager.has_instances(0x1234));
    }

    #[test]
    fn test_deletion_manager_remove_nonexistent() {
        let mut manager = DeletionManager::new();
        assert!(!manager.remove_instance(0x1234, 0).unwrap());
    }

    #[test]
    fn test_deletion_manager_get_slot_fingerprints() {
        let mut manager = DeletionManager::new();
        manager.add_instance(0x1234, 0);
        manager.add_instance(0x5678, 0);

        let fingerprints = manager.get_slot_fingerprints(0);
        assert_eq!(fingerprints.len(), 2);
    }

    #[test]
    fn test_deletion_manager_get_slot_fingerprints_empty() {
        let manager = DeletionManager::new();
        let fingerprints = manager.get_slot_fingerprints(0);
        assert_eq!(fingerprints.len(), 0);
    }

    #[test]
    fn test_deletion_manager_clone() {
        let mut manager = DeletionManager::new();
        manager.add_instance(0x1234, 0);

        let cloned = manager.clone();
        assert_eq!(cloned.instance_count(0x1234), 1);
    }

    #[test]
    fn test_deletion_manager_multiple_adds_same_slot() {
        let mut manager = DeletionManager::new();
        manager.add_instance(0x1234, 0);
        manager.add_instance(0x5678, 0);
        manager.add_instance(0x9ABC, 0);

        assert_eq!(manager.instance_count(0x1234), 1);
        assert_eq!(manager.instance_count(0x5678), 1);
        assert_eq!(manager.instance_count(0x9ABC), 1);
    }

    #[test]
    fn test_deletion_manager_remove_from_slot() {
        let mut manager = DeletionManager::new();
        manager.add_instance(0x1234, 0);
        manager.add_instance(0x5678, 0);

        manager.remove_instance(0x1234, 0).unwrap();
        let fingerprints = manager.get_slot_fingerprints(0);
        assert_eq!(fingerprints.len(), 1);
        assert_eq!(fingerprints[0], 0x5678);
    }

    #[test]
    fn test_deletion_manager_zero_count_after_remove() {
        let mut manager = DeletionManager::new();
        manager.add_instance(0x1234, 0);
        manager.add_instance(0x1234, 1);

        manager.remove_instance(0x1234, 0).unwrap();
        manager.remove_instance(0x1234, 1).unwrap();

        assert_eq!(manager.instance_count(0x1234), 0);
        assert!(!manager.has_instances(0x1234));
    }
}
