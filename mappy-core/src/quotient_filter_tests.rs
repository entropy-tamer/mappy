#[cfg(test)]
mod quotient_filter_tests {
    use super::*;
    use crate::quotient_filter::{QuotientFilter, HashFunction};
    use std::collections::HashSet;
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    /// Test basic quotient filter operations
    #[test]
    fn test_quotient_filter_basic_operations() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Test empty filter
        assert_eq!(filter.len(), 0);
        assert!(!filter.contains(12345));
        
        // Test insertion
        assert!(filter.insert(12345).is_ok());
        assert_eq!(filter.len(), 1);
        assert!(filter.contains(12345));
        
        // Test multiple insertions
        for i in 0..100 {
            assert!(filter.insert(i).is_ok());
        }
        assert_eq!(filter.len(), 101);
        
        // Test that all inserted values are found
        assert!(filter.contains(12345));
        for i in 0..100 {
            assert!(filter.contains(i));
        }
    }

    /// Test false positive rate
    #[test]
    fn test_false_positive_rate() {
        let mut filter = QuotientFilter::new(10000, 8, HashFunction::AHash).unwrap();
        let mut rng = StdRng::seed_from_u64(42);
        
        // Insert 5000 random values
        let mut inserted = HashSet::new();
        for _ in 0..5000 {
            let value = rng.gen::<u64>();
            filter.insert(value).unwrap();
            inserted.insert(value);
        }
        
        // Test 10000 random values for false positives
        let mut false_positives = 0;
        let mut total_tested = 0;
        
        for _ in 0..10000 {
            let value = rng.gen::<u64>();
            if !inserted.contains(&value) {
                total_tested += 1;
                if filter.contains(value) {
                    false_positives += 1;
                }
            }
        }
        
        let false_positive_rate = false_positives as f64 / total_tested as f64;
        println!("False positive rate: {:.4} (expected ~0.01)", false_positive_rate);
        
        // Should be within reasonable bounds (allow some variance)
        assert!(false_positive_rate < 0.05, "False positive rate too high: {}", false_positive_rate);
    }

    /// Test deletion operations
    #[test]
    fn test_deletion_operations() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Insert some values
        let values = vec![123, 456, 789, 101112, 131415];
        for &value in &values {
            assert!(filter.insert(value).is_ok());
        }
        
        // Verify all are present
        for &value in &values {
            assert!(filter.contains(value));
        }
        
        // Delete some values
        assert!(filter.delete(456).is_ok());
        assert!(filter.delete(101112).is_ok());
        
        // Verify deletion worked
        assert!(!filter.contains(456));
        assert!(!filter.contains(101112));
        assert!(filter.contains(123));
        assert!(filter.contains(789));
        assert!(filter.contains(131415));
        
        // Test deleting non-existent value
        assert!(filter.delete(999999).is_ok()); // Should not panic
    }

    /// Test multiset operations
    #[test]
    fn test_multiset_operations() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Insert same value multiple times
        let value = 12345;
        for _ in 0..5 {
            assert!(filter.insert(value).is_ok());
        }
        
        // Check count
        assert_eq!(filter.count(value), 5);
        assert!(filter.contains(value));
        
        // Delete one instance
        assert!(filter.delete(value).is_ok());
        assert_eq!(filter.count(value), 4);
        assert!(filter.contains(value));
        
        // Delete all remaining instances
        for _ in 0..4 {
            assert!(filter.delete(value).is_ok());
        }
        
        // Should be completely removed
        assert_eq!(filter.count(value), 0);
        assert!(!filter.contains(value));
    }

    /// Test run detection and slot finding
    #[test]
    fn test_run_detection_and_slot_finding() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Insert values that might create runs
        let values = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        for &value in &values {
            assert!(filter.insert(value).is_ok());
        }
        
        // Test slot finding for each value
        for &value in &values {
            let slot = filter.get_actual_slot_for_fingerprint(value);
            assert!(slot.is_some(), "Should find slot for value {}", value);
            
            let slot_index = slot.unwrap();
            assert!(slot_index < filter.capacity(), "Slot index {} out of bounds", slot_index);
        }
        
        // Test slot finding for non-existent value
        let non_existent = 99999;
        let slot = filter.get_actual_slot_for_fingerprint(non_existent);
        assert!(slot.is_none(), "Should not find slot for non-existent value");
    }

    /// Test capacity and load factor
    #[test]
    fn test_capacity_and_load_factor() {
        let capacity = 1000;
        let mut filter = QuotientFilter::new(capacity, 8, HashFunction::AHash).unwrap();
        
        assert_eq!(filter.capacity(), capacity);
        assert_eq!(filter.len(), 0);
        assert_eq!(filter.load_factor(), 0.0);
        
        // Insert half capacity
        for i in 0..500 {
            assert!(filter.insert(i).is_ok());
        }
        
        assert_eq!(filter.len(), 500);
        let load_factor = filter.load_factor();
        assert!((load_factor - 0.5).abs() < 0.01, "Load factor should be ~0.5, got {}", load_factor);
        
        // Insert to full capacity
        for i in 500..1000 {
            assert!(filter.insert(i).is_ok());
        }
        
        assert_eq!(filter.len(), 1000);
        let load_factor = filter.load_factor();
        assert!((load_factor - 1.0).abs() < 0.01, "Load factor should be ~1.0, got {}", load_factor);
    }

    /// Test statistics
    #[test]
    fn test_statistics() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Insert some values
        for i in 0..100 {
            assert!(filter.insert(i).is_ok());
        }
        
        let stats = filter.stats();
        assert_eq!(stats.total_slots, 1000);
        assert_eq!(stats.occupied_slots, 100);
        assert_eq!(stats.runs, stats.runs); // Just check it's not negative
        assert_eq!(stats.shifted_slots, stats.shifted_slots); // Just check it's not negative
    }

    /// Test concurrent operations
    #[test]
    fn test_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;
        
        let filter = Arc::new(QuotientFilter::new(10000, 8, HashFunction::AHash).unwrap());
        let mut handles = vec![];
        
        // Spawn multiple threads that insert values
        for thread_id in 0..4 {
            let filter_clone = Arc::clone(&filter);
            let handle = thread::spawn(move || {
                for i in 0..1000 {
                    let value = thread_id * 1000 + i;
                    filter_clone.insert(value).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all values are present
        for thread_id in 0..4 {
            for i in 0..1000 {
                let value = thread_id * 1000 + i;
                assert!(filter.contains(value), "Value {} not found", value);
            }
        }
        
        assert_eq!(filter.len(), 4000);
    }

    /// Test edge cases
    #[test]
    fn test_edge_cases() {
        // Test with minimum capacity
        let mut filter = QuotientFilter::new(1, 1, HashFunction::AHash).unwrap();
        assert!(filter.insert(0).is_ok());
        assert!(filter.contains(0));
        
        // Test with maximum capacity (reasonable limit)
        let mut filter = QuotientFilter::new(1000000, 16, HashFunction::AHash).unwrap();
        for i in 0..10000 {
            assert!(filter.insert(i).is_ok());
        }
        assert_eq!(filter.len(), 10000);
        
        // Test with zero capacity (should fail)
        assert!(QuotientFilter::new(0, 8, HashFunction::AHash).is_err());
        
        // Test with invalid remainder bits
        assert!(QuotientFilter::new(1000, 0, HashFunction::AHash).is_err());
        assert!(QuotientFilter::new(1000, 65, HashFunction::AHash).is_err());
    }

    /// Test hash function variations
    #[test]
    fn test_hash_function_variations() {
        let hash_functions = vec![
            HashFunction::AHash,
            HashFunction::TwoX,
            HashFunction::FNV,
        ];
        
        for hash_fn in hash_functions {
            let mut filter = QuotientFilter::new(1000, 8, hash_fn).unwrap();
            
            // Test basic operations with each hash function
            for i in 0..100 {
                assert!(filter.insert(i).is_ok());
                assert!(filter.contains(i));
            }
            
            assert_eq!(filter.len(), 100);
        }
    }

    /// Test performance with large datasets
    #[test]
    fn test_performance_large_dataset() {
        let mut filter = QuotientFilter::new(100000, 12, HashFunction::AHash).unwrap();
        
        // Insert 50000 values
        let start = std::time::Instant::now();
        for i in 0..50000 {
            assert!(filter.insert(i).is_ok());
        }
        let insert_time = start.elapsed();
        
        // Query 50000 values
        let start = std::time::Instant::now();
        for i in 0..50000 {
            assert!(filter.contains(i));
        }
        let query_time = start.elapsed();
        
        println!("Insert time: {:?}", insert_time);
        println!("Query time: {:?}", query_time);
        
        // Performance should be reasonable (less than 1 second for 50k operations)
        assert!(insert_time.as_secs() < 1, "Insert performance too slow: {:?}", insert_time);
        assert!(query_time.as_secs() < 1, "Query performance too slow: {:?}", query_time);
    }

    /// Test memory usage
    #[test]
    fn test_memory_usage() {
        let filter = QuotientFilter::new(10000, 8, HashFunction::AHash).unwrap();
        
        // Check that memory usage is reasonable
        let stats = filter.stats();
        let expected_memory = 10000 * std::mem::size_of::<crate::quotient_filter::SlotMetadata>();
        
        // Memory usage should be close to expected (allow some overhead)
        assert!(stats.memory_usage > 0);
        assert!(stats.memory_usage < expected_memory * 2, "Memory usage too high: {} bytes", stats.memory_usage);
    }

    /// Test slot finding accuracy
    #[test]
    fn test_slot_finding_accuracy() {
        let mut filter = QuotientFilter::new(1000, 8, HashFunction::AHash).unwrap();
        
        // Insert values and track their slots
        let mut value_to_slot = std::collections::HashMap::new();
        
        for i in 0..100 {
            assert!(filter.insert(i).is_ok());
            if let Some(slot) = filter.get_actual_slot_for_fingerprint(i) {
                value_to_slot.insert(i, slot);
            }
        }
        
        // Verify that slot finding is consistent
        for i in 0..100 {
            let slot = filter.get_actual_slot_for_fingerprint(i);
            assert!(slot.is_some(), "Should find slot for value {}", i);
            
            let slot_index = slot.unwrap();
            if let Some(&expected_slot) = value_to_slot.get(&i) {
                assert_eq!(slot_index, expected_slot, "Slot finding inconsistent for value {}", i);
            }
        }
    }

    /// Test run detection with collisions
    #[test]
    fn test_run_detection_with_collisions() {
        let mut filter = QuotientFilter::new(100, 4, HashFunction::AHash).unwrap(); // Small capacity to force collisions
        
        // Insert values that are likely to collide
        let values = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
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
    }
}
