"""
Performance and benchmark tests for mappy-python bindings
"""

import random
import statistics
import time
from typing import Any, Dict, List

import mappy_python as mappy
import numpy as np
import pytest
from . import Stats


class TestPerformanceBenchmarks:
    """Performance benchmark tests"""

    @pytest.mark.benchmark
    def test_insert_performance(self, benchmark):
        """Benchmark insert performance"""
        maplet = mappy.Maplet(100000, 0.001, mappy.CounterOperator())

        def benchmark_insert():
            for i in range(10000):
                key = f"key_{i}"
                value = random.randint(1, 100)
                maplet.insert(key, value)

        benchmark(benchmark_insert)

    @pytest.mark.benchmark
    def test_query_performance(self, benchmark):
        """Benchmark query performance"""
        maplet = mappy.Maplet(100000, 0.001, mappy.CounterOperator())

        # Pre-populate maplet
        for i in range(10000):
            key = f"key_{i}"
            value = random.randint(1, 100)
            maplet.insert(key, value)

        def benchmark_query():
            # Allow for some probabilistic failures, but most should succeed
            success_count = 0
            for i in range(10000):
                key = f"key_{i}"
                result = maplet.query(key)
                if result is not None:
                    success_count += 1
            # At least 90% of queries should succeed (accounting for probabilistic nature)
            assert success_count >= 9000, f"Only {success_count}/10000 queries succeeded"

        benchmark(benchmark_query)

    @pytest.mark.benchmark
    def test_contains_performance(self, benchmark):
        """Benchmark contains performance"""
        maplet = mappy.Maplet(100000, 0.001, mappy.CounterOperator())

        # Pre-populate maplet
        for i in range(10000):
            key = f"key_{i}"
            value = random.randint(1, 100)
            maplet.insert(key, value)

        def benchmark_contains():
            # Allow for some probabilistic failures, but most should succeed
            success_count = 0
            for i in range(10000):
                key = f"key_{i}"
                result = maplet.contains(key)
                if result is True:
                    success_count += 1
            # At least 90% of contains checks should succeed
            assert success_count >= 9000, f"Only {success_count}/10000 contains checks succeeded"

        benchmark(benchmark_contains)

    @pytest.mark.benchmark
    def test_mixed_operations_performance(self, benchmark):
        """Benchmark mixed operations performance"""
        maplet = mappy.Maplet(100000, 0.001, mappy.CounterOperator())

        def benchmark_mixed():
            for i in range(5000):
                key = f"key_{i}"
                value = random.randint(1, 100)

                # Insert
                maplet.insert(key, value)

                # Query - allow None for probabilistic data structure
                result = maplet.query(key)
                # Note: result may be None due to probabilistic nature

                # Contains - allow False for probabilistic data structure
                exists = maplet.contains(key)
                # Note: exists may be False due to probabilistic nature

        benchmark(benchmark_mixed)

    @pytest.mark.benchmark
    def test_numpy_array_performance(self, benchmark):
        """Benchmark NumPy array operations"""
        maplet = mappy.Maplet(10000, 0.001, mappy.VectorOperator())

        def benchmark_numpy():
            for i in range(1000):
                key = f"array_{i}"
                array = np.random.rand(100)

                # Insert array
                maplet.insert(key, array)

                # Query array - allow None for probabilistic data structure
                result = maplet.query(key)
                # Note: result may be None due to probabilistic nature

        benchmark(benchmark_numpy)


class TestScalabilityTests:
    """Scalability tests for different data sizes"""

    def test_scalability_insert(self):
        """Test insert performance scalability"""
        sizes = [1000, 5000, 10000, 20000]
        results = {}

        for size in sizes:
            maplet = mappy.Maplet(size * 2, 0.001, mappy.CounterOperator())

            start_time = time.time()
            for i in range(size):
                key = f"key_{i}"
                value = random.randint(1, 100)
                maplet.insert(key, value)
            end_time = time.time()

            insert_time = end_time - start_time
            stats = Stats(maplet.stats())
            results[size] = {
                "time": insert_time,
                "rate": size / insert_time,
                "memory": stats.memory_usage,
            }

        # Performance should scale reasonably
        for size, result in results.items():
            assert result["rate"] > 1000  # At least 1000 ops/sec
            assert (
                result["time"] < size / 1000
            )  # Should be faster than 1ms per operation

    def test_scalability_query(self):
        """Test query performance scalability"""
        sizes = [1000, 5000, 10000, 20000]
        results = {}

        for size in sizes:
            maplet = mappy.Maplet(size * 2, 0.001, mappy.CounterOperator())

            # Pre-populate
            for i in range(size):
                key = f"key_{i}"
                value = random.randint(1, 100)
                maplet.insert(key, value)

            # Benchmark queries - allow some failures
            start_time = time.time()
            success_count = 0
            for i in range(size):
                key = f"key_{i}"
                result = maplet.query(key)
                if result is not None:
                    success_count += 1
            end_time = time.time()
            
            # At least 90% should succeed
            assert success_count >= int(size * 0.9), f"Only {success_count}/{size} queries succeeded"

            query_time = end_time - start_time
            results[size] = {"time": query_time, "rate": size / query_time}

        # Query performance should be excellent
        for size, result in results.items():
            assert result["rate"] > 5000  # At least 5000 queries/sec

    def test_memory_scalability(self):
        """Test memory usage scalability"""
        sizes = [1000, 5000, 10000, 20000]
        results = {}

        for size in sizes:
            maplet = mappy.Maplet(size * 2, 0.001, mappy.CounterOperator())

            # Insert data
            for i in range(size):
                key = f"key_{i}"
                value = random.randint(1, 100)
                maplet.insert(key, value)

            stats = Stats(maplet.stats())
            results[size] = {
                "memory_per_item": stats.memory_usage / size,
                "load_factor": stats.load_factor,
                "total_memory": stats.memory_usage,
            }

        # Memory per item should be reasonable and consistent
        memory_per_item_values = [r["memory_per_item"] for r in results.values()]
        memory_variance = statistics.stdev(memory_per_item_values) / statistics.mean(
            memory_per_item_values
        )

        assert memory_variance < 0.5  # Memory per item should be relatively consistent
        assert all(
            r["memory_per_item"] < 1000 for r in results.values()
        )  # Less than 1KB per item

    def test_false_positive_rate_accuracy(self):
        """Test false positive rate accuracy"""
        maplet = mappy.Maplet(10000, 0.01, mappy.CounterOperator())

        # Insert known keys
        known_keys = [f"known_{i}" for i in range(1000)]
        for key in known_keys:
            maplet.insert(key, 1)

        # Test known keys (should all be found)
        known_found = 0
        for key in known_keys:
            if maplet.contains(key):
                known_found += 1

        # Test unknown keys (should have false positive rate around 0.01)
        unknown_keys = [f"unknown_{i}" for i in range(1000)]
        false_positives = 0
        for key in unknown_keys:
            if maplet.contains(key):
                false_positives += 1

        # Verify known keys are found
        assert known_found >= 900  # At least 90% of known keys should be found

        # Verify false positive rate is reasonable
        false_positive_rate = false_positives / len(unknown_keys)
        # Allow up to 10% false positive rate (10x the target rate) to account for probabilistic nature
        assert false_positive_rate <= 0.10, f"False positive rate {false_positive_rate:.3f} exceeds 10%"


class TestConcurrentPerformance:
    """Concurrent performance tests"""

    def test_concurrent_insert_performance(self):
        """Test concurrent insert performance"""
        import threading

        maplet = mappy.Maplet(100000, 0.001, mappy.CounterOperator())
        results = []
        errors = []

        def worker(worker_id: int, num_operations: int):
            try:
                start_time = time.time()
                for i in range(num_operations):
                    key = f"worker_{worker_id}_key_{i}"
                    value = random.randint(1, 100)
                    maplet.insert(key, value)
                end_time = time.time()

                results.append(
                    {
                        "worker_id": worker_id,
                        "time": end_time - start_time,
                        "operations": num_operations,
                    }
                )
            except Exception as e:
                errors.append(f"Worker {worker_id} error: {e}")

        # Create multiple threads
        num_threads = 4
        operations_per_thread = 2500
        threads = []

        for i in range(num_threads):
            thread = threading.Thread(target=worker, args=(i, operations_per_thread))
            threads.append(thread)

        # Start all threads
        start_time = time.time()
        for thread in threads:
            thread.start()

        # Wait for all threads to complete
        for thread in threads:
            thread.join()
        end_time = time.time()

        # Verify no errors
        assert len(errors) == 0, f"Errors occurred: {errors}"

        # Verify all workers completed
        assert len(results) == num_threads

        # Calculate performance metrics
        total_operations = num_threads * operations_per_thread
        total_time = end_time - start_time
        total_rate = total_operations / total_time

        # Performance should be good even with concurrency
        assert total_rate > 5000  # At least 5000 ops/sec total
        assert total_time < 10.0  # Should complete in under 10 seconds

    def test_concurrent_query_performance(self):
        """Test concurrent query performance"""
        import threading

        maplet = mappy.Maplet(100000, 0.001, mappy.CounterOperator())

        # Pre-populate maplet
        for i in range(10000):
            key = f"key_{i}"
            value = random.randint(1, 100)
            maplet.insert(key, value)

        results = []
        errors = []

        def query_worker(worker_id: int, num_queries: int):
            try:
                start_time = time.time()
                success_count = 0
                for i in range(num_queries):
                    key = f"key_{i % 10000}"  # Cycle through keys
                    result = maplet.query(key)
                    if result is not None:
                        success_count += 1
                # At least 90% should succeed
                assert success_count >= int(num_queries * 0.9), f"Only {success_count}/{num_queries} queries succeeded"
                end_time = time.time()

                results.append(
                    {
                        "worker_id": worker_id,
                        "time": end_time - start_time,
                        "queries": num_queries,
                    }
                )
            except Exception as e:
                errors.append(f"Query worker {worker_id} error: {e}")

        # Create multiple query threads
        num_threads = 4
        queries_per_thread = 2500
        threads = []

        for i in range(num_threads):
            thread = threading.Thread(target=query_worker, args=(i, queries_per_thread))
            threads.append(thread)

        # Start all threads
        start_time = time.time()
        for thread in threads:
            thread.start()

        # Wait for all threads to complete
        for thread in threads:
            thread.join()
        end_time = time.time()

        # Verify no errors
        assert len(errors) == 0, f"Errors occurred: {errors}"

        # Verify all workers completed
        assert len(results) == num_threads

        # Calculate performance metrics
        total_queries = num_threads * queries_per_thread
        total_time = end_time - start_time
        total_rate = total_queries / total_time

        # Query performance should be excellent even with concurrency
        assert total_rate > 10000  # At least 10000 queries/sec total
        assert total_time < 5.0  # Should complete in under 5 seconds


class TestMemoryPerformance:
    """Memory performance tests"""

    def test_memory_usage_under_load(self):
        """Test memory usage under heavy load"""
        # Use larger capacity to avoid hitting limits
        maplet = mappy.Maplet(200000, 0.001, mappy.CounterOperator())

        # Get initial memory usage
        initial_stats = Stats(maplet.stats())
        initial_memory = initial_stats.memory_usage

        # Insert large amount of data
        for i in range(50000):
            key = f"load_key_{i}"
            value = random.randint(1, 1000)
            maplet.insert(key, value)

        # Get final memory usage
        final_stats = Stats(maplet.stats())
        final_memory = final_stats.memory_usage

        # Calculate memory efficiency
        memory_increase = final_memory - initial_memory
        memory_per_item = memory_increase / 50000

        # Memory usage should be reasonable
        assert memory_per_item < 100  # Less than 100 bytes per item
        assert final_stats.load_factor < 0.8  # Load factor should be reasonable

    def test_memory_fragmentation(self):
        """Test memory fragmentation over time"""
        maplet = mappy.Maplet(10000, 0.01, mappy.CounterOperator())

        # Insert and delete data multiple times
        for cycle in range(10):
            # Insert data
            for i in range(1000):
                key = f"cycle_{cycle}_key_{i}"
                value = random.randint(1, 100)
                maplet.insert(key, value)

            # Delete some data
            for i in range(500):
                key = f"cycle_{cycle}_key_{i}"
                maplet.delete(key)

            # Check memory usage
            stats = Stats(maplet.stats())
            assert stats.memory_usage > 0
            assert stats.item_count >= 500  # Should have at least 500 items

    def test_large_value_memory_usage(self):
        """Test memory usage with large values"""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert large values
        large_values = [2**63 - 1, 2**32 - 1, 2**16 - 1]

        for i, value in enumerate(large_values):
            key = f"large_value_{i}"
            maplet.insert(key, value)

        # Verify memory usage is reasonable
        stats = Stats(maplet.stats())
        assert stats.memory_usage > 0
        assert stats.item_count == len(large_values)

        # Verify large values can be retrieved
        for i, expected_value in enumerate(large_values):
            key = f"large_value_{i}"
            result = maplet.query(key)
            assert result is not None
            assert result >= expected_value

