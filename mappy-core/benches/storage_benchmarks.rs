//! Storage backend performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mappy_core::{Engine, EngineConfig, PersistenceMode};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Generate random test data
fn generate_test_data(size: usize) -> Vec<(String, Vec<u8>)> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..size)
        .map(|i| {
            let key = format!("key_{:08x}", i);
            let value_size = rng.gen_range(10..=1000);
            let value = (0..value_size).map(|_| rng.gen()).collect();
            (key, value)
        })
        .collect()
}

/// Benchmark storage backend write performance
fn bench_storage_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("storage_write");
    
    for size in [1000, 10000, 100000].iter() {
        let test_data = generate_test_data(*size);
        
        // Memory storage
        group.bench_with_input(BenchmarkId::new("Memory", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::Memory,
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                black_box(engine)
            })
        });
        
        // Disk storage
        group.bench_with_input(BenchmarkId::new("Disk", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = std::env::temp_dir().join("mappy_bench_disk");
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::Disk,
                    data_dir: Some(temp_dir.to_string_lossy().to_string()),
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                black_box(engine)
            })
        });
        
        // AOF storage
        group.bench_with_input(BenchmarkId::new("AOF", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = std::env::temp_dir().join("mappy_bench_aof");
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::AOF,
                    data_dir: Some(temp_dir.to_string_lossy().to_string()),
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                black_box(engine)
            })
        });
        
        // Hybrid storage
        group.bench_with_input(BenchmarkId::new("Hybrid", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = std::env::temp_dir().join("mappy_bench_hybrid");
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::Hybrid,
                    data_dir: Some(temp_dir.to_string_lossy().to_string()),
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                black_box(engine)
            })
        });
    }
    
    group.finish();
}

/// Benchmark storage backend read performance
fn bench_storage_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("storage_read");
    
    for size in [1000, 10000, 100000].iter() {
        let test_data = generate_test_data(*size);
        let read_keys: Vec<String> = test_data.iter().map(|(k, _)| k.clone()).collect();
        
        // Memory storage
        group.bench_with_input(BenchmarkId::new("Memory", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::Memory,
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                // Pre-populate
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                // Benchmark reads
                for key in &read_keys {
                    black_box(engine.get(key).await.unwrap());
                }
                
                black_box(engine)
            })
        });
        
        // Disk storage
        group.bench_with_input(BenchmarkId::new("Disk", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = std::env::temp_dir().join("mappy_bench_disk_read");
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::Disk,
                    data_dir: Some(temp_dir.to_string_lossy().to_string()),
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                // Pre-populate
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                // Benchmark reads
                for key in &read_keys {
                    black_box(engine.get(key).await.unwrap());
                }
                
                black_box(engine)
            })
        });
        
        // AOF storage
        group.bench_with_input(BenchmarkId::new("AOF", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = std::env::temp_dir().join("mappy_bench_aof_read");
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::AOF,
                    data_dir: Some(temp_dir.to_string_lossy().to_string()),
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                // Pre-populate
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                // Benchmark reads
                for key in &read_keys {
                    black_box(engine.get(key).await.unwrap());
                }
                
                black_box(engine)
            })
        });
        
        // Hybrid storage
        group.bench_with_input(BenchmarkId::new("Hybrid", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = std::env::temp_dir().join("mappy_bench_hybrid_read");
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::Hybrid,
                    data_dir: Some(temp_dir.to_string_lossy().to_string()),
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                // Pre-populate
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                // Benchmark reads
                for key in &read_keys {
                    black_box(engine.get(key).await.unwrap());
                }
                
                black_box(engine)
            })
        });
    }
    
    group.finish();
}

/// Benchmark TTL operations
fn bench_ttl_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ttl_operations");
    
    for size in [1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        
        group.bench_with_input(BenchmarkId::new("Set_TTL", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let config = EngineConfig::default();
                let engine = Engine::new(config).await.unwrap();
                
                // Pre-populate
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                // Benchmark TTL setting
                for (key, _) in &test_data {
                    engine.expire(key, 3600).await.unwrap();
                }
                
                black_box(engine)
            })
        });
        
        group.bench_with_input(BenchmarkId::new("Check_TTL", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let config = EngineConfig::default();
                let engine = Engine::new(config).await.unwrap();
                
                // Pre-populate with TTL
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                    engine.expire(key, 3600).await.unwrap();
                }
                
                // Benchmark TTL checking
                for (key, _) in &test_data {
                    black_box(engine.ttl(key).await.unwrap());
                }
                
                black_box(engine)
            })
        });
    }
    
    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_operations");
    
    for size in [1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        
        group.bench_with_input(BenchmarkId::new("Concurrent_Write", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let config = EngineConfig::default();
                let engine = Engine::new(config).await.unwrap();
                
                let handles: Vec<_> = test_data.chunks(100).map(|chunk| {
                    let engine = engine.clone();
                    let chunk = chunk.to_vec();
                    tokio::spawn(async move {
                        for (key, value) in chunk {
                            engine.set(key, value).await.unwrap();
                        }
                    })
                }).collect();
                
                for handle in handles {
                    handle.await.unwrap();
                }
                
                black_box(engine)
            })
        });
        
        group.bench_with_input(BenchmarkId::new("Concurrent_Read", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let config = EngineConfig::default();
                let engine = Engine::new(config).await.unwrap();
                
                // Pre-populate
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                let read_keys: Vec<String> = test_data.iter().map(|(k, _)| k.clone()).collect();
                let handles: Vec<_> = read_keys.chunks(100).map(|chunk| {
                    let engine = engine.clone();
                    let chunk = chunk.to_vec();
                    tokio::spawn(async move {
                        for key in chunk {
                            black_box(engine.get(&key).await.unwrap());
                        }
                    })
                }).collect();
                
                for handle in handles {
                    handle.await.unwrap();
                }
                
                black_box(engine)
            })
        });
    }
    
    group.finish();
}

/// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_usage");
    
    for size in [1000, 10000, 100000].iter() {
        let test_data = generate_test_data(*size);
        
        group.bench_with_input(BenchmarkId::new("Memory_Storage", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::Memory,
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                let memory_usage = engine.memory_usage().await.unwrap();
                black_box(memory_usage)
            })
        });
        
        group.bench_with_input(BenchmarkId::new("Hybrid_Storage", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = std::env::temp_dir().join("mappy_bench_memory");
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let config = EngineConfig {
                    persistence_mode: PersistenceMode::Hybrid,
                    data_dir: Some(temp_dir.to_string_lossy().to_string()),
                    ..Default::default()
                };
                let engine = Engine::new(config).await.unwrap();
                
                for (key, value) in &test_data {
                    engine.set(key.clone(), value.clone()).await.unwrap();
                }
                
                let memory_usage = engine.memory_usage().await.unwrap();
                black_box(memory_usage)
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_storage_write,
    bench_storage_read,
    bench_ttl_operations,
    bench_concurrent_operations,
    bench_memory_usage
);
criterion_main!(benches);



