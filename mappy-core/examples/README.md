# Mappy Examples

This directory contains comprehensive examples demonstrating the capabilities of the Mappy key-value store.

## Examples Overview

### 1. Basic Examples

#### `counter.rs` - K-mer Counting

Demonstrates using Mappy for bioinformatics applications, specifically k-mer counting in DNA sequences.

**Key Features:**

- Approximate counting with space efficiency
- Merge operations for duplicate k-mers
- Performance comparison with HashMap

**Usage:**

```bash
cargo run --example counter
```

#### `routing.rs` - Network Routing

Shows how to use Mappy for network routing table management with prefix matching.

**Key Features:**

- Set operations for route aggregation
- Longest prefix matching
- Network topology simulation

**Usage:**

```bash
cargo run --example routing
```

### 2. Advanced Examples

#### `comprehensive_demo.rs` - Full Feature Demo

Comprehensive demonstration of all Mappy features including:

- Basic operations (get, set, delete)
- TTL management
- Different storage backends
- Statistics and monitoring
- Concurrent operations
- Real-world use cases

**Usage:**

```bash
cargo run --example comprehensive_demo
```

#### `performance_comparison.rs` - Performance Analysis

Detailed performance comparison between Mappy and standard Rust collections:

- Insert/query performance
- Memory usage analysis
- Concurrent operation benchmarks
- False positive rate impact

**Usage:**

```bash
cargo run --example performance_comparison
```

#### `caching_service.rs` - Real-world Caching

Production-ready caching service example with:

- Cache hit/miss tracking
- TTL management
- Cache invalidation
- Performance monitoring
- Database integration simulation

**Usage:**

```bash
cargo run --example caching_service
```

## Running Examples

### Prerequisites

Make sure you have the required dependencies installed:

```bash
cargo build
```

### Basic Examples

```bash
# K-mer counting
cargo run --example counter

# Network routing
cargo run --example routing
```

### Advanced Examples

```bash
# Comprehensive demo
cargo run --example comprehensive_demo

# Performance comparison
cargo run --example performance_comparison

# Caching service
cargo run --example caching_service
```

### Running with Release Optimizations

For better performance, run examples in release mode:

```bash
cargo run --release --example comprehensive_demo
```

## Example Outputs

### Counter Example

```
🧬 K-mer Counting Demo
=====================

Processing 3 sequences with k=3...

Sequence 1: ATGCGATCG
  K-mers: ATC, TCG, CGA, GAT, ATC, TCG

Sequence 2: GCTAGCTAG
  K-mers: GCT, CTA, TAG, AGC, GCT, CTA, TAG

Sequence 3: TTTAAACCC
  K-mers: TTT, TTA, TAA, AAA, AAC, ACC, CCC

Querying test k-mers:
  ATC: Some(2)
  TCG: Some(2)
  GCT: Some(2)
  TTT: Some(1)
  XYZ: None

Maplet Statistics:
  Length: 7
  Load factor: 0.07
  False positive rate: 0.01
```

### Routing Example

```
🌐 Network Routing Demo
======================

Adding routes to routing table...

Route: 192.168.1.0/24 -> [192.168.1.1, 192.168.1.2]
Route: 10.0.0.0/8 -> [10.0.0.1]
Route: 0.0.0.0/0 -> [203.0.113.1]

Querying test IPs:
  192.168.1.100 -> Some({"192.168.1.1", "192.168.1.2"})
  10.0.0.5 -> Some({"10.0.0.1"})
  8.8.8.8 -> Some({"203.0.113.1"})
  172.16.0.1 -> None

Maplet Statistics:
  Length: 3
  Load factor: 0.03
  False positive rate: 0.01
```

### Comprehensive Demo

```
🚀 Mappy Comprehensive Demo
==========================

📝 Demo 1: Basic Operations
---------------------------
  user:1 = "Alice"
  user:2 exists: true
  All keys: ["user:1", "user:2", "user:3"]
  Deleted user:3: true
  Final keys: ["user:1", "user:2"]
  ✅ Basic operations demo completed

⏰ Demo 2: TTL Management
-------------------------
  session:abc123 TTL: Some(2) seconds
  Waiting for session:abc123 to expire...
  session:abc123 after expiration: None
  cache:data remaining TTL: Some(7) seconds
  Removed TTL from cache:data: true
  cache:data TTL after persist: None
  ✅ TTL management demo completed

💾 Demo 3: Storage Backends
---------------------------
  Memory storage: "memory_value"
  Disk storage: "disk_value"
  AOF storage: "aof_value"
  ✅ Storage backends demo completed

📊 Demo 4: Statistics and Monitoring
------------------------------------
  Engine Statistics:
    Uptime: 0 seconds
    Total operations: 100
    Maplet stats: MapletStats { length: 100, load_factor: 0.01, false_positive_rate: 0.01 }
    Storage stats: StorageStats { total_keys: 100, memory_usage: 1024, disk_usage: 0, operations_count: 100, avg_latency_us: 100 }
    TTL stats: TTLStats { total_keys_with_ttl: 10, expired_keys: 0, cleanup_operations: 0 }
  ✅ Statistics demo completed

🔄 Demo 5: Concurrent Operations
-------------------------------
  Total keys after concurrent operations: 1000
  ✅ Concurrent operations completed successfully
  ✅ Concurrent operations demo completed

🌍 Demo 6: Real-world Use Cases
-------------------------------
  📱 Session Management:
    Session TTL: Some(3600) seconds
  🗄️  Caching:
    Cache hit: "result_data"
  🚦 Rate Limiting:
    Request 2: Allowed (count: 2)
    Request 3: Allowed (count: 3)
    Request 4: Rate limited (count: 3)
    Request 5: Rate limited (count: 3)
  🚩 Feature Flags:
    New UI: "enabled"
    Beta Feature: "disabled"
  ✅ Real-world use cases demo completed

✅ All demos completed successfully!
```

## Customizing Examples

### Modifying Parameters

You can easily modify example parameters by editing the source files:

```rust
// In counter.rs
let k = 4; // Change k-mer size
let sequences = vec![
    "ATGCGATCGATCG".to_string(),
    "GCTAGCTAGCTAG".to_string(),
    // Add more sequences
];

// In routing.rs
let routes = vec![
    ("192.168.1.0/24".to_string(), vec!["192.168.1.1", "192.168.1.2"]),
    ("10.0.0.0/8".to_string(), vec!["10.0.0.1"]),
    // Add more routes
];
```

### Adding New Examples

To create a new example:

1. Create a new file in the `examples/` directory
2. Add the example to `Cargo.toml` if needed
3. Follow the existing patterns for error handling and async operations

Example template:

```rust
use mappy_core::{Engine, EngineConfig};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 My Custom Example");

    let config = EngineConfig::default();
    let engine = Engine::new(config).await?;

    // Your example code here

    engine.close().await?;
    println!("✅ Example completed!");
    Ok(())
}
```

## Performance Tips

### For Large Datasets

- Use release mode: `cargo run --release --example your_example`
- Adjust maplet capacity based on expected data size
- Consider using disk persistence for large datasets
- Monitor memory usage with `engine.memory_usage().await?`

### For High Performance

- Use memory persistence mode for temporary data
- Set appropriate false positive rates (0.01-0.1)
- Use concurrent operations for parallel processing
- Monitor statistics with `engine.stats().await?`

### For Production Use

- Use disk or hybrid persistence modes
- Set appropriate TTL values
- Implement proper error handling
- Monitor cache hit rates and performance metrics

## Troubleshooting

### Common Issues

1. **Out of Memory**: Reduce maplet capacity or use disk persistence
2. **Slow Performance**: Use release mode and optimize false positive rate
3. **Data Loss**: Ensure proper persistence mode and TTL settings
4. **Compilation Errors**: Check dependencies and Rust version

### Getting Help

- Check the [API documentation](../API.md)
- Review the [main README](../README.md)
- Look at existing examples for patterns
- Check error messages and logs

## Contributing

When adding new examples:

1. Follow the existing naming conventions
2. Include comprehensive error handling
3. Add meaningful output and logging
4. Document the example's purpose and usage
5. Test with different configurations
6. Update this README with new examples

## License

Examples are provided under the same MIT license as the main project.
