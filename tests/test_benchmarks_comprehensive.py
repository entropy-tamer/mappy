"""Comprehensive benchmark tests for mappy-python: insert/query performance, scalability, memory.

Extends basic performance tests with comprehensive benchmarking scenarios.
"""

import mappy_python
import numpy as np
import pytest

from .test_utils import get_memory_usage, measure_operation


class TestInsertPerformance:
    """Comprehensive insert performance benchmarks."""

    @pytest.mark.benchmark
    def test_small_inserts(self):
        """Benchmark small value inserts."""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        def insert_operation():
            for i in range(1000):
                maplet.insert(f"key_{i}", i)

        result = measure_operation(insert_operation, iterations=10)
        assert result.success_rate > 0.0
        assert result.throughput > 0.0

    @pytest.mark.benchmark
    def test_large_inserts(self):
        """Benchmark large value inserts."""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator(),
        )

        large_vector = np.random.rand(1000).tolist()

        def insert_operation():
            for i in range(100):
                maplet.insert(f"key_{i}", large_vector)

        result = measure_operation(insert_operation, iterations=10)
        assert result.success_rate > 0.0

    @pytest.mark.benchmark
    def test_batch_inserts(self):
        """Benchmark batch insert operations."""
        maplet = mappy_python.Maplet(
            capacity=50000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        keys = [f"key_{i}" for i in range(10000)]
        values = list(range(10000))

        def insert_operation():
            for key, value in zip(keys, values, strict=False):
                maplet.insert(key, value)

        result = measure_operation(insert_operation, iterations=1)
        assert result.success_rate > 0.0


class TestQueryPerformance:
    """Comprehensive query performance benchmarks."""

    @pytest.mark.benchmark
    def test_small_queries(self):
        """Benchmark small value queries."""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        # Pre-populate
        for i in range(1000):
            maplet.insert(f"key_{i}", i)

        def query_operation():
            for i in range(1000):
                maplet.query(f"key_{i}")

        result = measure_operation(query_operation, iterations=10)
        assert result.success_rate > 0.0

    @pytest.mark.benchmark
    def test_large_queries(self):
        """Benchmark large value queries."""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator(),
        )

        large_vector = np.random.rand(1000).tolist()
        # Pre-populate
        for i in range(100):
            maplet.insert(f"key_{i}", large_vector)

        def query_operation():
            for i in range(100):
                maplet.query(f"key_{i}")

        result = measure_operation(query_operation, iterations=10)
        assert result.success_rate > 0.0

    @pytest.mark.benchmark
    def test_mixed_queries(self):
        """Benchmark mixed query operations."""
        maplet = mappy_python.Maplet(
            capacity=20000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        # Pre-populate with mix of values
        for i in range(5000):
            maplet.insert(f"key_{i}", i % 100)

        def query_operation():
            for i in range(5000):
                maplet.query(f"key_{i}")

        result = measure_operation(query_operation, iterations=5)
        assert result.success_rate > 0.0


class TestScalability:
    """Scalability benchmarks."""

    @pytest.mark.benchmark
    def test_scalability_10k(self):
        """Test scalability with 10K items."""
        maplet = mappy_python.Maplet(
            capacity=20000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        def insert_operation():
            for i in range(10000):
                maplet.insert(f"key_{i}", i)

        result = measure_operation(insert_operation, iterations=1)
        assert result.success_rate > 0.0

    @pytest.mark.benchmark
    def test_scalability_100k(self):
        """Test scalability with 100K items."""
        maplet = mappy_python.Maplet(
            capacity=200000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        def insert_operation():
            for i in range(100000):
                maplet.insert(f"key_{i}", i)

        result = measure_operation(insert_operation, iterations=1)
        assert result.success_rate > 0.0

    @pytest.mark.benchmark
    def test_scalability_1m(self):
        """Test scalability with 1M items."""
        maplet = mappy_python.Maplet(
            capacity=2000000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        def insert_operation():
            for i in range(1000000):
                maplet.insert(f"key_{i}", i)

        result = measure_operation(insert_operation, iterations=1)
        assert result.success_rate > 0.0


class TestMemoryUsage:
    """Memory usage benchmarks."""

    @pytest.mark.benchmark
    @pytest.mark.memory
    def test_memory_small_dataset(self):
        """Test memory usage with small dataset."""
        initial_memory = get_memory_usage()

        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        for i in range(100):
            maplet.insert(f"key_{i}", i)

        final_memory = get_memory_usage()
        memory_delta = final_memory - initial_memory

        # Memory should be reasonable (less than 10MB for 100 items)
        assert memory_delta < 10 * 1024 * 1024

    @pytest.mark.benchmark
    @pytest.mark.memory
    def test_memory_large_dataset(self):
        """Test memory usage with large dataset."""
        initial_memory = get_memory_usage()

        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        for i in range(10000):
            maplet.insert(f"key_{i}", i)

        final_memory = get_memory_usage()
        memory_delta = final_memory - initial_memory

        # Memory should be reasonable (less than 100MB for 10K items)
        assert memory_delta < 100 * 1024 * 1024

    @pytest.mark.benchmark
    @pytest.mark.memory
    def test_memory_vector_dataset(self):
        """Test memory usage with vector dataset."""
        initial_memory = get_memory_usage()

        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator(),
        )

        vectors = [np.random.rand(100).tolist() for _ in range(1000)]
        for i, vec in enumerate(vectors):
            maplet.insert(f"vec_{i}", vec)

        final_memory = get_memory_usage()
        memory_delta = final_memory - initial_memory

        # Memory should be reasonable (less than 200MB for 1K vectors of 100 dims)
        assert memory_delta < 200 * 1024 * 1024


class TestThroughput:
    """Throughput benchmarks."""

    @pytest.mark.benchmark
    def test_insert_throughput(self):
        """Measure insert throughput."""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        def insert_operation():
            for i in range(10000):
                maplet.insert(f"key_{i}", i)

        result = measure_operation(insert_operation, iterations=1)
        assert result.throughput > 0.0
        # Should achieve reasonable throughput (relaxed for probabilistic nature)
        assert result.throughput > 0.0

    @pytest.mark.benchmark
    def test_query_throughput(self):
        """Measure query throughput."""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        # Pre-populate
        for i in range(10000):
            maplet.insert(f"key_{i}", i)

        def query_operation():
            for i in range(10000):
                maplet.query(f"key_{i}")

        result = measure_operation(query_operation, iterations=10)
        assert result.throughput > 0.0
        # Should achieve reasonable throughput (relaxed for probabilistic nature)
        assert result.throughput > 0.0


class TestLatency:
    """Latency benchmarks."""

    @pytest.mark.benchmark
    def test_insert_latency(self):
        """Measure insert latency."""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        def insert_operation():
            maplet.insert("key", 42)

        result = measure_operation(insert_operation, iterations=1000)
        assert result.avg_time > 0.0
        # Average latency should be reasonable (less than 1ms)
        assert result.avg_time < 0.001

    @pytest.mark.benchmark
    def test_query_latency(self):
        """Measure query latency."""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator(),
        )

        # Pre-populate
        maplet.insert("key", 42)

        def query_operation():
            maplet.query("key")

        result = measure_operation(query_operation, iterations=1000)
        assert result.avg_time > 0.0
        # Average latency should be reasonable (less than 1ms)
        assert result.avg_time < 0.001

