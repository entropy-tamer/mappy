# Quotient Filter Documentation

## Overview

The Quotient Filter feature provides enhanced slot finding capabilities with run detection, shifting support, and comprehensive benchmarking for the Mappy maplet data structure. This feature enables precise inspection of the internal quotient filter state and debugging capabilities.

## Table of Contents

1. [Features](#features)
2. [Installation](#installation)
3. [Usage](#usage)
4. [API Reference](#api-reference)
5. [Testing](#testing)
6. [Benchmarking](#benchmarking)
7. [Python Integration](#python-integration)
8. [Performance Characteristics](#performance-characteristics)
9. [Troubleshooting](#troubleshooting)

## Features

### Core Features

- **Precise Slot Finding**: Locate exact storage slots considering runs and shifting
- **Run Detection**: Handle quotient filter runs with multiple fingerprints
- **Shifting Support**: Account for linear probing and slot shifting
- **Debugging Support**: Inspect internal storage layout for optimization
- **Performance Analysis**: Understand memory access patterns and cache behavior

### Comprehensive Testing & Benchmarking

- **62+ Test Cases**: Covering all edge cases and scenarios
- **Performance Benchmarks**: Detailed benchmarks showing 10-60M operations/second
- **Python Integration**: Full Python bindings for features
- **Memory Analysis**: Space efficiency and memory optimization testing
- **Concurrency Testing**: Thread-safe operations and race condition testing

## Installation

### Rust

Enable the `quotient-filter` feature in your `Cargo.toml`:

```toml
[dependencies]
mappy-core = { version = "0.1.0", features = ["quotient-filter"] }
```

### Python

The Python bindings automatically include features when the feature is enabled:

```bash
cd mappy-python
maturin develop --features quotient-filter
```

## Usage

### Rust Usage

#### Basic Slot Finding

```rust
use mappy_core::{Maplet, CounterOperator};

#[tokio::main]
async fn main() {
    // Create a maplet with quotient filter support
    let maplet = Maplet::<String, u64, CounterOperator>::new(1000, 0.01).unwrap();
    
    // Insert some data
    maplet.insert("test_key".to_string(), 42).await.unwrap();
    
    // Find the actual slot where a key's fingerprint is stored
    let slot = maplet.find_slot_for_key(&"test_key".to_string()).await;
    match slot {
        Some(slot_index) => println!("Key found at slot {}", slot_index),
        None => println!("Key not found"),
    }
}
```

#### Engine Integration

```rust
use mappy_core::{Engine, EngineConfig, MapletConfig, StorageConfig, TTLConfig, PersistenceMode};

#[tokio::main]
async fn main() {
    // Create engine configuration
    let config = EngineConfig {
        maplet: MapletConfig {
            capacity: 1000,
            false_positive_rate: 0.01,
            max_load_factor: 0.95,
            auto_resize: true,
            enable_deletion: true,
            enable_merging: true,
        },
        storage: StorageConfig {
            data_dir: "./data".to_string(),
            max_memory: Some(1024 * 1024 * 1024), // 1GB
            enable_compression: true,
            sync_interval: 1,
            write_buffer_size: 1024 * 1024,
        },
        ttl: TTLConfig {
            cleanup_interval_secs: 60,
            max_cleanup_batch_size: 1000,
            enable_background_cleanup: true,
        },
        persistence_mode: PersistenceMode::Hybrid,
        data_dir: Some("./data".to_string()),
    };
    
    // Create engine
    let engine = Engine::new(config).await.unwrap();
    
    // Insert data
    engine.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
    
    // Find slot for key
    let slot = engine.find_slot_for_key("key1").await.unwrap();
    println!("Key 'key1' found at slot: {:?}", slot);
}
```

### Python Usage

#### Basic Maplet Usage

```python
import mappy_python

# Create Maplet with advanced features
maplet = mappy_python.PyMaplet(capacity=1000, false_positive_rate=0.01)

# Insert data
maplet.insert("key", 42)

# Find slot for key
slot = maplet.find_slot_for_key("key")
print(f"Key 'key' found at slot: {slot}")

# Test with non-existing key (may return slot due to false positives)
non_existing_slot = maplet.find_slot_for_key("non_existing")
print(f"Non-existing key slot: {non_existing_slot}")
```

#### Engine Usage

```python
import mappy_python

# Create Engine with advanced features
config = mappy_python.PyEngineConfig(
    capacity=1000,
    false_positive_rate=0.01,
    persistence_mode="memory"
)
engine = mappy_python.PyEngine(config)

# Insert data
engine.set("key", b"value")

# Find slot for key
slot = engine.find_slot_for_key("key")
print(f"Key 'key' found at slot: {slot}")

# Cleanup
engine.close()
```

## API Reference

### Rust API

#### Maplet Methods

```rust
impl<K, V, O> Maplet<K, V, O> {
    /// Find the slot for a key (advanced quotient filter feature)
    #[cfg(feature = "advanced-quotient-filter")]
    pub async fn find_slot_for_key(&self, key: &K) -> Option<usize>
}
```

#### Engine Methods

```rust
impl Engine {
    /// Find the slot for a key (advanced quotient filter feature)
    #[cfg(feature = "advanced-quotient-filter")]
    pub async fn find_slot_for_key(&self, key: &str) -> MapletResult<Option<usize>>
}
```

### Python API

#### PyMaplet Methods

```python
class PyMaplet:
    def find_slot_for_key(self, key: str) -> Optional[int]:
        """
        Find the slot for a key (advanced quotient filter feature).
        
        Args:
            key: The key to find the slot for
            
        Returns:
            Optional[int]: The slot index if found, None otherwise
        """
```

#### PyEngine Methods

```python
class PyEngine:
    def find_slot_for_key(self, key: str) -> Optional[int]:
        """
        Find the slot for a key (advanced quotient filter feature).
        
        Args:
            key: The key to find the slot for
            
        Returns:
            Optional[int]: The slot index if found, None otherwise
        """
```

## Testing

### Running Tests

#### Rust Tests

```bash
# Run all tests with advanced features
cargo test --features advanced-quotient-filter

# Run specific test categories
cargo test --features advanced-quotient-filter quotient_filter
cargo test --features advanced-quotient-filter maplet
cargo test --features advanced-quotient-filter engine
```

#### Python Tests

```bash
# Run Python tests
python3 test_python_advanced_quotient_filter.py

# Run comprehensive test suite
./test_advanced_quotient_filter_complete.sh
```

### Test Coverage

The advanced quotient filter implementation includes 62+ comprehensive test cases:

#### Basic Operations (15 tests)

- Insert operations with various data types
- Query operations with different key types
- Delete operations and cleanup
- Error handling and edge cases

#### False Positive Rate (8 tests)

- Validation of probabilistic accuracy
- False positive rate measurement
- Error rate control and monitoring
- Statistical validation

#### Multiset Operations (10 tests)

- Counter operations and aggregation
- Set operations and unions
- Min/Max operations
- String concatenation

#### Run Detection (12 tests)

- Advanced slot finding with run handling
- Run start and end detection
- Shifting support and linear probing
- Collision chain management

#### Capacity Management (8 tests)

- Load factor monitoring
- Resizing behavior
- Capacity limits and overflow
- Memory usage optimization

#### Concurrency (9 tests)

- Thread-safe operations
- Race condition testing
- Concurrent access patterns
- Lock-free read operations

## Benchmarking

### Running Benchmarks

```bash
# Run basic quotient filter benchmarks
cargo bench --bench basic_quotient_filter_benchmarks

# Run with specific sample size
cargo bench --bench basic_quotient_filter_benchmarks -- --sample-size 10

# Run comprehensive test suite
./run_tests_and_benchmarks.sh
```

### Benchmark Results

#### Performance Characteristics

| Operation | Dataset Size | Performance | Throughput |
|-----------|--------------|-------------|------------|
| Insert    | 1,000 items  | 60.5 µs     | 16.5M ops/sec |
| Insert    | 10,000 items | 565 µs      | 17.7M ops/sec |
| Insert    | 100,000 items| 9.4 ms      | 10.6M ops/sec |
| Query     | 1,000 items  | 22.2 µs     | 45.0M ops/sec |
| Query     | 10,000 items | 274 µs      | 36.5M ops/sec |
| Query     | 100,000 items| 6.1 ms      | 16.4M ops/sec |
| Delete    | 1,000 items  | 117 µs      | 8.5M ops/sec |
| Delete    | 10,000 items | 1.19 ms     | 8.4M ops/sec |
| Delete    | 100,000 items| 24.7 ms     | 4.0M ops/sec |
| Slot Finding | 1,000 items | 16.3 µs     | 61.5M ops/sec |
| Slot Finding | 10,000 items | 201 µs      | 49.7M ops/sec |
| Slot Finding | 100,000 items| 4.1 ms      | 24.5M ops/sec |

#### Hash Function Performance

| Hash Function | Performance | Use Case |
|---------------|-------------|----------|
| AHash         | Fastest     | General purpose, good distribution |
| TwoX         | Medium      | Cryptographic security, slower |
| Fnv          | Slowest     | Simple, deterministic |

#### Memory Usage

| Dataset Size | Memory Usage | Efficiency |
|--------------|--------------|------------|
| 1,000 items  | ~8KB         | 8 bytes/item |
| 10,000 items | ~80KB        | 8 bytes/item |
| 100,000 items| ~800KB       | 8 bytes/item |
| 1,000,000 items| ~8MB       | 8 bytes/item |

## Python Integration

### Installation

```bash
# Build Python bindings with advanced features
cd mappy-python
maturin develop --features advanced-quotient-filter

# Or install from source
pip install -e . --features advanced-quotient-filter
```

### Python Features

- **Slot Finding**: `find_slot_for_key()` method for both Maplet and Engine
- **Error Handling**: Proper Python exceptions for invalid operations
- **Performance**: Same high-performance as Rust implementation
- **Concurrency**: Thread-safe operations in Python
- **Memory Management**: Automatic cleanup and resource management

### Python Test Suite

The Python test suite includes:

- **Basic Functionality**: Maplet and Engine operations
- **Advanced Features**: Slot finding and run detection
- **Performance Testing**: Benchmarking and memory analysis
- **Concurrent Operations**: Multi-threaded testing
- **Error Handling**: Exception handling and edge cases

## Performance Characteristics

### Key Performance Metrics

- **Insert Performance**: 10-17 million operations/second
- **Query Performance**: 16-45 million operations/second
- **Delete Performance**: 4-8 million operations/second
- **Slot Finding**: 24-61 million operations/second
- **Memory Efficiency**: Linear scaling with 8 bytes/item overhead
- **Cache Performance**: Optimized for sequential access patterns

### Optimization Features

- **Run Detection**: Efficient handling of quotient filter runs
- **Shifting Support**: Optimized linear probing
- **Memory Layout**: Cache-friendly data structures
- **Concurrent Access**: Lock-free read operations
- **Hash Function Selection**: Multiple hash functions for different use cases

## Troubleshooting

### Common Issues

#### Compilation Errors

```bash
# Ensure feature is enabled
cargo build --features advanced-quotient-filter

# Check feature dependencies
cargo check --features advanced-quotient-filter
```

#### Python Import Errors

```bash
# Rebuild Python bindings
cd mappy-python
maturin develop --features advanced-quotient-filter

# Check Python path
python3 -c "import mappy_python; print('Import successful')"
```

#### Test Failures

```bash
# Run tests with verbose output
cargo test --features advanced-quotient-filter -- --nocapture

# Run specific test
cargo test --features advanced-quotient-filter test_slot_finding_for_key
```

### Performance Issues

#### Slow Benchmarks

- Ensure release mode: `cargo bench --release`
- Use appropriate sample sizes: `--sample-size 10`
- Check system resources and background processes

#### Memory Usage

- Monitor memory usage with `htop` or `top`
- Use appropriate capacity settings
- Consider load factor and false positive rate

### Debugging

#### Enable Debug Logging

```rust
// Add to your Cargo.toml
[features]
debug = []

// Use in code
#[cfg(feature = "debug")]
println!("Debug: slot found at {}", slot);
```

#### Inspect Internal State

```rust
// Get quotient filter statistics
let stats = maplet.stats();
println!("Load factor: {:.2}%", stats.load_factor * 100.0);
println!("Memory usage: {} bytes", stats.memory_usage);
```

## Contributing

### Adding New Tests

1. Add test cases to `quotient_filter_tests.rs`
2. Add advanced tests to `advanced_quotient_filter_tests.rs`
3. Update benchmark categories in `basic_quotient_filter_benchmarks.rs`
4. Run tests to ensure they pass

### Adding New Features

1. Implement feature in core Rust code
2. Add Python bindings if needed
3. Add comprehensive tests
4. Update documentation
5. Add benchmarks if performance-critical

### Code Style

- Follow Rust naming conventions
- Add comprehensive documentation
- Include error handling
- Write tests for new functionality
- Update benchmarks for performance-critical code

## License

MIT License - see LICENSE file for details.

## References

- **Research Paper**: Bender, M. A., Conway, A., Farach-Colton, M., Johnson, R., & Pandey, P. (2025). Time To Replace Your Filter: How Maplets Simplify System Design.
- **Quotient Filters**: Pandey et al. (2017b) - "A general-purpose counting filter: Making every bit count"
- **Cuckoo Filters**: Fan et al. (2014) - "Cuckoo Filter: Practically Better Than Bloom"
- **API Documentation**: [https://docs.rs/mappy-core](https://docs.rs/mappy-core)
