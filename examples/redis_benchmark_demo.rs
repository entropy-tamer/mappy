//! Redis Benchmark Demo
//! 
//! This example demonstrates how to run Redis benchmarks and interpret the results.
//! It provides a simple interface to compare Mappy with Redis performance.

use std::time::Instant;
use tokio::runtime::Runtime;
use redis::AsyncCommands;
use redis::Client as RedisClient;
use mappy_core::{Maplet, CounterOperator, SetOperator};

/// Simple Redis client wrapper
struct SimpleRedis {
    connection: redis::aof::Connection,
}

impl SimpleRedis {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = RedisClient::open("redis://127.0.0.1:6379")?;
        let connection = client.get_async_connection().await?;
        Ok(Self { connection })
    }
    
    async fn set(&mut self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _: () = self.connection.set(key, value).await?;
        Ok(())
    }
    
    async fn get(&mut self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let result: Option<String> = self.connection.get(key).await?;
        Ok(result)
    }
    
    async fn flush_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let _: () = self.connection.flushall().await?;
        Ok(())
    }
}

/// Benchmark configuration
struct BenchmarkConfig {
    num_items: usize,
    key_prefix: String,
    value_prefix: String,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            num_items: 1000,
            key_prefix: "demo_key".to_string(),
            value_prefix: "demo_value".to_string(),
        }
    }
}

/// Benchmark results
struct BenchmarkResults {
    operation: String,
    duration: std::time::Duration,
    throughput: f64,
    memory_usage: Option<usize>,
}

/// Run a simple benchmark comparing Redis and Mappy
async fn run_simple_benchmark(config: BenchmarkConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("🦊 Mappy vs Redis Simple Benchmark");
    println!("==================================");
    println!("Items: {}", config.num_items);
    println!();
    
    // Test data
    let test_data: Vec<(String, String)> = (0..config.num_items)
        .map(|i| {
            let key = format!("{}_{}", config.key_prefix, i);
            let value = format!("{}_{}", config.value_prefix, i);
            (key, value)
        })
        .collect();
    
    // Benchmark Redis
    println!("📊 Benchmarking Redis...");
    let mut redis = SimpleRedis::new().await?;
    redis.flush_all().await?;
    
    let start = Instant::now();
    for (key, value) in &test_data {
        redis.set(key, value).await?;
    }
    let redis_insert_duration = start.elapsed();
    
    let start = Instant::now();
    for (key, _) in &test_data {
        redis.get(key).await?;
    }
    let redis_query_duration = start.elapsed();
    
    // Benchmark Mappy
    println!("📊 Benchmarking Mappy...");
    let maplet = Maplet::<String, String, SetOperator>::new(config.num_items * 8, 0.01).unwrap();
    
    let start = Instant::now();
    for (key, value) in &test_data {
        maplet.insert(key.clone(), value.clone()).await?;
    }
    let mappy_insert_duration = start.elapsed();
    
    let start = Instant::now();
    for (key, _) in &test_data {
        maplet.query(key).await;
    }
    let mappy_query_duration = start.elapsed();
    
    // Display results
    println!();
    println!("📈 Results:");
    println!("===========");
    
    // Insert operations
    println!("Insert Operations:");
    println!("  Redis:  {:.2}ms ({:.0} ops/s)", 
             redis_insert_duration.as_secs_f64() * 1000.0,
             config.num_items as f64 / redis_insert_duration.as_secs_f64());
    println!("  Mappy:  {:.2}ms ({:.0} ops/s)", 
             mappy_insert_duration.as_secs_f64() * 1000.0,
             config.num_items as f64 / mappy_insert_duration.as_secs_f64());
    
    // Query operations
    println!("Query Operations:");
    println!("  Redis:  {:.2}ms ({:.0} ops/s)", 
             redis_query_duration.as_secs_f64() * 1000.0,
             config.num_items as f64 / redis_query_duration.as_secs_f64());
    println!("  Mappy:  {:.2}ms ({:.0} ops/s)", 
             mappy_query_duration.as_secs_f64() * 1000.0,
             config.num_items as f64 / mappy_query_duration.as_secs_f64());
    
    // Performance comparison
    println!();
    println!("📊 Performance Comparison:");
    println!("=========================");
    
    let redis_insert_ops = config.num_items as f64 / redis_insert_duration.as_secs_f64();
    let mappy_insert_ops = config.num_items as f64 / mappy_insert_duration.as_secs_f64();
    let redis_query_ops = config.num_items as f64 / redis_query_duration.as_secs_f64();
    let mappy_query_ops = config.num_items as f64 / mappy_query_duration.as_secs_f64();
    
    println!("Insert Performance:");
    if mappy_insert_ops > redis_insert_ops {
        println!("  🦊 Mappy is {:.1}x faster than Redis", mappy_insert_ops / redis_insert_ops);
    } else {
        println!("  🔴 Redis is {:.1}x faster than Mappy", redis_insert_ops / mappy_insert_ops);
    }
    
    println!("Query Performance:");
    if mappy_query_ops > redis_query_ops {
        println!("  🦊 Mappy is {:.1}x faster than Redis", mappy_query_ops / redis_query_ops);
    } else {
        println!("  🔴 Redis is {:.1}x faster than Mappy", redis_query_ops / mappy_query_ops);
    }
    
    // Memory usage (approximate)
    println!();
    println!("💾 Memory Usage (Approximate):");
    println!("==============================");
    
    // Redis memory (rough estimate)
    let redis_memory = config.num_items * (config.key_prefix.len() + config.value_prefix.len() + 20); // Rough estimate
    println!("  Redis:  ~{} KB", redis_memory / 1024);
    
    // Mappy memory
    let mappy_stats = maplet.stats().await;
    println!("  Mappy:  ~{} KB", mappy_stats.memory_usage / 1024);
    
    let memory_ratio = redis_memory as f64 / mappy_stats.memory_usage as f64;
    if memory_ratio > 1.0 {
        println!("  🦊 Mappy uses {:.1}x less memory than Redis", memory_ratio);
    } else {
        println!("  🔴 Redis uses {:.1}x less memory than Mappy", 1.0 / memory_ratio);
    }
    
    Ok(())
}

/// Run counter benchmark
async fn run_counter_benchmark(config: BenchmarkConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("🔢 Counter Operations Benchmark");
    println!("===============================");
    
    // Test data for counters
    let counter_data: Vec<(String, u64)> = (0..config.num_items)
        .map(|i| {
            let key = format!("counter_{}", i);
            let value = (i % 100) as u64 + 1;
            (key, value)
        })
        .collect();
    
    // Benchmark Redis counters
    println!("📊 Benchmarking Redis Counters...");
    let mut redis = SimpleRedis::new().await?;
    redis.flush_all().await?;
    
    let start = Instant::now();
    for (key, value) in &counter_data {
        redis.set(key, &value.to_string()).await?;
        // Simulate increment
        let _: i64 = redis.connection.incr(key, 1).await?;
    }
    let redis_counter_duration = start.elapsed();
    
    // Benchmark Mappy counters
    println!("📊 Benchmarking Mappy Counters...");
    let maplet = Maplet::<String, u64, CounterOperator>::new(config.num_items * 8, 0.01).unwrap();
    
    let start = Instant::now();
    for (key, value) in &counter_data {
        maplet.insert(key.clone(), *value).await?;
    }
    let mappy_counter_duration = start.elapsed();
    
    // Display results
    println!();
    println!("📈 Counter Results:");
    println!("==================");
    
    let redis_counter_ops = config.num_items as f64 / redis_counter_duration.as_secs_f64();
    let mappy_counter_ops = config.num_items as f64 / mappy_counter_duration.as_secs_f64();
    
    println!("  Redis:  {:.2}ms ({:.0} ops/s)", 
             redis_counter_duration.as_secs_f64() * 1000.0, redis_counter_ops);
    println!("  Mappy:  {:.2}ms ({:.0} ops/s)", 
             mappy_counter_duration.as_secs_f64() * 1000.0, mappy_counter_ops);
    
    if mappy_counter_ops > redis_counter_ops {
        println!("  🦊 Mappy is {:.1}x faster than Redis", mappy_counter_ops / redis_counter_ops);
    } else {
        println!("  🔴 Redis is {:.1}x faster than Mappy", redis_counter_ops / mappy_counter_ops);
    }
    
    Ok(())
}

/// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦊 Mappy vs Redis Benchmark Demo");
    println!("================================");
    println!();
    
    // Check if Redis is running
    match SimpleRedis::new().await {
        Ok(_) => println!("✅ Redis connection successful"),
        Err(e) => {
            println!("❌ Redis connection failed: {}", e);
            println!("Please ensure Redis is running on localhost:6379");
            return Ok(());
        }
    }
    
    // Run benchmarks with different configurations
    let configs = vec![
        BenchmarkConfig { num_items: 100, ..Default::default() },
        BenchmarkConfig { num_items: 1000, ..Default::default() },
        BenchmarkConfig { num_items: 10000, ..Default::default() },
    ];
    
    for config in configs {
        run_simple_benchmark(config).await?;
        run_counter_benchmark(config).await?;
        println!();
        println!("{}", "=".repeat(50));
        println!();
    }
    
    println!("🎉 Benchmark demo completed!");
    println!();
    println!("💡 Tips:");
    println!("  - Run 'cargo bench --bench redis_comparison' for comprehensive benchmarks");
    println!("  - Use './benchmark_runner.sh --redis' for automated benchmarking");
    println!("  - Check REDIS_BENCHMARKS.md for detailed documentation");
    
    Ok(())
}
