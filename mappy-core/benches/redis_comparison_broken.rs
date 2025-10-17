//! Benchmarks comparing Mappy performance with Redis
//! 
//! This benchmark suite compares Mappy's performance against Redis for various
//! operations including insertions, queries, deletions, and concurrent access patterns.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Runtime;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

// Redis client
use redis::AsyncCommands;
use redis::Client as RedisClient;

// Mappy imports
use mappy_core::{Maplet, CounterOperator, StringOperator};

/// Redis connection manager
#[derive(Clone)]
struct RedisBenchmark {
    client: RedisClient,
}

impl RedisBenchmark {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = RedisClient::open("redis://127.0.0.1:6379")?;
        
        Ok(Self {
            client,
        })
    }
    
    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, Box<dyn std::error::Error>> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }
    
    async fn flush_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut connection = self.get_connection().await?;
        let _: () = connection.flushall().await?;
        Ok(())
    }
    
    async fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut connection = self.get_connection().await?;
        let _: () = connection.set(key, value).await?;
        Ok(())
    }
    
    async fn get(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut connection = self.get_connection().await?;
        let result: Option<String> = connection.get(key).await?;
        Ok(result)
    }
    
    async fn del(&self, key: &str) -> Result<u32, Box<dyn std::error::Error>> {
        let mut connection = self.get_connection().await?;
        let result: u32 = connection.del(key).await?;
        Ok(result)
    }
    
    async fn incr(&self, key: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let mut connection = self.get_connection().await?;
        let result: i64 = connection.incr(key, 1).await?;
        Ok(result)
    }
    
    async fn exists(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut connection = self.get_connection().await?;
        let result: bool = connection.exists(key).await?;
        Ok(result)
    }
    
    async fn info(&self, section: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut connection = self.get_connection().await?;
        let result: String = connection.info(section).await?;
        Ok(result)
    }
}

/// Generate random test data
fn generate_test_data(size: usize) -> Vec<(String, String)> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..size)
        .map(|i| {
            let key = format!("key_{}", i);
            let value = format!("value_{}", rng.gen_range(1..=1000));
            (key, value)
        })
        .collect()
}

/// Generate random test data for counters
fn generate_counter_data(size: usize) -> Vec<(String, u64)> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..size)
        .map(|i| {
            let key = format!("counter_{}", i);
            let value = rng.gen_range(1..=1000);
            (key, value)
        })
        .collect()
}

/// Benchmark basic set/get operations
fn bench_basic_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("basic_operations");
    group.measurement_time(Duration::from_secs(30));
    
    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        
        // Benchmark Redis SET operations
        group.bench_with_input(BenchmarkId::new("Redis-SET", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let redis = RedisBenchmark::new().await.unwrap();
                    redis.flush_all().await.unwrap();
                    
                    for (key, value) in &test_data {
                        redis.set(key, value).await.unwrap();
                    }
                    
                    black_box(redis)
                })
            })
        });
        
        // Benchmark Mappy insert operations
        group.bench_with_input(BenchmarkId::new("Mappy-INSERT", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let maplet = Maplet::<String, String, StringOperator>::new(*size * 8, 0.01).unwrap();
                    
                    for (key, value) in &test_data {
                        maplet.insert(key.clone(), value.clone()).await.unwrap();
                    }
                    
                    black_box(maplet)
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark GET operations
fn bench_get_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("get_operations");
    group.measurement_time(Duration::from_secs(30));
    
    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        let query_keys: Vec<String> = test_data.iter().map(|(k, _)| k.clone()).collect();
        
        // Prepare Redis data
        let redis = rt.block_on(async {
            let redis = RedisBenchmark::new().await.unwrap();
            redis.flush_all().await.unwrap();
            for (key, value) in &test_data {
                redis.set(key, value).await.unwrap();
            }
            redis
        });
        
        // Prepare Mappy data
        let maplet = rt.block_on(async {
            let maplet = Maplet::<String, String, StringOperator>::new(*size * 8, 0.01).unwrap();
            for (key, value) in &test_data {
                maplet.insert(key.clone(), value.clone()).await.unwrap();
            }
            maplet
        });
        
        // Benchmark Redis GET operations
        group.bench_with_input(BenchmarkId::new("Redis-GET", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    for key in &query_keys {
                        black_box(redis.get(key).await.unwrap());
                    }
                })
                })
            })
        });
        
        // Benchmark Mappy query operations
        group.bench_with_input(BenchmarkId::new("Mappy-QUERY", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    for key in &query_keys {
                        black_box(maplet.query(key).await);
                    }
                })
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark counter operations (INCR vs Maplet Counter)
fn bench_counter_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("counter_operations");
    group.measurement_time(Duration::from_secs(30));
    
    for size in [100, 1000, 10000].iter() {
        let test_data = generate_counter_data(*size);
        
        // Benchmark Redis INCR operations
        group.bench_with_input(BenchmarkId::new("Redis-INCR", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let redis = RedisBenchmark::new().await.unwrap();
                redis.flush_all().await.unwrap();
                
                for (key, value) in &test_data {
                    // Set initial value
                    redis.set(key, &value.to_string()).await.unwrap();
                    // Increment
                    redis.incr(key).await.unwrap();
                }
                
                black_box(redis)
                })
            })
        });
        
        // Benchmark Mappy Counter operations
        group.bench_with_input(BenchmarkId::new("Mappy-COUNTER", size), size, |b, _| {
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

/// Benchmark deletion operations
fn bench_delete_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("delete_operations");
    group.measurement_time(Duration::from_secs(30));
    
    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        let delete_keys: Vec<String> = test_data.iter().map(|(k, _)| k.clone()).collect();
        
        // Benchmark Redis DEL operations
        group.bench_with_input(BenchmarkId::new("Redis-DEL", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let redis = RedisBenchmark::new().await.unwrap();
                redis.flush_all().await.unwrap();
                
                // Insert data
                for (key, value) in &test_data {
                    redis.set(key, value).await.unwrap();
                }
                
                // Delete data
                for key in &delete_keys {
                    redis.del(key).await.unwrap();
                }
                
                black_box(redis)
                })
            })
        });
        
        // Benchmark Mappy delete operations
        group.bench_with_input(BenchmarkId::new("Mappy-DELETE", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let maplet = Maplet::<String, String, StringOperator>::new(*size * 8, 0.01).unwrap();
                
                // Insert data
                for (key, value) in &test_data {
                    maplet.insert(key.clone(), value.clone()).await.unwrap();
                }
                
                // Delete data
                for key in &delete_keys {
                    maplet.delete(key).await.unwrap();
                }
                
                black_box(maplet)
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark existence checks
fn bench_exists_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("exists_operations");
    group.measurement_time(Duration::from_secs(30));
    
    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        let check_keys: Vec<String> = test_data.iter().map(|(k, _)| k.clone()).collect();
        
        // Prepare Redis data
        let redis = rt.block_on(async {
            let redis = RedisBenchmark::new().await.unwrap();
            redis.flush_all().await.unwrap();
            for (key, value) in &test_data {
                redis.set(key, value).await.unwrap();
            }
            redis
        });
        
        // Prepare Mappy data
        let maplet = rt.block_on(async {
            let maplet = Maplet::<String, String, StringOperator>::new(*size * 8, 0.01).unwrap();
            for (key, value) in &test_data {
                maplet.insert(key.clone(), value.clone()).await.unwrap();
            }
            maplet
        });
        
        // Benchmark Redis EXISTS operations
        group.bench_with_input(BenchmarkId::new("Redis-EXISTS", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                for key in &check_keys {
                    black_box(redis.exists(key).await.unwrap());
                }
                })
            })
        });
        
        // Benchmark Mappy contains operations
        group.bench_with_input(BenchmarkId::new("Mappy-CONTAINS", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                for key in &check_keys {
                    black_box(maplet.contains(key).await);
                }
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(30));
    
    for size in [1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        
        // Benchmark Redis concurrent operations
        group.bench_with_input(BenchmarkId::new("Redis-CONCURRENT", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let redis = RedisBenchmark::new().await.unwrap();
                redis.flush_all().await.unwrap();
                
                // Simulate concurrent operations with multiple tasks
                let handles: Vec<_> = test_data.chunks(100).map(|chunk| {
                    let mut redis = redis.clone();
                    let chunk = chunk.to_vec();
                    tokio::spawn(async move {
                        for (key, value) in chunk {
                            redis.set(&key, &value).await.unwrap();
                        }
                    })
                }).collect();
                
                for handle in handles {
                    handle.await.unwrap();
                }
                
                black_box(redis)
                })
            })
        });
        
        // Benchmark Mappy concurrent operations
        group.bench_with_input(BenchmarkId::new("Mappy-CONCURRENT", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let maplet = std::sync::Arc::new(Maplet::<String, String, StringOperator>::new(*size * 8, 0.01).unwrap());
                
                // Simulate concurrent operations with multiple tasks
                let handles: Vec<_> = test_data.chunks(100).map(|chunk| {
                    let maplet = std::sync::Arc::clone(&maplet);
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

/// Benchmark memory usage comparison
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(30));
    
    for size in [1000, 10000, 100000].iter() {
        let test_data = generate_test_data(*size);
        
        // Benchmark Redis memory usage (approximate)
        group.bench_with_input(BenchmarkId::new("Redis-MEMORY", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let redis = RedisBenchmark::new().await.unwrap();
                redis.flush_all().await.unwrap();
                
                for (key, value) in &test_data {
                    redis.set(key, value).await.unwrap();
                }
                
                // Get Redis memory info (approximate)
                let info: String = redis.connection.info("memory").await.unwrap();
                black_box(info)
                })
            })
        });
        
        // Benchmark Mappy memory usage
        group.bench_with_input(BenchmarkId::new("Mappy-MEMORY", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let maplet = Maplet::<String, String, StringOperator>::new(*size * 8, 0.01).unwrap();
                
                for (key, value) in &test_data {
                    maplet.insert(key.clone(), value.clone()).await.unwrap();
                }
                
                let stats = maplet.stats().await;
                black_box(stats)
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark mixed workload (realistic usage pattern)
fn bench_mixed_workload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("mixed_workload");
    group.measurement_time(Duration::from_secs(60));
    
    for size in [1000, 10000].iter() {
        let test_data = generate_test_data(*size);
        let mut rng = StdRng::seed_from_u64(42);
        
        // Benchmark Redis mixed workload
        group.bench_with_input(BenchmarkId::new("Redis-MIXED", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let redis = RedisBenchmark::new().await.unwrap();
                redis.flush_all().await.unwrap();
                
                // Mixed operations: 60% SET, 30% GET, 10% DEL
                for (key, value) in &test_data {
                    let op = rng.gen_range(0..100);
                    if op < 60 {
                        // SET operation
                        redis.set(key, value).await.unwrap();
                    } else if op < 90 {
                        // GET operation
                        redis.get(key).await.unwrap();
                    } else {
                        // DEL operation
                        redis.del(key).await.unwrap();
                    }
                }
                
                black_box(redis)
                })
            })
        });
        
        // Benchmark Mappy mixed workload
        group.bench_with_input(BenchmarkId::new("Mappy-MIXED", size), size, |b, _| {
            b.iter(|| {
    rt.block_on(async {
                let maplet = Maplet::<String, String, StringOperator>::new(*size * 8, 0.01).unwrap();
                
                // Mixed operations: 60% INSERT, 30% QUERY, 10% DELETE
                for (key, value) in &test_data {
                    let op = rng.gen_range(0..100);
                    if op < 60 {
                        // INSERT operation
                        maplet.insert(key.clone(), value.clone()).await.unwrap();
                    } else if op < 90 {
                        // QUERY operation
                        maplet.query(key).await;
                    } else {
                        // DELETE operation
                        maplet.delete(key).await.unwrap();
                    }
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
    bench_basic_operations,
    bench_get_operations,
    bench_counter_operations,
    bench_delete_operations,
    bench_exists_operations,
    bench_concurrent_operations,
    bench_memory_usage,
    bench_mixed_workload
);
criterion_main!(benches);
