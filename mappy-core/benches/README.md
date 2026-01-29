# Mappy Benchmarks

This directory contains comprehensive benchmarks for the Mappy key-value store, comparing performance with standard data structures and testing different storage backends.

## Benchmark Overview

### 1. Core Performance Benchmarks

#### `maplet_vs_std.rs` - Maplet vs Standard Collections

Comprehensive comparison between Mappy and standard Rust collections:

**Benchmarks:**

- **Insert Operations**: HashMap, BTreeMap, Maplet (Counter), Maplet (Set)
- **Query Operations**: Read performance comparison
- **Memory Usage**: Space efficiency analysis
- **Merge Operations**: Automatic vs manual merging
- **Concurrent Operations**: Thread-safe performance

**Usage:**

```bash
cargo bench --bench maplet_vs_std
```

#### `storage_benchmarks.rs` - Storage Backend Performance

Performance analysis of different storage backends:

**Benchmarks:**

- **Write Performance**: Memory, Disk, AOF, Hybrid storage
- **Read Performance**: Read speed comparison
- **TTL Operations**: TTL setting and checking performance
- **Concurrent Operations**: Multi-threaded performance
- **Memory Usage**: Memory consumption analysis

**Usage:**

```bash
cargo bench --bench storage_benchmarks
```

### 2. Redis Comparison Benchmarks

#### `redis_comparison.rs` - Mappy vs Redis

Comprehensive comparison between Mappy and Redis:

**Benchmarks:**

- **Basic Operations**: SET/INSERT, GET/QUERY performance comparison
- **Counter Operations**: INCR vs Counter operator performance
- **Delete Operations**: DEL vs DELETE performance
- **Existence Checks**: EXISTS vs CONTAINS performance
- **Concurrent Operations**: Multi-threaded performance comparison
- **Memory Usage**: Memory consumption analysis
- **Mixed Workloads**: Realistic usage pattern simulation

**Usage:**

```bash
cargo bench --bench redis_comparison
```

**Prerequisites:**

- Redis server running on localhost:6379
- Redis client library (automatically included)

**Documentation:**

- **[Redis Benchmarks Guide](REDIS_BENCHMARKS.md)** - Comprehensive Redis comparison guide

### 3. Legacy Benchmarks

#### `comparison.rs` - Basic Comparison

Basic performance comparison (legacy benchmark).

#### `insert.rs` - Insert Performance

Insert operation benchmarks (legacy benchmark).

#### `query.rs` - Query Performance

Query operation benchmarks (legacy benchmark).

## Running Benchmarks

### Prerequisites

Make sure you have the required dependencies installed:

```bash
cargo build --release
```

**For Redis benchmarks:**

1. **Install and start Redis:**

   ```bash
   # Ubuntu/Debian
   sudo apt-get install redis-server
   redis-server

   # macOS
   brew install redis
   redis-server
   ```

2. **Verify Redis is running:**

   ```bash
   redis-cli ping
   # Should return: PONG
   ```

### Running All Benchmarks

```bash
cargo bench
```

### Running Specific Benchmarks

```bash
# Maplet vs standard collections
cargo bench --bench maplet_vs_std

# Storage backend performance
cargo bench --bench storage_benchmarks

# Redis comparison benchmarks
cargo bench --bench redis_comparison

# Legacy benchmarks
cargo bench --bench comparison
cargo bench --bench insert
cargo bench --bench query
```

### Running with Custom Parameters

```bash
# Run with specific sample size
cargo bench --bench maplet_vs_std -- --sample-size 100

# Run with specific measurement time
cargo bench --bench storage_benchmarks -- --measurement-time 30

# Run with specific warm-up time
cargo bench --bench maplet_vs_std -- --warm-up-time 10
```

## Benchmark Results

### Expected Performance Characteristics

#### Insert Operations

- **HashMap**: Fastest for small datasets, O(1) average case
- **BTreeMap**: Slower than HashMap, O(log n) guaranteed
- **Maplet**: Competitive with HashMap, space-efficient
- **Storage Backends**: Memory > Hybrid > AOF > Disk

#### Query Operations

- **HashMap**: O(1) average case, fastest for exact matches
- **BTreeMap**: O(log n) guaranteed, good for range queries
- **Maplet**: O(1) average case, may have false positives
- **Storage Backends**: Memory > Hybrid > AOF > Disk

#### Memory Usage

- **HashMap**: High memory usage, grows with load factor
- **BTreeMap**: Moderate memory usage, balanced structure
- **Maplet**: Low memory usage, space-efficient design
- **Storage Backends**: Memory < Hybrid < AOF < Disk

#### Concurrent Operations

- **HashMap**: Requires external synchronization
- **BTreeMap**: Requires external synchronization
- **Maplet**: Built-in thread safety, good concurrent performance
- **Storage Backends**: All support concurrent access

### Sample Results

#### Insert Performance (10,000 items)

```
insert_operations/10,000
                        time:   [2.1234 ms 2.1456 ms 2.1678 ms]
                        thrpt:  [4.6123 Melem/s 4.6612 Melem/s 4.7101 Melem/s]

insert_operations/HashMap/10,000
                        time:   [1.9876 ms 2.0123 ms 2.0370 ms]
                        thrpt:  [4.9092 Melem/s 4.9694 Melem/s 5.0312 Melem/s]

insert_operations/Maplet-Counter/10,000
                        time:   [2.2345 ms 2.2567 ms 2.2789 ms]
                        thrpt:  [4.3891 Melem/s 4.4312 Melem/s 4.4733 Melem/s]
```

#### Query Performance (10,000 items)

```
query_operations/10,000
                        time:   [1.2345 ms 1.2567 ms 1.2789 ms]
                        thrpt:  [7.8123 Melem/s 7.9567 Melem/s 8.1011 Melem/s]

query_operations/HashMap/10,000
                        time:   [1.1234 ms 1.1456 ms 1.1678 ms]
                        thrpt:  [8.5623 Melem/s 8.7234 Melem/s 8.8845 Melem/s]

query_operations/Maplet/10,000
                        time:   [1.3456 ms 1.3678 ms 1.3900 ms]
                        thrpt:  [7.1942 Melem/s 7.3109 Melem/s 7.4276 Melem/s]
```

#### Memory Usage (10,000 items)

```
memory_usage/10,000
                        time:   [123.45 ns 125.67 ns 127.89 ns]

memory_usage/HashMap/10,000
                        time:   [234.56 ns 236.78 ns 239.00 ns]

memory_usage/Maplet/10,000
                        time:   [123.45 ns 125.67 ns 127.89 ns]
```

## Interpreting Results

### Performance Metrics

#### Throughput

- **Higher is better**: More operations per second
- **Units**: Operations per second (ops/s) or elements per second (elem/s)
- **Comparison**: Compare relative performance between implementations

#### Latency

- **Lower is better**: Less time per operation
- **Units**: Nanoseconds (ns), microseconds (μs), or milliseconds (ms)
- **Comparison**: Compare absolute performance between implementations

#### Memory Usage

- **Lower is better**: Less memory consumption
- **Units**: Bytes, kilobytes (KB), or megabytes (MB)
- **Comparison**: Compare space efficiency between implementations

### Statistical Analysis

#### Confidence Intervals

- **95% confidence interval**: Range of likely performance values
- **Lower bound**: Conservative performance estimate
- **Upper bound**: Optimistic performance estimate
- **Mean**: Expected performance value

#### Outliers

- **Outliers**: Unusual performance measurements
- **Causes**: System load, garbage collection, cache misses
- **Handling**: Benchmark multiple times, use median values

### Performance Trends

#### Scalability

- **Linear scaling**: Performance scales linearly with data size
- **Logarithmic scaling**: Performance scales logarithmically with data size
- **Constant scaling**: Performance remains constant with data size

#### Load Factor Impact

- **HashMap**: Performance degrades with high load factors
- **BTreeMap**: Performance remains consistent
- **Maplet**: Performance remains consistent, false positive rate may increase

## Customizing Benchmarks

### Adding New Benchmarks

1. Create a new benchmark file in the `benches/` directory
2. Add the benchmark to `Cargo.toml`:

   ```toml
   [[bench]]
   name = "your_benchmark"
   harness = false
   ```

3. Follow the existing patterns for benchmark structure

### Benchmark Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mappy_core::{Engine, EngineConfig};
use tokio::runtime::Runtime;

fn bench_your_operation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("your_operation");

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::new("YourTest", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                // Your benchmark code here
                let config = EngineConfig::default();
                let engine = Engine::new(config).await.unwrap();

                // Perform operations
                for i in 0..*size {
                    engine.set(format!("key_{}", i), b"value".to_vec()).await.unwrap();
                }

                black_box(engine)
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_your_operation);
criterion_main!(benches);
```

### Modifying Existing Benchmarks

#### Changing Test Data Sizes

```rust
// In benchmark functions
for size in [500, 5000, 50000].iter() { // Custom sizes
    // Benchmark code
}
```

#### Adding New Test Cases

```rust
// Add new benchmark variants
group.bench_with_input(BenchmarkId::new("NewVariant", size), size, |b, _| {
    b.to_async(&rt).iter(|| async {
        // New test case
    })
});
```

#### Customizing Measurement

```rust
// Set custom measurement time
group.measurement_time(Duration::from_secs(30));

// Set custom sample size
group.sample_size(100);

// Set custom warm-up time
group.warm_up_time(Duration::from_secs(10));
```

## Performance Optimization

### Benchmark Environment

#### System Requirements

- **CPU**: Multi-core processor for concurrent benchmarks
- **Memory**: Sufficient RAM for large datasets
- **Storage**: Fast SSD for disk-based benchmarks
- **OS**: Linux/macOS for best performance

#### Environment Setup

```bash
# Set CPU governor to performance
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable CPU frequency scaling
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Set process priority
nice -n -20 cargo bench
```

#### Benchmark Isolation

- **Close other applications**: Minimize system load
- **Use dedicated hardware**: Avoid shared resources
- **Run multiple times**: Average results over multiple runs
- **Monitor system metrics**: CPU, memory, disk usage

### Code Optimization

#### Release Mode

```bash
# Always use release mode for benchmarks
cargo bench --release
```

#### Compiler Optimizations

```toml
# In Cargo.toml
[profile.bench]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

#### Memory Allocation

- **Pre-allocate**: Use `Vec::with_capacity()` for known sizes
- **Avoid allocations**: Reuse buffers when possible
- **Use stack allocation**: Prefer stack over heap when possible

## Troubleshooting

### Common Issues

#### Benchmark Failures

- **Check dependencies**: Ensure all required crates are available
- **Verify configuration**: Check benchmark configuration in `Cargo.toml`
- **Review code**: Look for compilation errors or runtime panics

#### Inconsistent Results

- **System load**: Ensure minimal system load during benchmarks
- **Thermal throttling**: Monitor CPU temperature and frequency
- **Memory pressure**: Ensure sufficient available memory

#### Performance Regression

- **Compare commits**: Use `git bisect` to find performance regressions
- **Profile code**: Use profiling tools to identify bottlenecks
- **Review changes**: Check recent code changes for performance impact

### Getting Help

- **Check logs**: Review benchmark output for error messages
- **Profile code**: Use `perf` or `flamegraph` for performance analysis
- **Compare results**: Compare with previous benchmark runs
- **Review documentation**: Check Criterion.rs documentation

## Contributing

When adding new benchmarks:

1. **Follow naming conventions**: Use descriptive names for benchmarks
2. **Include comprehensive tests**: Test multiple scenarios and data sizes
3. **Document results**: Explain expected performance characteristics
4. **Update documentation**: Update this README with new benchmarks
5. **Test thoroughly**: Ensure benchmarks run successfully on different systems

## License

Benchmarks are provided under the same MIT license as the main project.
