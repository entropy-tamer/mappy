# Mappy Documentation Index

## Overview

This document provides a comprehensive index of all documentation for the Mappy project, including the new Advanced Quotient Filter implementations, comprehensive testing, benchmarking, and Python integration.

## Main Documentation

### Core Documentation

1. **[README.md](README.md)** - Main project documentation
   - Project overview and key features
   - Quick start guide and basic usage
   - Advanced quotient filter features
   - Performance characteristics and benchmarks
   - Use cases and examples

2. **[TECHNICAL_README.md](TECHNICAL_README.md)** - Comprehensive technical guide
   - Architecture and design principles
   - Mathematical foundation and guarantees
   - API reference and implementation details
   - Performance analysis and optimization
   - Advanced quotient filter technical details

3. **[QUOTIENT_FILTER.md](QUOTIENT_FILTER.md)** - Quotient filter features documentation
   - Quotient filter features and capabilities
   - Comprehensive testing and benchmarking
   - Python integration and bindings
   - Performance characteristics and optimization
   - Troubleshooting and debugging guide

## Testing and Benchmarking Documentation

### Test Documentation

4. **[mappy-core/benches/QUOTIENT_FILTER_BENCHMARKS.md](mappy-core/benches/QUOTIENT_FILTER_BENCHMARKS.md)** - Comprehensive benchmark documentation
   - Test structure and organization
   - Benchmark categories and results
   - Performance characteristics and metrics
   - Running instructions and troubleshooting
   - Recent benchmark results with 10-60M operations/second

### Test Files

5. **[mappy-core/src/quotient_filter_tests.rs](mappy-core/src/quotient_filter_tests.rs)** - Basic quotient filter tests
   - 62+ comprehensive test cases
   - Basic operations, false positive rate, multiset operations
   - Run detection, capacity management, concurrency
   - Edge cases, hash functions, memory usage

6. **[mappy-core/src/advanced_quotient_filter_tests.rs](mappy-core/src/advanced_quotient_filter_tests.rs)** - Advanced quotient filter tests
   - 12+ advanced test cases
   - Advanced slot finding, run detection, Maplet integration
   - Collision handling, performance testing
   - Hash function variations, multiset operations
   - Concurrency, edge cases, accuracy testing

### Benchmark Files

7. **[mappy-core/benches/basic_quotient_filter_benchmarks.rs](mappy-core/benches/basic_quotient_filter_benchmarks.rs)** - Basic benchmarks
   - Insert, query, delete performance
   - Hash function comparison
   - Slot finding performance
   - Multiset operations
   - Memory usage analysis

8. **[mappy-core/benches/simple_quotient_filter_benchmarks.rs](mappy-core/benches/simple_quotient_filter_benchmarks.rs)** - Simple benchmarks
   - Simplified benchmark suite
   - Focus on core operations
   - Performance validation

## Scripts and Automation

### Test and Benchmark Scripts

9. **[run_tests_and_benchmarks.sh](run_tests_and_benchmarks.sh)** - Comprehensive test runner
   - Run all tests and benchmarks
   - Performance analysis and stress testing
   - Error handling and reporting
   - Automated validation

10. **[test_advanced_quotient_filter_complete.sh](test_advanced_quotient_filter_complete.sh)** - Complete test suite
    - Rust and Python testing
    - Performance benchmarking
    - Memory usage analysis
    - Feature verification

### Python Test Scripts

11. **[test_python_advanced_quotient_filter.py](test_python_advanced_quotient_filter.py)** - Python test suite
    - Python bindings testing
    - Advanced features validation
    - Performance benchmarking
    - Concurrent operations testing
    - Error handling and edge cases

## Implementation Files

### Core Implementation

12. **[mappy-core/src/maplet.rs](mappy-core/src/maplet.rs)** - Core Maplet implementation
    - Maplet data structure
    - Advanced slot finding with `find_slot_for_key()`
    - Integration with quotient filter
    - Async operations and error handling

13. **[mappy-core/src/quotient_filter.rs](mappy-core/src/quotient_filter.rs)** - Quotient filter implementation
    - Core quotient filter logic
    - Advanced slot finding with run detection
    - Shifting support and linear probing
    - Hash function integration

14. **[mappy-core/src/engine.rs](mappy-core/src/engine.rs)** - Engine implementation
    - High-level Engine interface
    - Advanced slot finding integration
    - Async operations and error handling
    - Storage and persistence support

### Python Bindings

15. **[mappy-python/src/lib.rs](mappy-python/src/lib.rs)** - Python bindings
    - PyO3 integration for Python support
    - Advanced quotient filter Python methods
    - Error handling and type conversion
    - Async runtime integration

## Configuration Files

### Cargo Configuration

16. **[mappy-core/Cargo.toml](mappy-core/Cargo.toml)** - Core package configuration
    - Dependencies and features
    - Advanced quotient filter feature flag
    - Benchmark configuration
    - Metadata and documentation

17. **[mappy-python/Cargo.toml](mappy-python/Cargo.toml)** - Python bindings configuration
    - PyO3 dependencies
    - Python-specific features
    - Build configuration

## Key Features Documented

### Advanced Quotient Filter Features

- **Precise Slot Finding**: Locate exact storage slots considering runs and shifting
- **Run Detection**: Handle quotient filter runs with multiple fingerprints
- **Shifting Support**: Account for linear probing and slot shifting
- **Debugging Support**: Inspect internal storage layout for optimization
- **Performance Analysis**: Understand memory access patterns and cache behavior

### Comprehensive Testing

- **62+ Test Cases**: Covering all edge cases and scenarios
- **Basic Operations**: Insert, query, delete with various data types
- **False Positive Rate**: Validation of probabilistic accuracy
- **Multiset Operations**: Counter and aggregation operations
- **Run Detection**: Advanced slot finding with run handling
- **Capacity Management**: Load factor and resizing behavior
- **Concurrency**: Thread-safe operations and race condition testing
- **Edge Cases**: Boundary conditions and error scenarios
- **Hash Functions**: AHash, TwoX, Fnv performance comparison
- **Memory Usage**: Space efficiency and memory optimization
- **Advanced Features**: Slot finding, run detection, shifting support

### Performance Benchmarks

- **Insert Performance**: 10-17 million operations/second
- **Query Performance**: 16-45 million operations/second
- **Delete Performance**: 4-8 million operations/second
- **Slot Finding**: 24-61 million operations/second
- **Hash Function Comparison**: AHash (fastest), TwoX (medium), Fnv (slowest)
- **Memory Usage**: Linear scaling with efficient space utilization
- **Load Factor Impact**: Performance analysis across different load factors

### Python Integration

- **Slot Finding**: `find_slot_for_key()` method for both Maplet and Engine
- **Error Handling**: Proper Python exceptions for invalid operations
- **Performance**: Same high-performance as Rust implementation
- **Concurrency**: Thread-safe operations in Python
- **Memory Management**: Automatic cleanup and resource management

## Quick Start Guide

### For Users

1. **Read [README.md](README.md)** for project overview and quick start
2. **Check [ADVANCED_QUOTIENT_FILTER.md](ADVANCED_QUOTIENT_FILTER.md)** for advanced features
3. **Run tests**: `./run_tests_and_benchmarks.sh`
4. **Run benchmarks**: `cargo bench --bench basic_quotient_filter_benchmarks`

### For Developers

1. **Read [TECHNICAL_README.md](TECHNICAL_README.md)** for technical details
2. **Review test files** for implementation examples
3. **Check benchmark results** for performance characteristics
4. **Run comprehensive tests**: `./test_advanced_quotient_filter_complete.sh`

### For Python Users

1. **Read Python sections** in [ADVANCED_QUOTIENT_FILTER.md](ADVANCED_QUOTIENT_FILTER.md)
2. **Run Python tests**: `python3 test_python_advanced_quotient_filter.py`
3. **Check Python bindings** in [mappy-python/src/lib.rs](mappy-python/src/lib.rs)

## Performance Summary

### Recent Benchmark Results

| Operation    | Dataset Size  | Performance | Throughput    |
| ------------ | ------------- | ----------- | ------------- |
| Insert       | 1,000 items   | 60.5 µs     | 16.5M ops/sec |
| Insert       | 10,000 items  | 565 µs      | 17.7M ops/sec |
| Insert       | 100,000 items | 9.4 ms      | 10.6M ops/sec |
| Query        | 1,000 items   | 22.2 µs     | 45.0M ops/sec |
| Query        | 10,000 items  | 274 µs      | 36.5M ops/sec |
| Query        | 100,000 items | 6.1 ms      | 16.4M ops/sec |
| Delete       | 1,000 items   | 117 µs      | 8.5M ops/sec  |
| Delete       | 10,000 items  | 1.19 ms     | 8.4M ops/sec  |
| Delete       | 100,000 items | 24.7 ms     | 4.0M ops/sec  |
| Slot Finding | 1,000 items   | 16.3 µs     | 61.5M ops/sec |
| Slot Finding | 10,000 items  | 201 µs      | 49.7M ops/sec |
| Slot Finding | 100,000 items | 4.1 ms      | 24.5M ops/sec |

### Memory Usage

| Dataset Size    | Memory Usage | Efficiency   |
| --------------- | ------------ | ------------ |
| 1,000 items     | ~8KB         | 8 bytes/item |
| 10,000 items    | ~80KB        | 8 bytes/item |
| 100,000 items   | ~800KB       | 8 bytes/item |
| 1,000,000 items | ~8MB         | 8 bytes/item |

## Contributing

### Adding Documentation

1. **Update relevant documentation files**
2. **Add new test cases** to appropriate test files
3. **Update benchmark results** in benchmark documentation
4. **Update this index** when adding new documentation

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
- **API Documentation**: [https://docs.rs/mappy-core](https://docs.rs/mappy-core)
- **Quotient Filters**: Pandey et al. (2017b) - "A general-purpose counting filter: Making every bit count"
- **Cuckoo Filters**: Fan et al. (2014) - "Cuckoo Filter: Practically Better Than Bloom"
