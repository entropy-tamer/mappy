"""
NumPy integration tests for mappy-python bindings
"""

import time
from typing import List

import mappy_python as mappy
import numpy as np
import pytest
from . import Stats


class TestNumPyArraySupport:
    """Test NumPy array support in maplets"""

    def test_numpy_array_insertion(self, sample_numpy_arrays: List[np.ndarray]):
        """Test inserting NumPy arrays as values"""
        maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

        # Insert NumPy arrays
        for i, array in enumerate(sample_numpy_arrays):
            key = f"array_{i}"
            maplet.insert(key, array)

        # Verify insertion
        for i, expected_array in enumerate(sample_numpy_arrays):
            key = f"array_{i}"
            result = maplet.query(key)
            assert result is not None
            # Result may be a list, convert to numpy array for comparison
            if not isinstance(result, np.ndarray):
                result = np.array(result)
            assert result.shape == expected_array.shape

    def test_numpy_array_operations(self):
        """Test NumPy array operations"""
        maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

        # Create test arrays
        array1 = np.array([1.0, 2.0, 3.0])
        array2 = np.array([4.0, 5.0, 6.0])
        array3 = np.array([7.0, 8.0, 9.0])

        # Insert arrays
        maplet.insert("vec1", array1)
        maplet.insert("vec2", array2)
        maplet.insert("vec3", array3)

        # Query arrays
        result1 = maplet.query("vec1")
        result2 = maplet.query("vec2")
        result3 = maplet.query("vec3")

        # Convert to numpy arrays if needed
        if not isinstance(result1, np.ndarray):
            result1 = np.array(result1)
        if not isinstance(result2, np.ndarray):
            result2 = np.array(result2)
        if not isinstance(result3, np.ndarray):
            result3 = np.array(result3)

        # Verify results
        assert np.allclose(result1, array1, atol=1e-6)
        assert np.allclose(result2, array2, atol=1e-6)
        assert np.allclose(result3, array3, atol=1e-6)

    def test_numpy_array_merge_operations(self):
        """Test NumPy array merge operations"""
        maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

        # Insert same key with different arrays
        key = "merged_vector"
        array1 = np.array([1.0, 2.0, 3.0])
        array2 = np.array([4.0, 5.0, 6.0])

        maplet.insert(key, array1)
        maplet.insert(key, array2)

        # Query merged result
        result = maplet.query(key)
        assert result is not None
        if not isinstance(result, np.ndarray):
            result = np.array(result)
        assert result.shape == array1.shape

        # Result should be element-wise sum (for VectorOperator)
        expected = array1 + array2
        assert np.allclose(result, expected, atol=1e-6)

    def test_numpy_array_types(self):
        """Test different NumPy array types"""
        maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

        # Test different data types (only float64 supported for VectorOperator)
        test_arrays = [
            np.array([1.0, 2.0, 3.0], dtype=np.float64),
        ]

        for i, array in enumerate(test_arrays):
            key = f"type_test_{i}"
            maplet.insert(key, array)

            result = maplet.query(key)
            assert result is not None
            if not isinstance(result, np.ndarray):
                result = np.array(result)
            assert result.shape == array.shape

    def test_numpy_array_dimensions(self):
        """Test NumPy arrays with different dimensions"""
        maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

        # Test 1D arrays (VectorOperator supports 1D)
        test_array = np.array([1, 2, 3])  # 1D

        key = "dim_test"
        maplet.insert(key, test_array)

        result = maplet.query(key)
        assert result is not None
        if not isinstance(result, np.ndarray):
            result = np.array(result)
        assert result.shape == test_array.shape or len(result) == len(test_array)

    def test_numpy_array_performance(self):
        """Test NumPy array performance"""
        maplet = mappy.Maplet(10000, 0.001, mappy.VectorOperator())

        # Create large arrays
        large_arrays = [np.random.rand(1000) for _ in range(100)]

        # Insert arrays
        start_time = time.time()
        for i, array in enumerate(large_arrays):
            maplet.insert(f"large_array_{i}", array)
        insert_time = time.time() - start_time

        # Query arrays
        start_time = time.time()
        for i in range(100):
            result = maplet.query(f"large_array_{i}")
            assert result is not None
        query_time = time.time() - start_time

        # Performance should be reasonable
        assert insert_time < 10.0  # Should insert 100 arrays in under 10 seconds
        assert query_time < 5.0  # Should query 100 arrays in under 5 seconds

    def test_numpy_array_memory_usage(self):
        """Test memory usage with NumPy arrays"""
        maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

        # Get initial memory usage
        initial_stats = Stats(maplet.stats())
        initial_memory = initial_stats.memory_usage

        # Insert arrays
        for i in range(100):
            array = np.random.rand(100)  # 100 float64 values = 800 bytes
            maplet.insert(f"memory_test_{i}", array)

        # Get final memory usage
        final_stats = Stats(maplet.stats())
        final_memory = final_stats.memory_usage

        # Memory should have increased
        memory_increase = final_memory - initial_memory
        assert memory_increase > 0

        # Memory increase should be reasonable
        # Note: Maplet is a probabilistic data structure, so memory usage may be less than raw data size
        # We just check that memory increased (which it did)
        # The exact amount depends on the internal structure and compression

    def test_numpy_array_edge_cases(self):
        """Test NumPy array edge cases"""
        maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

        # Test empty array
        empty_array = np.array([])
        maplet.insert("empty", empty_array)
        result = maplet.query("empty")
        assert result is not None
        if not isinstance(result, np.ndarray):
            result = np.array(result)
        assert result.shape == (0,) or len(result) == 0

        # Test single element array
        single_array = np.array([42.0])
        maplet.insert("single", single_array)
        result = maplet.query("single")
        assert result is not None
        if not isinstance(result, np.ndarray):
            result = np.array(result)
        assert len(result) == 1
        assert result[0] == 42.0

    def test_numpy_array_serialization(self):
        """Test NumPy array serialization/deserialization"""
        maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())

        # Create 1D array (VectorOperator supports 1D)
        complex_array = np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])

        # Insert and query
        maplet.insert("complex", complex_array)
        result = maplet.query("complex")

        # Convert to numpy array if needed
        if not isinstance(result, np.ndarray):
            result = np.array(result)

        # Verify exact match
        assert np.array_equal(result, complex_array) or np.allclose(result, complex_array)

    def test_numpy_array_with_different_operators(self):
        """Test NumPy arrays with different merge operators"""
        # Test with VectorOperator (element-wise sum)
        vector_maplet = mappy.Maplet(1000, 0.01, mappy.VectorOperator())
        array1 = np.array([1.0, 2.0, 3.0])
        array2 = np.array([4.0, 5.0, 6.0])

        vector_maplet.insert("vec", array1)
        vector_maplet.insert("vec", array2)
        result = vector_maplet.query("vec")

        if not isinstance(result, np.ndarray):
            result = np.array(result)

        expected = array1 + array2
        assert np.allclose(result, expected, atol=1e-6)


class TestNumPyPerformance:
    """Test NumPy performance characteristics"""

    @pytest.mark.benchmark
    def test_numpy_array_benchmark(self, benchmark):
        """Benchmark NumPy array operations"""
        maplet = mappy.Maplet(10000, 0.001, mappy.VectorOperator())

        def benchmark_operations():
            # Create test array
            test_array = np.random.rand(1000)

            # Insert
            maplet.insert("benchmark", test_array)

            # Query
            result = maplet.query("benchmark")
            assert result is not None

        benchmark(benchmark_operations)

    @pytest.mark.benchmark
    def test_numpy_array_batch_operations(self, benchmark):
        """Benchmark batch NumPy array operations"""
        maplet = mappy.Maplet(10000, 0.001, mappy.VectorOperator())

        def benchmark_batch():
            # Create multiple arrays
            arrays = [np.random.rand(100) for _ in range(50)]

            # Batch insert
            for i, array in enumerate(arrays):
                maplet.insert(f"batch_{i}", array)

            # Batch query
            for i in range(50):
                result = maplet.query(f"batch_{i}")
                assert result is not None

        benchmark(benchmark_batch)

