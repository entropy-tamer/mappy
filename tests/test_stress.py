"""
Stress tests: high load, concurrent operations, long-running stability.

Tests mappy-python under extreme conditions and stress scenarios.
"""

import pytest
import numpy as np
import mappy_python
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from .test_utils import measure_operation


class TestHighLoad:
    """Test under high load conditions"""
    
    @pytest.mark.stress
    def test_high_load_inserts(self):
        """Test high load insert operations"""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def insert_operation():
            for i in range(50000):
                maplet.insert(f"key_{i}", i)
        
        result = measure_operation(insert_operation, iterations=1)
        assert result.success_rate > 0.0
    
    @pytest.mark.stress
    def test_high_load_queries(self):
        """Test high load query operations"""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        # Pre-populate
        for i in range(50000):
            maplet.insert(f"key_{i}", i)
        
        def query_operation():
            for i in range(50000):
                maplet.query(f"key_{i}")
        
        result = measure_operation(query_operation, iterations=1)
        assert result.success_rate > 0.0
    
    @pytest.mark.stress
    def test_high_load_mixed(self):
        """Test high load mixed operations"""
        maplet = mappy_python.Maplet(
            capacity=200000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def mixed_operation():
            for i in range(10000):
                maplet.insert(f"key_{i}", i)
            for i in range(10000):
                maplet.query(f"key_{i}")
            for i in range(10000, 20000):
                maplet.insert(f"key_{i}", i)
        
        result = measure_operation(mixed_operation, iterations=1)
        assert result.success_rate > 0.0


class TestConcurrentOperations:
    """Test concurrent operations"""
    
    @pytest.mark.stress
    def test_concurrent_inserts(self):
        """Test concurrent insert operations"""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def insert_worker(worker_id, num_keys):
            for i in range(num_keys):
                key = f"worker_{worker_id}_key_{i}"
                maplet.insert(key, i)
        
        num_workers = 10
        keys_per_worker = 1000
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [
                executor.submit(insert_worker, i, keys_per_worker)
                for i in range(num_workers)
            ]
            
            for future in as_completed(futures):
                future.result()  # Wait for completion
        
        # Verify some keys were inserted
        success_count = 0
        for worker_id in range(num_workers):
            for i in range(min(100, keys_per_worker)):
                key = f"worker_{worker_id}_key_{i}"
                result = maplet.query(key)
                if result is not None:
                    success_count += 1
        
        assert success_count > 0
    
    @pytest.mark.stress
    def test_concurrent_queries(self):
        """Test concurrent query operations"""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        # Pre-populate
        for i in range(10000):
            maplet.insert(f"key_{i}", i)
        
        def query_worker(worker_id, num_keys):
            success_count = 0
            for i in range(num_keys):
                key = f"key_{i}"
                result = maplet.query(key)
                if result is not None:
                    success_count += 1
            return success_count
        
        num_workers = 10
        keys_per_worker = 1000
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [
                executor.submit(query_worker, i, keys_per_worker)
                for i in range(num_workers)
            ]
            
            results = [future.result() for future in as_completed(futures)]
        
        # Should have some successful queries
        total_success = sum(results)
        assert total_success > 0
    
    @pytest.mark.stress
    def test_concurrent_mixed(self):
        """Test concurrent mixed operations"""
        maplet = mappy_python.Maplet(
            capacity=200000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def mixed_worker(worker_id, num_ops):
            for i in range(num_ops):
                key = f"worker_{worker_id}_key_{i}"
                maplet.insert(key, i)
                maplet.query(key)
        
        num_workers = 5
        ops_per_worker = 1000
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [
                executor.submit(mixed_worker, i, ops_per_worker)
                for i in range(num_workers)
            ]
            
            for future in as_completed(futures):
                future.result()
        
        # Verify some operations succeeded
        success_count = 0
        for worker_id in range(num_workers):
            for i in range(min(100, ops_per_worker)):
                key = f"worker_{worker_id}_key_{i}"
                result = maplet.query(key)
                if result is not None:
                    success_count += 1
        
        assert success_count > 0


class TestLongRunning:
    """Test long-running stability"""
    
    @pytest.mark.stress
    @pytest.mark.slow
    def test_long_running_inserts(self):
        """Test long-running insert operations"""
        maplet = mappy_python.Maplet(
            capacity=1000000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        start_time = time.time()
        duration = 10  # Run for 10 seconds
        
        insert_count = 0
        while time.time() - start_time < duration:
            key = f"long_key_{insert_count}"
            maplet.insert(key, insert_count)
            insert_count += 1
        
        # Should have inserted many keys
        assert insert_count > 0
    
    @pytest.mark.stress
    @pytest.mark.slow
    def test_long_running_mixed(self):
        """Test long-running mixed operations"""
        maplet = mappy_python.Maplet(
            capacity=1000000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        start_time = time.time()
        duration = 10  # Run for 10 seconds
        
        operation_count = 0
        while time.time() - start_time < duration:
            key = f"long_mixed_{operation_count}"
            maplet.insert(key, operation_count)
            maplet.query(key)
            operation_count += 1
        
        # Should have performed many operations
        assert operation_count > 0


class TestVectorStress:
    """Stress tests for vector operations"""
    
    @pytest.mark.stress
    def test_large_vector_stress(self):
        """Test stress with large vectors"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Large vectors (10K elements)
        large_vectors = [np.random.rand(10000) for _ in range(100)]
        
        for i, vec in enumerate(large_vectors):
            maplet.insert(f"large_vec_{i}", vec)
        
        # Verify retrieval
        success_count = 0
        for i, expected in enumerate(large_vectors):
            result = maplet.query(f"large_vec_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(large_vectors) * 0.8
    
    @pytest.mark.stress
    def test_many_vectors_stress(self):
        """Test stress with many vectors"""
        maplet = mappy_python.Maplet(
            capacity=50000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Many vectors (1K vectors of 100 dims)
        vectors = [np.random.rand(100) for _ in range(1000)]
        
        for i, vec in enumerate(vectors):
            maplet.insert(f"vec_{i}", vec)
        
        # Verify retrieval
        success_count = 0
        for i, expected in enumerate(vectors):
            result = maplet.query(f"vec_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(vectors) * 0.8
    
    @pytest.mark.stress
    def test_concurrent_vector_operations(self):
        """Test concurrent vector operations"""
        maplet = mappy_python.Maplet(
            capacity=100000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        def vector_worker(worker_id, num_vectors):
            vectors = [np.random.rand(100) for _ in range(num_vectors)]
            for i, vec in enumerate(vectors):
                key = f"worker_{worker_id}_vec_{i}"
                maplet.insert(key, vec)
        
        num_workers = 5
        vectors_per_worker = 100
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [
                executor.submit(vector_worker, i, vectors_per_worker)
                for i in range(num_workers)
            ]
            
            for future in as_completed(futures):
                future.result()
        
        # Verify some vectors were inserted
        success_count = 0
        for worker_id in range(num_workers):
            for i in range(min(50, vectors_per_worker)):
                key = f"worker_{worker_id}_vec_{i}"
                result = maplet.query(key)
                if result is not None:
                    success_count += 1
        
        assert success_count > 0


class TestCapacityStress:
    """Stress tests near capacity limits"""
    
    @pytest.mark.stress
    def test_near_capacity_inserts(self):
        """Test inserts near capacity limit"""
        capacity = 10000
        maplet = mappy_python.Maplet(
            capacity=capacity,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        # Insert up to 90% of capacity
        num_inserts = int(capacity * 0.9)
        success_count = 0
        
        for i in range(num_inserts):
            try:
                maplet.insert(f"capacity_key_{i}", i)
                success_count += 1
            except Exception:
                pass  # May hit capacity
        
        # Should have inserted most keys
        assert success_count >= num_inserts * 0.8
    
    @pytest.mark.stress
    def test_capacity_vector_operations(self):
        """Test vector operations near capacity"""
        capacity = 5000
        maplet = mappy_python.Maplet(
            capacity=capacity,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Insert vectors up to 80% of capacity
        num_vectors = int(capacity * 0.8)
        vectors = [np.random.rand(50) for _ in range(num_vectors)]
        
        success_count = 0
        for i, vec in enumerate(vectors):
            try:
                maplet.insert(f"capacity_vec_{i}", vec)
                success_count += 1
            except Exception:
                pass  # May hit capacity
        
        # Should have inserted most vectors
        assert success_count >= num_vectors * 0.7






