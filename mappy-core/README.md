# mappy-core

[![Crates.io](https://img.shields.io/crates/v/mappy-core.svg)](https://crates.io/crates/mappy-core)
[![Documentation](https://docs.rs/mappy-core/badge.svg)](https://docs.rs/mappy-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Core maplet data structure implementation** - Space-efficient approximate key-value mappings with one-sided error guarantees.

## Overview

Maplets provide the same space benefits as filters while natively supporting key-value associations. Unlike traditional filters that only support set membership queries, maplets allow you to associate values with keys and retrieve them during queries with configurable merge operators.

Based on the research paper: *"Time To Replace Your Filter: How Maplets Simplify System Design"* by Bender et al. (2025).

## Key Features

- **Space Efficiency**: Achieves `O(log 1/ε + v)` bits per item where ε is the false-positive rate and v is the value size
- **Value Support**: Native key-value associations with configurable merge operators
- **One-Sided Errors**: Guarantees `M[k] ≺ m[k]` for application-specific ordering relations
- **Deletion Support**: Full support for removing key-value pairs
- **Merging**: Combine maplets using associative/commutative operators
- **Resizing**: Dynamic growth with efficient rehashing
- **Cache Locality**: Optimized memory layout for performance
- **Concurrency**: Thread-safe operations with lock-free reads

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
mappy-core = "0.1.0"
```

### Basic Usage

```rust
use mappy_core::{Maplet, CounterOperator};

// Create a maplet for counting with 1% false-positive rate
let mut maplet = Maplet::<String, u64, CounterOperator>::new(1000, 0.01);

// Insert key-value pairs
maplet.insert("key1".to_string(), 5).unwrap();
maplet.insert("key2".to_string(), 3).unwrap();

// Query values (may return approximate results)
let count1 = maplet.query(&"key1".to_string()); // Some(5) or Some(5 + other_values)
let count2 = maplet.query(&"key2".to_string()); // Some(3) or Some(3 + other_values)

// Check if key exists
let exists = maplet.contains(&"key1".to_string()); // true

// Get statistics
let stats = maplet.stats();
println!("Load factor: {:.2}%", stats.load_factor * 100.0);
println!("Memory usage: {} bytes", stats.memory_usage);
```

## Use Cases

### 1. K-mer Counting (Computational Biology)

```rust
use mappy_core::{Maplet, CounterOperator};

let mut kmer_counter = Maplet::<String, u32, CounterOperator>::new(1_000_000, 0.001);
// Count k-mers in DNA sequences with high accuracy
```

### 2. Network Routing Tables

```rust
use mappy_core::{Maplet, SetOperator};
use std::collections::HashSet;

let mut routing_table = Maplet::<String, HashSet<String>, SetOperator>::new(100_000, 0.01);
// Map network prefixes to sets of next-hop routers
```

### 3. LSM Storage Engine Index

```rust
use mappy_core::{Maplet, MaxOperator};

let mut sstable_index = Maplet::<String, u64, MaxOperator>::new(10_000_000, 0.001);
// Map keys to SSTable identifiers for efficient lookups
```

## Available Merge Operators

- **`CounterOperator`**: Adds values together (useful for counting)
- **`MaxOperator`**: Takes the maximum value
- **`MinOperator`**: Takes the minimum value
- **`SetOperator`**: Union of sets
- **`CustomOperator`**: Define your own merge function

## Performance Characteristics

- **Query Throughput**: Within 2x of HashMap for same memory usage
- **Memory Efficiency**: 20-50% reduction vs HashMap for typical workloads
- **Error Control**: False-positive rate within 1.5x of configured ε
- **Cache Performance**: Optimized for sequential access patterns

## Error Guarantees

Maplets provide the **strong maplet property**:

```
m[k] = M[k] ⊕ (⊕ᵢ₌₁ˡ M[kᵢ])
```

Where `Pr[ℓ ≥ L] ≤ ε^L`, meaning even when wrong, the result is close to correct.

## Storage Backends

- **Memory**: In-memory storage for fast access
- **Disk**: Persistent storage with configurable durability
- **AOF**: Append-only file for crash recovery
- **Hybrid**: Combination of memory and disk storage

## Examples

See the `examples/` directory for comprehensive usage examples:

- `counter.rs` - Basic counting example
- `routing.rs` - Network routing table simulation
- `caching_service.rs` - Distributed caching system
- `performance_comparison.rs` - Benchmarking against standard HashMap

## Benchmarks

Run benchmarks to see performance characteristics:

```bash
cargo bench
```

## Documentation

- **[API Documentation](https://docs.rs/mappy-core)** - Complete API reference
- **[Technical Guide](../TECHNICAL_README.md)** - Comprehensive technical documentation
- **[Research Foundation](https://github.com/entropy-tamer/mappy)** - Original research papers

## License

MIT License - see [LICENSE](../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/entropy-tamer/mappy) for contribution guidelines.

## References

Based on the research paper:

> Bender, M. A., Conway, A., Farach-Colton, M., Johnson, R., & Pandey, P. (2025). Time To Replace Your Filter: How Maplets Simplify System Design. arXiv preprint arXiv:2510.05518.
