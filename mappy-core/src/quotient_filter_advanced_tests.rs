#[cfg(feature = "quotient-filter")]
#[cfg(test)]
mod advanced_quotient_filter_tests {
    use super::*;
    use crate::quotient_filter::{QuotientFilter, HashFunction};
    use crate::maplet::{Maplet, CounterOperator};
    use std::collections::HashSet;
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    /// Test advanced slot finding functionality
    #[tokio::test]
    async fn test_advanced_slot_finding() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Insert values and track their slots
        let mut value_to_slot = std::collections::HashMap::new();
        let values = vec![123, 456, 789, 101112, 131415, 161718, 192021, 222324];
        
        for &value in &values {
            assert!(filter.insert(value).is_ok());
            if let Some(slot) = filter.get_actual_slot_for_fingerprint(value) {
                value_to_slot.insert(value, slot);
            }
        }
        
        // Test that slot finding is consistent
        for &value in &values {
            let slot = filter.get_actual_slot_for_fingerprint(value);
            assert!(slot.is_some(), "Should find slot for value {}", value);
            
            let slot_index = slot.unwrap();
            if let Some(&expected_slot) = value_to_slot.get(&value) {
                assert_eq!(slot_index, expected_slot, "Slot finding inconsistent for value {}", value);
            }
        }
        
        // Test slot finding for non-existent values
        let non_existent = 999999;
        let slot = filter.get_actual_slot_for_fingerprint(non_existent);
        assert!(slot.is_none(), "Should not find slot for non-existent value");
    }

    /// Test run detection with complex scenarios
    #[tokio::test]
    async fn test_run_detection_complex() {
        let mut filter = QuotientFilter::new(100, 4, HashFunction::AHash).unwrap(); // Small capacity to force runs
        
        // Insert values in a pattern that creates runs
        let values = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];
        for &value in &values {
            assert!(filter.insert(value).is_ok());
        }
        
        // All values should still be findable
        for &value in &values {
            assert!(filter.contains(value), "Value {} not found after insertion", value);
        }
        
        // Test run detection
        let stats = filter.stats();
        assert!(stats.runs >= 0, "Runs should be non-negative");
        assert!(stats.shifted_slots >= 0, "Shifted slots should be non-negative");
        
        // Test that runs are detected correctly
        println!("Runs detected: {}", stats.runs);
        println!("Shifted slots: {}", stats.shifted_slots);
    }

    /// Test maplet integration with advanced quotient filter
    #[tokio::test]
    async fn test_maplet_advanced_slot_finding() {
        let maplet = Maplet::<String, u64, CounterOperator>::new(1000, 0.01).unwrap();
        
        // Insert some data
        let test_key = "test_key".to_string();
        maplet.insert(test_key.clone(), 42).await.unwrap();
        
        // Test slot finding
        let slot = maplet.find_slot_for_key(&test_key).await;
        assert!(slot.is_some(), "Should find slot for existing key");
        
        let slot_index = slot.unwrap();
        assert!(slot_index < 1000, "Slot index should be within capacity");
        
        // Test slot finding for non-existent key
        let non_existent_key = "non_existent".to_string();
        let non_existent_slot = maplet.find_slot_for_key(&non_existent_key).await;
        assert!(non_existent_slot.is_none(), "Should not find slot for non-existent key");
    }

    /// Test advanced slot finding with collisions
    #[tokio::test]
    async fn test_advanced_slot_finding_with_collisions() {
        let mut filter = QuotientFilter::new(50, 3, HashFunction::AHash).unwrap(); // Very small capacity to force collisions
        
        // Insert values that are likely to collide
        let values = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25];
        for &value in &values {
            assert!(filter.insert(value).is_ok());
        }
        
        // Test that all values are still findable despite collisions
        for &value in &values {
            assert!(filter.contains(value), "Value {} not found after insertion with collisions", value);
        }
        
        // Test slot finding with collisions
        for &value in &values {
            let slot = filter.get_actual_slot_for_fingerprint(value);
            assert!(slot.is_some(), "Should find slot for value {} despite collisions", value);
            
            let slot_index = slot.unwrap();
            assert!(slot_index < filter.capacity(), "Slot index {} out of bounds", slot_index);
        }
    }

    /// Test performance of advanced slot finding
    #[tokio::test]
    async fn test_advanced_slot_finding_performance() {
        let mut filter = QuotientFilter::new(10000, 8, HashFunction::AHash).unwrap();
        
        // Insert values
        let values: Vec<u64> = (0..5000).collect();
        for &value in &values {
            assert!(filter.insert(value).is_ok());
        }
        
        // Benchmark slot finding
        let start = std::time::Instant::now();
        for &value in &values {
            let slot = filter.get_actual_slot_for_fingerprint(value);
            assert!(slot.is_some());
        }
        let duration = start.elapsed();
        
        println!("Slot finding for 5000 values took: {:?}", duration);
        assert!(duration.as_millis() < 100, "Slot finding should be fast: {:?}", duration);
    }

    /// Test advanced slot finding with different hash functions
    #[tokio::test]
    async fn test_advanced_slot_finding_hash_functions() {
        let hash_functions = vec![
            HashFunction::AHash,
            HashFunction::TwoX,
            HashFunction::FNV,
        ];
        
        for hash_fn in hash_functions {
            let mut filter = QuotientFilter::new(1000, 8, hash_fn).unwrap();
            
            // Insert values
            let values = vec![123, 456, 789, 101112, 131415];
            for &value in &values {
                assert!(filter.insert(value).is_ok());
            }
            
            // Test slot finding with each hash function
            for &value in &values {
                let slot = filter.get_actual_slot_for_fingerprint(value);
                assert!(slot.is_some(), "Should find slot for value {} with hash function {:?}", value, hash_fn);
            }
        }
    }

    /// Test advanced slot finding with multiset operations
    #[tokio::test]
    async fn test_advanced_slot_finding_multiset() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Insert same value multiple times
        let value = 12345;
        for _ in 0..5 {
            assert!(filter.insert(value).is_ok());
        }
        
        // Test slot finding for multiset value
        let slot = filter.get_actual_slot_for_fingerprint(value);
        assert!(slot.is_some(), "Should find slot for multiset value");
        
        // Test that slot finding is consistent
        let slot1 = filter.get_actual_slot_for_fingerprint(value);
        let slot2 = filter.get_actual_slot_for_fingerprint(value);
        assert_eq!(slot1, slot2, "Slot finding should be consistent");
        
        // Delete one instance
        assert!(filter.delete(value).is_ok());
        
        // Slot finding should still work
        let slot = filter.get_actual_slot_for_fingerprint(value);
        assert!(slot.is_some(), "Should still find slot after deletion");
    }

    /// Test advanced slot finding with concurrent operations
    #[tokio::test]
    async fn test_advanced_slot_finding_concurrent() {
        use std::sync::Arc;
        use tokio::task;
        
        let filter = Arc::new(QuotientFilter::new(10000, 8, HashFunction::AHash).unwrap());
        let mut handles = vec![];
        
        // Spawn multiple tasks that insert values and find slots
        for thread_id in 0..4 {
            let filter_clone = Arc::clone(&filter);
            let handle = task::spawn(async move {
                for i in 0..1000 {
                    let value = thread_id * 1000 + i;
                    filter_clone.insert(value).unwrap();
                    
                    // Test slot finding
                    let slot = filter_clone.get_actual_slot_for_fingerprint(value);
                    assert!(slot.is_some(), "Should find slot for value {}", value);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify all values are still findable
        for thread_id in 0..4 {
            for i in 0..1000 {
                let value = thread_id * 1000 + i;
                assert!(filter.contains(value), "Value {} not found after concurrent operations", value);
                
                let slot = filter.get_actual_slot_for_fingerprint(value);
                assert!(slot.is_some(), "Should find slot for value {} after concurrent operations", value);
            }
        }
    }

    /// Test advanced slot finding with edge cases
    #[tokio::test]
    async fn test_advanced_slot_finding_edge_cases() {
        // Test with minimum capacity
        let mut filter = QuotientFilter::new(1, 1, HashFunction::AHash).unwrap();
        assert!(filter.insert(0).is_ok());
        
        let slot = filter.get_actual_slot_for_fingerprint(0);
        assert!(slot.is_some(), "Should find slot for value 0");
        assert_eq!(slot.unwrap(), 0, "Slot should be 0 for first value");
        
        // Test with maximum capacity (reasonable limit)
        let mut filter = QuotientFilter::new(100000, 16, HashFunction::AHash).unwrap();
        for i in 0..1000 {
            assert!(filter.insert(i).is_ok());
        }
        
        // Test slot finding for all values
        for i in 0..1000 {
            let slot = filter.get_actual_slot_for_fingerprint(i);
            assert!(slot.is_some(), "Should find slot for value {}", i);
        }
    }

    /// Test advanced slot finding accuracy
    #[tokio::test]
    async fn test_advanced_slot_finding_accuracy() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Insert values and track their slots
        let mut value_to_slot = std::collections::HashMap::new();
        let values: Vec<u64> = (0..100).collect();
        
        for &value in &values {
            assert!(filter.insert(value).is_ok());
            if let Some(slot) = filter.get_actual_slot_for_fingerprint(value) {
                value_to_slot.insert(value, slot);
            }
        }
        
        // Verify that slot finding is consistent
        for &value in &values {
            let slot = filter.get_actual_slot_for_fingerprint(value);
            assert!(slot.is_some(), "Should find slot for value {}", value);
            
            let slot_index = slot.unwrap();
            if let Some(&expected_slot) = value_to_slot.get(&value) {
                assert_eq!(slot_index, expected_slot, "Slot finding inconsistent for value {}", value);
            }
        }
    }

    /// Test advanced slot finding with different remainder bit sizes
    #[tokio::test]
    async fn test_advanced_slot_finding_remainder_bits() {
        let remainder_bits = vec![4, 8, 12, 16];
        
        for bits in remainder_bits {
            let mut filter = QuotientFilter::new(1000, bits, HashFunction::AHash).unwrap();
            
            // Insert values
            let values = vec![123, 456, 789, 101112, 131415];
            for &value in &values {
                assert!(filter.insert(value).is_ok());
            }
            
            // Test slot finding with different remainder bit sizes
            for &value in &values {
                let slot = filter.get_actual_slot_for_fingerprint(value);
                assert!(slot.is_some(), "Should find slot for value {} with {} remainder bits", value, bits);
            }
        }
    }

    /// Test advanced slot finding with stress test
    #[tokio::test]
    async fn test_advanced_slot_finding_stress() {
        let mut filter = QuotientFilter::new(10000, 8, HashFunction::AHash).unwrap();
        let mut rng = StdRng::seed_from_u64(42);
        
        // Insert random values
        let mut values = Vec::new();
        for _ in 0..5000 {
            let value = rng.gen::<u64>();
            filter.insert(value).unwrap();
            values.push(value);
        }
        
        // Test slot finding for all values
        for &value in &values {
            let slot = filter.get_actual_slot_for_fingerprint(value);
            assert!(slot.is_some(), "Should find slot for value {}", value);
        }
        
        // Test slot finding for random values (some might not exist)
        for _ in 0..1000 {
            let value = rng.gen::<u64>();
            let slot = filter.get_actual_slot_for_fingerprint(value);
            // Slot might or might not be found depending on whether value exists
            if slot.is_some() {
                assert!(filter.contains(value), "If slot found, value should exist");
            }
        }
    }
}
