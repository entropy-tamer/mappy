//! Benchmarks comparing maplet performance with standard data structures

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use mappy_core::{Maplet, CounterOperator, SetOperator};
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use tokio::runtime::Runtime;

/// Generate random test data
fn generate_test_data(size: usize) -> Vec<(String, u64)> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..size)
        .map(|i| (format!("key_{}", i), rng.gen_range(1..=1000)))
        .collect()
}

/// Benchmark insert operations
fn bench_insert(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("insert_operations");
    
    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        
        // Benchmark HashMap
        group.bench_with_input(BenchmarkId::new("HashMap", size), size, |b, _| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (key, value) in &test_data {
                    map.insert(key.clone(), *value);
                }
                black_box(map)
            })
        });
        
        // Benchmark BTreeMap
        group.bench_with_input(BenchmarkId::new("BTreeMap", size), size, |b, _| {
            b.iter(|| {
                let mut map = BTreeMap::new();
                for (key, value) in &test_data {
                    map.insert(key.clone(), *value);
                }
                black_box(map)
            })
        });
        
        // Benchmark Maplet (Counter)
        group.bench_with_input(BenchmarkId::new("Maplet-Counter", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let maplet = Maplet::<String, u64, CounterOperator>::new(*size * 8, 0.01).unwrap();
                    for (key, value) in &test_data {
                        maplet.insert(key.clone(), *value).await.unwrap();
                    }
                    black_box(maplet)
                })
            })
        });
        
        // Benchmark Maplet (Set) - using HashSet as value type
        group.bench_with_input(BenchmarkId::new("Maplet-Set", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let maplet = Maplet::<String, std::collections::HashSet<u64>, SetOperator>::new(*size * 8, 0.01).unwrap();
                    for (key, value) in &test_data {
                        let mut set = std::collections::HashSet::new();
                        set.insert(*value);
                        maplet.insert(key.clone(), set).await.unwrap();
                    }
                    black_box(maplet)
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark query operations
fn bench_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("query_operations");
    
    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        let query_keys: Vec<String> = test_data.iter().map(|(k, _)| k.clone()).collect();
        
        // Prepare HashMap
        let mut hashmap = HashMap::new();
        for (key, value) in &test_data {
            hashmap.insert(key.clone(), *value);
        }
        
        // Prepare BTreeMap
        let mut btreemap = BTreeMap::new();
        for (key, value) in &test_data {
            btreemap.insert(key.clone(), *value);
        }
        
        // Prepare Maplet
        let maplet = rt.block_on(async {
            let maplet = Maplet::<String, u64, CounterOperator>::new(*size, 0.01).unwrap();
            for (key, value) in &test_data {
                maplet.insert(key.clone(), *value).await.unwrap();
            }
            maplet
        });
        
        // Benchmark HashMap queries
        group.bench_with_input(BenchmarkId::new("HashMap", size), size, |b, _| {
            b.iter(|| {
                for key in &query_keys {
                    black_box(hashmap.get(key));
                }
            })
        });
        
        // Benchmark BTreeMap queries
        group.bench_with_input(BenchmarkId::new("BTreeMap", size), size, |b, _| {
            b.iter(|| {
                for key in &query_keys {
                    black_box(btreemap.get(key));
                }
            })
        });
        
        // Benchmark Maplet queries
        group.bench_with_input(BenchmarkId::new("Maplet", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    for key in &query_keys {
                        black_box(maplet.query(key).await);
                    }
                })
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
        
        // Benchmark HashMap memory
        group.bench_with_input(BenchmarkId::new("HashMap", size), size, |b, _| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (key, value) in &test_data {
                    map.insert(key.clone(), *value);
                }
                // Force allocation
                black_box(map.capacity());
            })
        });
        
        // Benchmark Maplet memory
        group.bench_with_input(BenchmarkId::new("Maplet", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let maplet = Maplet::<String, u64, CounterOperator>::new(*size * 8, 0.01).unwrap();
                    for (key, value) in &test_data {
                        maplet.insert(key.clone(), *value).await.unwrap();
                    }
                    // Get stats to measure memory
                    let stats = maplet.stats().await;
                    black_box(stats);
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark merge operations
fn bench_merge_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("merge_operations");
    
    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        
        // Benchmark HashMap merge (manual)
        group.bench_with_input(BenchmarkId::new("HashMap-Merge", size), size, |b, _| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (key, value) in &test_data {
                    *map.entry(key.clone()).or_insert(0) += *value;
                }
                black_box(map)
            })
        });
        
        // Benchmark Maplet merge (automatic)
        group.bench_with_input(BenchmarkId::new("Maplet-Merge", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let maplet = Maplet::<String, u64, CounterOperator>::new(*size * 8, 0.01).unwrap();
                    for (key, value) in &test_data {
                        maplet.insert(key.clone(), *value).await.unwrap();
                    }
                    black_box(maplet)
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_operations");
    
    for size in [1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        
        // Benchmark concurrent HashMap (with RwLock)
        group.bench_with_input(BenchmarkId::new("HashMap-Concurrent", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    use tokio::sync::RwLock;
                    let map = Arc::new(RwLock::new(HashMap::new()));
                    
                    let handles: Vec<_> = test_data.chunks(100).map(|chunk| {
                        let map = Arc::clone(&map);
                        let chunk = chunk.to_vec();
                        tokio::spawn(async move {
                            let mut map = map.write().await;
                            for (key, value) in chunk {
                                map.insert(key, value);
                            }
                        })
                    }).collect();
                    
                    for handle in handles {
                        handle.await.unwrap();
                    }
                    
                    black_box(map)
                })
            })
        });
        
        // Benchmark concurrent Maplet
        group.bench_with_input(BenchmarkId::new("Maplet-Concurrent", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let maplet = Arc::new(Maplet::<String, u64, CounterOperator>::new(*size, 0.01).unwrap());
                    
                    let handles: Vec<_> = test_data.chunks(100).map(|chunk| {
                        let maplet = Arc::clone(&maplet);
                        let chunk = chunk.to_vec();
                        tokio::spawn(async move {
                            for (key, value) in chunk {
                                maplet.insert(key, value).await.unwrap();
                            }
                        })
                    }).collect();
                    
                    for handle in handles {
                        handle.await.unwrap();
                    }
                    
                    black_box(maplet)
                })
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_insert,
    bench_query,
    bench_memory_usage,
    bench_merge_operations,
    bench_concurrent
);
criterion_main!(benches);
