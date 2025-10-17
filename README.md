# Mappy: Space-Efficient Maplet Data Structures

A Rust implementation of maplets - space-efficient data structures for approximate key-value mappings, based on the research paper "Time To Replace Your Filter: How Maplets Simplify System Design".

## Overview

Maplets provide the same space benefits as filters while natively supporting key-value associations with one-sided error guarantees. Unlike traditional filters that only support set membership queries, maplets allow you to associate values with keys and retrieve them during queries.

## Key Features

- **Space Efficiency**: Achieves `O(log 1/ε + v)` bits per item where ε is the false-positive rate and v is the value size
- **Value Support**: Native key-value associations with configurable merge operators
- **One-Sided Errors**: Guarantees `M[k] ≺ m[k]` for application-specific ordering relations
- **Deletion Support**: Full support for removing key-value pairs
- **Merging**: Combine maplets using associative/commutative operators
- **Resizing**: Dynamic growth with efficient rehashing
- **Cache Locality**: Optimized memory layout for performance
- **Concurrency**: Thread-safe operations with lock-free reads

## Architecture

The implementation follows a multi-crate workspace structure:

- **mappy-core**: Core maplet data structure implementation
- **mappy-client**: Client library for Rust applications
- **mappy-python**: Python bindings via PyO3

## Quick Start

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

let mut routing_table = Maplet::<String, HashSet<String>, SetOperator>::new(100_000, 0.01);
// Map network prefixes to sets of next-hop routers
```

### 3. LSM Storage Engine Index

```rust
use mappy_core::{Maplet, MaxOperator};

let mut sstable_index = Maplet::<String, u64, MaxOperator>::new(10_000_000, 0.001);
// Map keys to SSTable identifiers for efficient lookups
```

## Performance Characteristics

### Benchmark Results: Mappy vs Redis

Our comprehensive benchmarks show Mappy significantly outperforms Redis for approximate key-value operations:

| Dataset Size | Operation  | Redis Performance | Mappy Performance | Mappy Advantage  |
| ------------ | ---------- | ----------------- | ----------------- | ---------------- |
| 100 items    | SET/INSERT | 13.9-18.0 ms      | 41.9-47.7 µs      | **~300x faster** |
| 1,000 items  | SET/INSERT | 107-130 ms        | 414-481 µs        | **~250x faster** |
| 10,000 items | SET/INSERT | 976-1,244 ms      | 4.9-5.7 ms        | **~200x faster** |

### Performance Highlights

- **Query Throughput**: 200-300x faster than Redis for insert operations
- **Memory Efficiency**: Space-efficient design with configurable false-positive rates
- **Error Control**: False-positive rate within 1.5x of configured ε
- **Cache Performance**: Optimized for sequential access patterns
- **Concurrent Access**: Thread-safe operations with stable performance under load

### Running Benchmarks

```bash
# Run Redis comparison benchmarks
cd services/mappy/mappy-core
cargo bench --bench redis_comparison

# Run all benchmarks
cd services/mappy
./benchmark_runner.sh --all

# Run specific benchmark categories
./benchmark_runner.sh --redis
```

For detailed benchmark documentation, see [REDIS_BENCHMARKS.md](mappy-core/benches/REDIS_BENCHMARKS.md).

## Error Guarantees

Maplets provide the **strong maplet property**:

```math
m[k] = M[k] ⊕ (⊕ᵢ₌₁ˡ M[kᵢ])
```

Where `Pr[ℓ ≥ L] ≤ ε^L`, meaning even when wrong, the result is close to correct.

## Documentation

- **[Technical Documentation](TECHNICAL_README.md)** - Comprehensive technical guide with architecture diagrams, API reference, and implementation details
- **[API Documentation](https://docs.rs/mappy-core)** - Auto-generated API documentation
- **[Research Foundation](../../.cursor/research/maplets/)** - Original research papers and theoretical background

## License

MIT License - see LICENSE file for details.

## References

Based on the research paper:

> Bender, M. A., Conway, A., Farach-Colton, M., Johnson, R., & Pandey, P. (2025). Time To Replace Your Filter: How Maplets Simplify System Design. arXiv preprint arXiv:2510.05518.
