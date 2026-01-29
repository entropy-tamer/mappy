# Testing Guide for Mappy Python Bindings

This directory contains comprehensive test suites for the mappy-python bindings.

## Quick Start

```bash
# Run all tests with pytest
pytest test_mappy_python.py -v

# Run specific test classes
pytest test_mappy_python.py::TestPyMaplet -v

# Run tests with coverage
pytest test_mappy_python.py --cov=mappy_python --cov-report=html
```

## Test Structure

### `test_mappy_python.py` - Comprehensive Pytest Suite

The main test suite with complete coverage of all mappy-python functionalities:

- **TestPyMaplet**: Tests for PyMaplet class
  - Creation and initialization
  - Insert, query, contains operations
  - Length and emptiness checks
  - Error rate and load factor
  - Slot finding (quotient filter features)
  - Unicode and edge case handling

- **TestPyEngineConfig**: Tests for configuration
  - Default and custom configurations
  - All persistence modes
  - Property modification
  - Invalid configurations

- **TestPyEngine**: Tests for PyEngine class
  - Basic CRUD operations
  - TTL (Time-To-Live) operations
  - Statistics and monitoring
  - Memory usage
  - Persistence modes
  - Slot finding (quotient filter features)

- **TestPersistence**: Tests for persistence features
  - Disk persistence (with note on maplet-first design behavior)
  - AOF persistence
  - Hybrid persistence

- **TestTTL**: Tests for TTL features
  - Expiration handling
  - TTL extension
  - Cleanup intervals

- **TestConcurrency**: Tests for concurrent operations
  - Thread-safe insertions
  - Concurrent engine operations

- **TestPerformance**: Performance benchmarks
  - Insertion performance
  - Query performance
  - Slot finding performance

- **TestEdgeCases**: Edge cases and error handling
  - Invalid parameters
  - Special characters
  - Large values
  - Operations after close

## Running Tests

### Prerequisites

1. Build the Python bindings:

   ```bash
   cd mappy-python
   maturin develop
   ```

2. Install test dependencies:

   ```bash
   pip install pytest pytest-asyncio pytest-cov
   ```

### Basic Usage

```bash
# Run all tests
pytest test_mappy_python.py -v

# Run with output
pytest test_mappy_python.py -v -s

# Run specific test
pytest test_mappy_python.py::TestPyMaplet::test_maplet_insert -v

# Run tests matching a pattern
pytest test_mappy_python.py -k "maplet" -v

# Run tests with markers
pytest test_mappy_python.py -m "performance" -v
```

### Test Markers

The test suite uses pytest markers for organization:

- `@pytest.mark.unit` - Unit tests
- `@pytest.mark.integration` - Integration tests
- `@pytest.mark.performance` - Performance benchmarks
- `@pytest.mark.concurrency` - Concurrent operation tests
- `@pytest.mark.persistence` - Persistence-related tests
- `@pytest.mark.ttl` - Time-to-live related tests

Run tests by marker:

```bash
pytest test_mappy_python.py -m performance -v
```

### Coverage Reports

Generate coverage reports:

```bash
# Terminal report
pytest test_mappy_python.py --cov=mappy_python --cov-report=term

# HTML report
pytest test_mappy_python.py --cov=mappy_python --cov-report=html
# Open htmlcov/index.html in a browser

# XML report (for CI)
pytest test_mappy_python.py --cov=mappy_python --cov-report=xml
```

### Parallel Execution

Run tests in parallel for faster execution:

```bash
# Install pytest-xdist
pip install pytest-xdist

# Run tests in parallel
pytest test_mappy_python.py -n auto
```

## Test Fixtures

The test suite provides reusable fixtures:

- `basic_maplet`: Empty PyMaplet instance
- `populated_maplet`: PyMaplet with test data
- `memory_engine_config`: Engine config with memory persistence
- `disk_engine_config`: Engine config with disk persistence
- `hybrid_engine_config`: Engine config with hybrid persistence
- `memory_engine`: Engine instance with memory persistence
- `populated_engine`: Engine instance with test data
- `temp_dir`: Temporary directory for test data

## Writing New Tests

When adding new tests:

1. Use descriptive test names: `test_<feature>_<scenario>`
2. Use fixtures for setup/teardown
3. Add appropriate assertions
4. Test both success and failure cases
5. Use markers to categorize tests
6. Add docstrings explaining what is tested

Example:

```python
def test_maplet_new_feature(basic_maplet):
    """Test new feature of PyMaplet."""
    # Arrange
    basic_maplet.insert("test", 42)

    # Act
    result = basic_maplet.new_feature("test")

    # Assert
    assert result == expected_value
```

## Continuous Integration

Tests can be run in CI environments:

```bash
# CI-friendly output
pytest test_mappy_python.py --tb=short --junitxml=test-results.xml

# With coverage for CI
pytest test_mappy_python.py --cov=mappy_python --cov-report=xml --cov-report=term
```

## Troubleshooting

### Import Errors

If you get `ImportError: No module named 'mappy_python'`:

```bash
cd mappy-python
maturin develop
```

### Test Failures

1. Check that the bindings are built correctly
2. Verify Python version compatibility (3.8+)
3. Ensure all dependencies are installed
4. Run tests with `-v` flag for verbose output
5. Check test output for specific error messages

### Performance Test Failures

Performance tests may fail on slower systems. Adjust timeouts in `TestPerformance` class if needed.

## Contributing

When contributing tests:

1. Follow existing test patterns
2. Maintain test coverage
3. Add tests for new features
4. Update this documentation if needed
5. Run all tests before submitting PR

## Test Statistics

Current test coverage:

- **PyMaplet**: ~20 tests
- **PyEngine**: ~30 tests
- **PyEngineConfig**: ~10 tests
- **Persistence**: ~5 tests
- **TTL**: ~5 tests
- **Concurrency**: ~2 tests
- **Performance**: ~5 tests
- **Edge Cases**: ~10 tests

**Total**: ~90+ comprehensive tests
