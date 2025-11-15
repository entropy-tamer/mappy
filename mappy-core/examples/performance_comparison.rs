#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Performance comparison between Mappy and standard data structures
//!
//! This example demonstrates the space efficiency and performance characteristics
//! of Mappy compared to standard Rust collections.

use mappy_core::{Engine, EngineConfig, PersistenceMode};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Mappy Performance Comparison");
    println!("===============================\n");

    // Test different data sizes
    let sizes = vec![1000, 10000, 100000];

    for size in sizes {
        println!("📊 Testing with {} items", size);
        println!("{}", "=".repeat(50));

        // Generate test data
        let test_data = generate_test_data(size);

        // Test HashMap
        test_hashmap(&test_data, size).await;

        // Test BTreeMap
        test_btreemap(&test_data, size).await;

        // Test Mappy (Memory)
        test_mappy(&test_data, size, PersistenceMode::Memory).await;

        // Test Mappy (Disk)
        test_mappy(&test_data, size, PersistenceMode::Disk).await;

        println!();
    }

    // Test memory efficiency
    test_memory_efficiency().await?;

    // Test concurrent performance
    test_concurrent_performance().await?;

    println!("✅ Performance comparison completed!");
    Ok(())
}

/// Generate random test data
fn generate_test_data(size: usize) -> Vec<(String, String)> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..size)
        .map(|i| {
            let key = format!("key_{:08x}", i);
            let value = format!("value_{}", rng.gen_range(1..=1000));
            (key, value)
        })
        .collect()
}

/// Test HashMap performance
async fn test_hashmap(test_data: &[(String, String)], size: usize) {
    let start = Instant::now();

    let mut map = HashMap::new();
    for (key, value) in test_data {
        map.insert(key.clone(), value.clone());
    }

    let insert_time = start.elapsed();

    // Test reads
    let start = Instant::now();
    for (key, _) in test_data.iter().take(1000) {
        let _ = map.get(key);
    }
    let read_time = start.elapsed();

    // Estimate memory usage (rough approximation)
    let estimated_memory = size * (32 + 32); // Key + value overhead

    println!("  HashMap:");
    println!("    Insert time: {:?}", insert_time);
    println!("    Read time (1000 items): {:?}", read_time);
    println!("    Estimated memory: {} bytes", estimated_memory);
    println!(
        "    Load factor: {:.2}",
        map.len() as f64 / map.capacity() as f64
    );
}

/// Test BTreeMap performance
async fn test_btreemap(test_data: &[(String, String)], size: usize) {
    let start = Instant::now();

    let mut map = BTreeMap::new();
    for (key, value) in test_data {
        map.insert(key.clone(), value.clone());
    }

    let insert_time = start.elapsed();

    // Test reads
    let start = Instant::now();
    for (key, _) in test_data.iter().take(1000) {
        let _ = map.get(key);
    }
    let read_time = start.elapsed();

    // Estimate memory usage (rough approximation)
    let estimated_memory = size * (32 + 32); // Key + value overhead

    println!("  BTreeMap:");
    println!("    Insert time: {:?}", insert_time);
    println!("    Read time (1000 items): {:?}", read_time);
    println!("    Estimated memory: {} bytes", estimated_memory);
}

/// Test Mappy performance
async fn test_mappy(
    test_data: &[(String, String)],
    _size: usize,
    persistence_mode: PersistenceMode,
) {
    let config = EngineConfig {
        persistence_mode,
        data_dir: if persistence_mode == PersistenceMode::Memory {
            None
        } else {
            Some(
                std::env::temp_dir()
                    .join("mappy_perf")
                    .to_string_lossy()
                    .to_string(),
            )
        },
        ..Default::default()
    };

    let engine = Engine::new(config).await.unwrap();

    let start = Instant::now();

    for (key, value) in test_data {
        engine
            .set(key.clone(), value.as_bytes().to_vec())
            .await
            .unwrap();
    }

    let insert_time = start.elapsed();

    // Test reads
    let start = Instant::now();
    for (key, _) in test_data.iter().take(1000) {
        let _ = engine.get(key).await.unwrap();
    }
    let read_time = start.elapsed();

    // Get actual memory usage
    let stats = engine.stats().await.unwrap();
    let memory_usage = engine.memory_usage().await.unwrap();

    println!("  Mappy ({:?}):", persistence_mode);
    println!("    Insert time: {:?}", insert_time);
    println!("    Read time (1000 items): {:?}", read_time);
    println!("    Memory usage: {} bytes", memory_usage);
    println!("    Maplet stats: {:?}", stats.maplet_stats);

    engine.close().await.unwrap();
}

/// Test memory efficiency with different false positive rates
async fn test_memory_efficiency() -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Memory Efficiency Test");
    println!("{}", "=".repeat(50));

    let test_data = generate_test_data(10000);
    let false_positive_rates = vec![0.001, 0.01, 0.1, 0.5];

    for fpr in false_positive_rates {
        let config = EngineConfig {
            persistence_mode: PersistenceMode::Memory,
            maplet: mappy_core::types::MapletConfig {
                capacity: 10000,
                false_positive_rate: fpr,
                max_load_factor: 0.8,
                auto_resize: false,
                enable_deletion: true,
                enable_merging: true,
            },
            ..Default::default()
        };

        let engine = Engine::new(config).await?;

        for (key, value) in &test_data {
            engine.set(key.clone(), value.as_bytes().to_vec()).await?;
        }

        let memory_usage = engine.memory_usage().await?;
        let stats = engine.stats().await?;

        println!("  False Positive Rate: {:.1}%", fpr * 100.0);
        println!("    Memory usage: {} bytes", memory_usage);
        println!("    Maplet stats: {:?}", stats.maplet_stats);

        engine.close().await?;
    }

    Ok(())
}

/// Test concurrent performance
async fn test_concurrent_performance() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Concurrent Performance Test");
    println!("{}", "=".repeat(50));

    let config = EngineConfig::default();
    let engine = Engine::new(config).await?;

    // Test concurrent writes
    let start = Instant::now();
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let engine = engine.clone();
            tokio::spawn(async move {
                for j in 0..1000 {
                    let key = format!("concurrent_{}_{}", i, j);
                    let value = format!("value_{}_{}", i, j);
                    engine.set(key, value.into_bytes()).await.unwrap();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    let concurrent_write_time = start.elapsed();

    // Test concurrent reads
    let start = Instant::now();
    let read_handles: Vec<_> = (0..5)
        .map(|i| {
            let engine = engine.clone();
            tokio::spawn(async move {
                for j in 0..2000 {
                    let key = format!("concurrent_{}_{}", i % 10, j % 1000);
                    let _ = engine.get(&key).await.unwrap();
                }
            })
        })
        .collect();

    for handle in read_handles {
        handle.await?;
    }

    let concurrent_read_time = start.elapsed();

    println!(
        "  Concurrent writes (10 tasks, 1000 items each): {:?}",
        concurrent_write_time
    );
    println!(
        "  Concurrent reads (5 tasks, 2000 items each): {:?}",
        concurrent_read_time
    );

    let stats = engine.stats().await?;
    println!("  Final stats: {:?}", stats);

    engine.close().await?;
    Ok(())
}

// Helper trait to clone Engine
trait EngineClone {
    fn clone(&self) -> Self;
}

impl EngineClone for Engine {
    fn clone(&self) -> Self {
        // This is a simplified clone for demo purposes
        // In a real implementation, you'd need proper cloning
        unimplemented!("Engine cloning not implemented in this demo")
    }
}
