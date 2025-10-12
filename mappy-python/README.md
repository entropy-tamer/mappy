# mappy-python

[![Crates.io](https://img.shields.io/crates/v/mappy-python.svg)](https://crates.io/crates/mappy-python)
[![PyPI](https://img.shields.io/pypi/v/mappy-python.svg)](https://pypi.org/project/mappy-python/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Python bindings for mappy maplet data structures** - Space-efficient approximate key-value mappings for Python.

## Overview

mappy-python provides Python bindings for the mappy-core Rust library, bringing high-performance, space-efficient maplet data structures to Python applications. Built with PyO3 for seamless integration and optimal performance.

## Key Features

- **High Performance**: Rust-powered core with Python convenience
- **Space Efficient**: Achieves `O(log 1/ε + v)` bits per item
- **Value Support**: Native key-value associations with configurable merge operators
- **One-Sided Errors**: Guaranteed error bounds for approximate queries
- **Deletion Support**: Full support for removing key-value pairs
- **Thread Safe**: Safe concurrent access from multiple Python threads
- **NumPy Integration**: Optional NumPy array support for numerical data
- **Async Support**: Compatible with asyncio for async applications

## Installation

### From PyPI (when available)

```bash
pip install mappy-python
```

### From Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python dependencies
pip install maturin

# Build and install
maturin develop --release
```

## Quick Start

```python
import mappy_python as mappy

# Create a maplet for counting with 1% false-positive rate
maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

# Insert key-value pairs
maplet.insert("user:123", 42)
maplet.insert("user:456", 17)
maplet.insert("user:789", 8)

# Query values (may return approximate results)
count = maplet.query("user:123")
print(f"User 123 count: {count}")  # 42 or 42 + other_values

# Check if key exists
exists = maplet.contains("user:123")
print(f"User 123 exists: {exists}")  # True

# Get statistics
stats = maplet.stats()
print(f"Load factor: {stats.load_factor * 100:.2f}%")
print(f"Memory usage: {stats.memory_usage} bytes")
```

## Available Merge Operators

```python
import mappy_python as mappy

# Counter operator - adds values together
counter_maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

# Max operator - takes maximum value
max_maplet = mappy.Maplet(1000, 0.01, mappy.MaxOperator())

# Min operator - takes minimum value
min_maplet = mappy.Maplet(1000, 0.01, mappy.MinOperator())

# Set operator - union of sets
set_maplet = mappy.Maplet(1000, 0.01, mappy.SetOperator())

# Custom operator - define your own merge function
def custom_merge(a, b):
    return a + b * 2  # Custom merge logic

custom_maplet = mappy.Maplet(1000, 0.01, mappy.CustomOperator(custom_merge))
```

## Use Cases

### 1. K-mer Counting (Bioinformatics)

```python
import mappy_python as mappy

# Create k-mer counter
kmer_counter = mappy.Maplet(1_000_000, 0.001, mappy.CounterOperator())

# Count k-mers in DNA sequences
def count_kmers(sequence, k=3):
    for i in range(len(sequence) - k + 1):
        kmer = sequence[i:i+k]
        kmer_counter.insert(kmer, 1)

# Example usage
dna_sequence = "ATCGATCGATCG"
count_kmers(dna_sequence)

# Query k-mer counts
atc_count = kmer_counter.query("ATC")
print(f"ATC appears approximately {atc_count} times")
```

### 2. Network Traffic Analysis

```python
import mappy_python as mappy
from collections import defaultdict

# Create traffic counter
traffic_counter = mappy.Maplet(100_000, 0.01, mappy.CounterOperator())

# Count bytes per IP address
def log_traffic(ip_address, bytes_transferred):
    traffic_counter.insert(ip_address, bytes_transferred)

# Example traffic data
traffic_data = [
    ("192.168.1.1", 1024),
    ("192.168.1.2", 2048),
    ("192.168.1.1", 512),
    ("10.0.0.1", 4096),
]

for ip, bytes_count in traffic_data:
    log_traffic(ip, bytes_count)

# Query traffic statistics
ip1_traffic = traffic_counter.query("192.168.1.1")
print(f"192.168.1.1 traffic: {ip1_traffic} bytes")
```

### 3. Distributed Caching

```python
import mappy_python as mappy
import asyncio
import aiohttp

class DistributedCache:
    def __init__(self, servers):
        self.servers = servers
        self.local_cache = mappy.Maplet(10_000, 0.01, mappy.MaxOperator())
    
    async def get(self, key):
        # Check local cache first
        local_value = self.local_cache.query(key)
        if local_value is not None:
            return local_value
        
        # Fetch from remote servers
        async with aiohttp.ClientSession() as session:
            for server in self.servers:
                try:
                    async with session.get(f"{server}/cache/{key}") as response:
                        if response.status == 200:
                            value = await response.json()
                            self.local_cache.insert(key, value)
                            return value
                except Exception:
                    continue
        
        return None
    
    async def set(self, key, value):
        # Update local cache
        self.local_cache.insert(key, value)
        
        # Propagate to remote servers
        async with aiohttp.ClientSession() as session:
            tasks = []
            for server in self.servers:
                task = session.post(f"{server}/cache/{key}", json=value)
                tasks.append(task)
            
            await asyncio.gather(*tasks, return_exceptions=True)

# Usage
cache = DistributedCache(["http://server1:8080", "http://server2:8080"])
await cache.set("user:123", {"name": "John", "age": 30})
user_data = await cache.get("user:123")
```

## NumPy Integration

```python
import mappy_python as mappy
import numpy as np

# Create maplet with NumPy support
vector_maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

# Store NumPy arrays
vector1 = np.array([1.0, 2.0, 3.0])
vector2 = np.array([4.0, 5.0, 6.0])

vector_maplet.insert("vec1", vector1)
vector_maplet.insert("vec2", vector2)

# Query vectors
result = vector_maplet.query("vec1")
print(f"Retrieved vector: {result}")
```

## Thread Safety

```python
import mappy_python as mappy
import threading
import time

# Create thread-safe maplet
shared_maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

def worker(worker_id):
    for i in range(100):
        key = f"worker_{worker_id}_item_{i}"
        shared_maplet.insert(key, 1)
        time.sleep(0.001)  # Simulate work

# Create multiple threads
threads = []
for i in range(4):
    thread = threading.Thread(target=worker, args=(i,))
    threads.append(thread)
    thread.start()

# Wait for all threads to complete
for thread in threads:
    thread.join()

# Verify results
total_items = sum(shared_maplet.query(f"worker_{i}_item_{j}") 
                 for i in range(4) for j in range(100))
print(f"Total items processed: {total_items}")
```

## Performance Optimization

### Batch Operations

```python
import mappy_python as mappy

# Create maplet
maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

# Batch insert for better performance
batch_data = [(f"key_{i}", i) for i in range(1000)]
maplet.batch_insert(batch_data)

# Batch query
keys = [f"key_{i}" for i in range(100)]
results = maplet.batch_query(keys)
```

### Memory Management

```python
import mappy_python as mappy

# Create maplet with specific memory constraints
maplet = mappy.Maplet(
    expected_items=100_000,
    false_positive_rate=0.001,
    operator=mappy.CounterOperator(),
    memory_limit_mb=100  # Limit memory usage
)

# Monitor memory usage
stats = maplet.stats()
print(f"Memory usage: {stats.memory_usage / 1024 / 1024:.2f} MB")
print(f"Load factor: {stats.load_factor * 100:.2f}%")
```

## Error Handling

```python
import mappy_python as mappy

try:
    maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())
    
    # Insert data
    maplet.insert("key1", 42)
    
    # Query data
    result = maplet.query("key1")
    print(f"Result: {result}")
    
except mappy.MapletError as e:
    print(f"Maplet error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Benchmarking

```python
import mappy_python as mappy
import time
import random

def benchmark_maplet():
    # Create maplet
    maplet = mappy.Maplet(100_000, 0.01, mappy.CounterOperator())
    
    # Benchmark inserts
    start_time = time.time()
    for i in range(100_000):
        key = f"key_{i}"
        value = random.randint(1, 100)
        maplet.insert(key, value)
    insert_time = time.time() - start_time
    
    # Benchmark queries
    start_time = time.time()
    for i in range(100_000):
        key = f"key_{i}"
        maplet.query(key)
    query_time = time.time() - start_time
    
    print(f"Insert time: {insert_time:.2f}s")
    print(f"Query time: {query_time:.2f}s")
    print(f"Insert rate: {100_000 / insert_time:.0f} ops/sec")
    print(f"Query rate: {100_000 / query_time:.0f} ops/sec")

benchmark_maplet()
```

## Examples

See the `examples/` directory for comprehensive usage examples:

- `basic_usage.py` - Basic maplet operations
- `kmer_counting.py` - Bioinformatics k-mer counting
- `distributed_cache.py` - Distributed caching system
- `numpy_integration.py` - NumPy array operations
- `threading_example.py` - Multi-threaded usage

## Testing

```bash
# Run Python tests
python -m pytest tests/

# Run benchmarks
python benchmarks/benchmark.py

# Run integration tests
python tests/test_integration.py
```

## Documentation

- **[API Documentation](https://docs.rs/mappy-python)** - Complete API reference
- **[Core Library](https://crates.io/crates/mappy-core)** - Underlying Rust implementation
- **[Main Repository](https://github.com/entropy-tamer/mappy)** - Source code and examples

## License

MIT License - see [LICENSE](../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/entropy-tamer/mappy) for contribution guidelines.

## Requirements

- Python 3.7+
- Rust 1.70+ (for building from source)
- NumPy (optional, for array operations)
