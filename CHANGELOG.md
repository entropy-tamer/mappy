# Changelog

All notable changes to the Mappy project will be documented in this file.

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.2.1] - 2025-11-01 - Version Bump and Testing Improvements

### Changed

#### Version and Metadata Updates

- **Version bump**: Updated from 0.2.0 to 0.2.1 across all Rust and Python packages
- **Author metadata**: Updated package author from "Reynard Team" to "Balazs Horvath"
- **Python package name**: Changed from `reynard-mappy` to `mappy-python` for consistency

#### Dependency Updates

- **proptest**: Updated from 1.8.0 to 1.9.0

#### Documentation

- **TECHNICAL_README.md**: Added comprehensive documentation on Engine persistence behavior
  - Documented maplet-first design for lookups
  - Explained implications for data persistence across engine restarts
  - Provided design rationale and future enhancement suggestions

#### Testing Infrastructure

- **Test improvements**: Enhanced disk persistence tests with unique subdirectories to avoid lock conflicts
- **New pytest suite**: Added comprehensive `test_mappy_python.py` with pytest-based test structure
- **Testing documentation**: Created `TESTING.md` guide for Python bindings testing
- **Test cleanup**: Removed legacy shell-based test scripts (`run_tests_and_benchmarks.sh`, `test_quotient_filter_complete.sh`)
- **pytest configuration**: Added `pytest.ini` for standardized test configuration

#### Python Bindings

- **README updates**: Enhanced Python bindings documentation with improved examples and usage patterns

### Technical Details

#### Files Modified

- `Cargo.toml` - Version and author metadata updates
- `mappy-python/pyproject.toml` - Package name, version, and author updates
- `mappy-core/src/engine.rs` - Test improvements for disk persistence
- `TECHNICAL_README.md` - Engine persistence behavior documentation
- `mappy-python/README.md` - Enhanced documentation

#### Files Added

- `TESTING.md` - Comprehensive testing guide
- `pytest.ini` - Pytest configuration
- `test_mappy_python.py` - Modern pytest-based test suite

#### Files Removed

- `run_tests_and_benchmarks.sh` - Legacy test script
- `test_quotient_filter_complete.sh` - Legacy test script

### Migration Notes

- **Python package name change**: If you're using the Python package, update imports if needed. The package functionality remains the same.
- **Testing**: New pytest-based test suite is available. Legacy test scripts have been removed in favor of modern pytest infrastructure.

## [0.2.0] - 2025-10-23 - Advanced Quotient Filter Implementation

### Added

#### Advanced Quotient Filter Features

- **Precise Slot Finding**: `find_slot_for_key()` method for both Maplet and Engine
- **Run Detection**: Handle quotient filter runs with multiple fingerprints
- **Shifting Support**: Account for linear probing and slot shifting
- **Debugging Support**: Inspect internal storage layout for optimization
- **Performance Analysis**: Understand memory access patterns and cache behavior

#### Comprehensive Testing Infrastructure

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

#### Performance Benchmarks

- **Insert Performance**: 10-17 million operations/second
- **Query Performance**: 16-45 million operations/second
- **Delete Performance**: 4-8 million operations/second
- **Slot Finding**: 24-61 million operations/second
- **Hash Function Comparison**: AHash (fastest), TwoX (medium), Fnv (slowest)
- **Memory Usage**: Linear scaling with efficient space utilization
- **Load Factor Impact**: Performance analysis across different load factors

#### Python Integration

- **Python Bindings**: Full Python bindings for advanced features
- **Slot Finding**: `find_slot_for_key()` method for both PyMaplet and PyEngine
- **Error Handling**: Proper Python exceptions for invalid operations
- **Performance**: Same high-performance as Rust implementation
- **Concurrency**: Thread-safe operations in Python
- **Memory Management**: Automatic cleanup and resource management

#### Documentation

- **Advanced Quotient Filter Guide**: Complete documentation for advanced features
- **Comprehensive Testing Guide**: Detailed testing and benchmarking documentation
- **Python Integration Guide**: Python bindings and usage examples
- **Performance Analysis**: Detailed benchmark results and performance characteristics
- **Documentation Index**: Comprehensive index of all documentation and resources

### Changed

#### Core Implementation

- **Maplet Integration**: Enhanced Maplet with advanced slot finding capabilities
- **Engine Integration**: Engine now supports advanced quotient filter features
- **Quotient Filter**: Enhanced with run detection and shifting support
- **Python Bindings**: Updated with advanced quotient filter methods

#### Testing Infrastructure

- **Test Coverage**: Expanded from basic tests to 62+ comprehensive test cases
- **Benchmark Suite**: Added detailed performance benchmarks
- **Python Tests**: Comprehensive Python test suite
- **Automation**: Automated test and benchmark runners

#### Documentation

- **README.md**: Updated with advanced features and comprehensive testing
- **TECHNICAL_README.md**: Enhanced with advanced quotient filter technical details
- **Benchmark Documentation**: Detailed benchmark results and performance analysis
- **API Documentation**: Updated with new methods and features

### Performance Improvements

#### Benchmark Results

- **Insert Operations**: 16.5M ops/sec (1K items), 17.7M ops/sec (10K items), 10.6M ops/sec (100K items)
- **Query Operations**: 45.0M ops/sec (1K items), 36.5M ops/sec (10K items), 16.4M ops/sec (100K items)
- **Delete Operations**: 8.5M ops/sec (1K items), 8.4M ops/sec (10K items), 4.0M ops/sec (100K items)
- **Slot Finding**: 61.5M ops/sec (1K items), 49.7M ops/sec (10K items), 24.5M ops/sec (100K items)

#### Memory Efficiency

- **Memory Usage**: 8 bytes/item across all dataset sizes
- **Linear Scaling**: Efficient memory usage with dataset growth
- **Cache Performance**: Optimized for sequential access patterns

### Technical Details

#### New Files Added

- `mappy-core/src/quotient_filter_tests.rs` - 62+ comprehensive test cases
- `mappy-core/src/advanced_quotient_filter_tests.rs` - 12+ advanced test cases
- `mappy-core/benches/basic_quotient_filter_benchmarks.rs` - Performance benchmarks
- `mappy-core/benches/simple_quotient_filter_benchmarks.rs` - Simple benchmarks
- `test_python_advanced_quotient_filter.py` - Python test suite
- `test_advanced_quotient_filter_complete.sh` - Complete test automation
- `run_tests_and_benchmarks.sh` - Comprehensive test runner
- `ADVANCED_QUOTIENT_FILTER.md` - Advanced features documentation
- `DOCUMENTATION_INDEX.md` - Documentation index

#### Enhanced Files

- `mappy-core/src/maplet.rs` - Added `find_slot_for_key()` method
- `mappy-core/src/engine.rs` - Added `find_slot_for_key()` method
- `mappy-core/src/quotient_filter.rs` - Enhanced with advanced slot finding
- `mappy-python/src/lib.rs` - Added Python bindings for advanced features
- `README.md` - Updated with advanced features and comprehensive testing
- `TECHNICAL_README.md` - Enhanced with advanced quotient filter details
- `mappy-core/benches/QUOTIENT_FILTER_BENCHMARKS.md` - Updated with recent results

#### Configuration Updates

- `mappy-core/Cargo.toml` - Added `advanced-quotient-filter` feature
- `mappy-python/Cargo.toml` - Updated with advanced features support

### Breaking Changes

None. All changes are additive and backward compatible.

### Migration Guide

No migration required. The advanced quotient filter features are opt-in via the `advanced-quotient-filter` feature flag.

### Usage Examples

#### Rust Usage

```rust
// Enable advanced features
cargo run --features advanced-quotient-filter

// Use slot finding
let slot = maplet.find_slot_for_key("my_key").await;
let engine_slot = engine.find_slot_for_key("my_key").await;
```

#### Python Usage

```python
import mappy_python

# Create Maplet with advanced features
maplet = mappy_python.PyMaplet(capacity=1000, false_positive_rate=0.01)
maplet.insert("key", 42)
slot = maplet.find_slot_for_key("key")  # Returns slot index

# Create Engine with advanced features
config = mappy_python.PyEngineConfig(capacity=1000, false_positive_rate=0.01)
engine = mappy_python.PyEngine(config)
engine.set("key", b"value")
slot = engine.find_slot_for_key("key")  # Returns slot index
```

### Testing

#### Running Tests

```bash
# Run all tests with advanced features
cargo test --features advanced-quotient-filter

# Run comprehensive test suite
./run_tests_and_benchmarks.sh

# Run Python tests
python3 test_python_advanced_quotient_filter.py

# Run complete test suite
./test_advanced_quotient_filter_complete.sh
```

#### Running Benchmarks

```bash
# Run basic benchmarks
cargo bench --bench basic_quotient_filter_benchmarks

# Run with specific sample size
cargo bench --bench basic_quotient_filter_benchmarks -- --sample-size 10
```

### Contributors

- **Advanced Quotient Filter Implementation**: Complete implementation with run detection and shifting support
- **Comprehensive Testing**: 62+ test cases covering all edge cases and scenarios
- **Performance Benchmarks**: Detailed benchmarks showing 10-60M operations/second
- **Python Integration**: Full Python bindings for advanced features
- **Documentation**: Comprehensive documentation for all new features

### Acknowledgments

- **Research Foundation**: Based on "Time To Replace Your Filter: How Maplets Simplify System Design"
- **Quotient Filter Research**: Pandey et al. (2017b) - "A general-purpose counting filter: Making every bit count"
- **Cuckoo Filter Research**: Fan et al. (2014) - "Cuckoo Filter: Practically Better Than Bloom"
- **Community**: Rust and Python communities for excellent tooling and support
