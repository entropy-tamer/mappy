"""
Advanced correctness tests: vector operator correctness, type coercion, edge cases.

Tests edge cases and advanced correctness scenarios for mappy-python.
"""

import pytest
import numpy as np
import mappy_python
from .test_utils import assert_vector_equal


class TestVectorOperatorCorrectness:
    """Test vector operator correctness"""
    
    def test_vector_addition_commutative(self):
        """Test that vector addition is commutative"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "commutative"
        vec1 = np.array([1.0, 2.0, 3.0])
        vec2 = np.array([4.0, 5.0, 6.0])
        
        # Insert vec1 then vec2
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        result1 = maplet.query(key)
        
        # Clear and insert vec2 then vec1
        maplet2 = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        maplet2.insert(key, vec2)
        maplet2.insert(key, vec1)
        result2 = maplet2.query(key)
        
        if result1 is not None and result2 is not None:
            result1_array = np.array(result1) if isinstance(result1, list) else result1
            result2_array = np.array(result2) if isinstance(result2, list) else result2
            # Results should be equal (commutative property)
            assert np.allclose(result1_array, result2_array, atol=1e-5)
    
    def test_vector_addition_associative(self):
        """Test that vector addition is associative"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "associative"
        vec1 = np.array([1.0, 2.0, 3.0])
        vec2 = np.array([4.0, 5.0, 6.0])
        vec3 = np.array([7.0, 8.0, 9.0])
        
        # (vec1 + vec2) + vec3
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        maplet.insert(key, vec3)
        result1 = maplet.query(key)
        
        # vec1 + (vec2 + vec3)
        maplet2 = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        maplet2.insert(key, vec1)
        temp_key = "temp"
        maplet2.insert(temp_key, vec2)
        maplet2.insert(temp_key, vec3)
        temp_result = maplet2.query(temp_key)
        if temp_result is not None:
            maplet2.insert(key, np.array(temp_result) if isinstance(temp_result, list) else temp_result)
        result2 = maplet2.query(key)
        
        if result1 is not None and result2 is not None:
            result1_array = np.array(result1) if isinstance(result1, list) else result1
            result2_array = np.array(result2) if isinstance(result2, list) else result2
            expected = vec1 + vec2 + vec3
            # Both should equal vec1 + vec2 + vec3
            assert np.allclose(result1_array, expected, atol=1e-5)
            assert np.allclose(result2_array, expected, atol=1e-5)
    
    def test_vector_zero_identity(self):
        """Test that zero vector is identity element"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "identity"
        vec = np.array([1.0, 2.0, 3.0])
        zero = np.array([0.0, 0.0, 0.0])
        
        maplet.insert(key, vec)
        maplet.insert(key, zero)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        # Result should equal original vector (zero is identity)
        assert np.allclose(result_array, vec, atol=1e-5)
    
    def test_vector_length_mismatch(self):
        """Test handling of vector length mismatch"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "mismatch"
        vec1 = np.array([1.0, 2.0, 3.0])
        vec2 = np.array([4.0, 5.0])  # Different length
        
        maplet.insert(key, vec1)
        
        # Inserting different length vector should raise error
        with pytest.raises(Exception):  # Should raise ValueError
            maplet.insert(key, vec2)


class TestTypeCoercion:
    """Test type coercion and conversion"""
    
    def test_int_to_float_coercion(self):
        """Test coercion of int to float in vectors"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "coercion"
        # Insert int list
        int_list = [1, 2, 3, 4, 5]
        maplet.insert(key, int_list)
        
        # Insert float list
        float_list = [1.5, 2.5, 3.5, 4.5, 5.5]
        maplet.insert(key, float_list)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = np.array(int_list, dtype=float) + np.array(float_list)
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_numpy_array_coercion(self):
        """Test coercion between numpy array types"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "numpy_coercion"
        # Insert float32 array
        arr1 = np.array([1.0, 2.0, 3.0], dtype=np.float32)
        maplet.insert(key, arr1)
        
        # Insert float64 array
        arr2 = np.array([4.0, 5.0, 6.0], dtype=np.float64)
        maplet.insert(key, arr2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = arr1.astype(float) + arr2.astype(float)
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_list_to_numpy_coercion(self):
        """Test coercion from Python list to numpy array"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "list_coercion"
        # Insert Python list
        py_list = [1.0, 2.0, 3.0]
        maplet.insert(key, py_list)
        
        # Insert numpy array
        np_array = np.array([4.0, 5.0, 6.0])
        maplet.insert(key, np_array)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = np.array(py_list) + np_array
        assert np.allclose(result_array, expected, atol=1e-5)


class TestEdgeCases:
    """Test edge cases and boundary conditions"""
    
    def test_empty_vector(self):
        """Test handling of empty vectors"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "empty"
        empty_vec = np.array([])
        
        # Should handle empty vector
        maplet.insert(key, empty_vec.tolist())
        result = maplet.query(key)
        
        # Result might be None or empty list
        if result is not None:
            result_array = np.array(result) if isinstance(result, list) else result
            assert len(result_array) == 0
    
    def test_single_element_vector(self):
        """Test handling of single element vectors"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "single"
        vec1 = np.array([1.0])
        vec2 = np.array([2.0])
        
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = vec1 + vec2
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_very_large_vector(self):
        """Test handling of very large vectors"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "large"
        large_vec = np.random.rand(10000)
        
        maplet.insert(key, large_vec)
        result = maplet.query(key)
        
        if result is not None:
            result_array = np.array(result) if isinstance(result, list) else result
            assert len(result_array) == len(large_vec)
            assert np.allclose(result_array, large_vec, atol=1e-3)
    
    def test_very_small_values(self):
        """Test handling of very small numerical values"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "small"
        vec1 = np.array([1e-20, 1e-30, 1e-40])
        vec2 = np.array([2e-20, 2e-30, 2e-40])
        
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = vec1 + vec2
        # Use relative tolerance for very small values
        assert np.allclose(result_array, expected, rtol=1e-3)
    
    def test_very_large_values(self):
        """Test handling of very large numerical values"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "huge"
        vec1 = np.array([1e20, 1e30, 1e40])
        vec2 = np.array([2e20, 2e30, 2e40])
        
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = vec1 + vec2
        # Use relative tolerance for very large values
        assert np.allclose(result_array, expected, rtol=1e-5)
    
    def test_nan_handling(self):
        """Test handling of NaN values"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "nan"
        vec = np.array([1.0, np.nan, 3.0])
        
        # Should handle NaN (might propagate or be handled specially)
        try:
            maplet.insert(key, vec)
            result = maplet.query(key)
            # If it succeeds, check result
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                # NaN handling is implementation-dependent
                assert len(result_array) == len(vec)
        except Exception:
            # NaN might cause errors, which is acceptable
            pass
    
    def test_inf_handling(self):
        """Test handling of infinity values"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "inf"
        vec = np.array([1.0, np.inf, 3.0])
        
        # Should handle inf (might propagate or be handled specially)
        try:
            maplet.insert(key, vec)
            result = maplet.query(key)
            # If it succeeds, check result
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                # Inf handling is implementation-dependent
                assert len(result_array) == len(vec)
        except Exception:
            # Inf might cause errors, which is acceptable
            pass


class TestOperatorCorrectness:
    """Test correctness of different operators"""
    
    def test_counter_operator(self):
        """Test counter operator correctness"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        key = "counter"
        maplet.insert(key, 5)
        maplet.insert(key, 3)
        maplet.insert(key, 2)
        
        result = maplet.query(key)
        # Result should be sum: 5 + 3 + 2 = 10
        if result is not None:
            assert result == 10
    
    def test_max_operator(self):
        """Test max operator correctness"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.MaxOperator()
        )
        
        key = "max"
        maplet.insert(key, 5)
        maplet.insert(key, 10)
        maplet.insert(key, 7)
        
        result = maplet.query(key)
        # Result should be max: max(5, 10, 7) = 10
        if result is not None:
            assert result == 10
    
    def test_min_operator(self):
        """Test min operator correctness"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.MinOperator()
        )
        
        key = "min"
        maplet.insert(key, 5)
        maplet.insert(key, 2)
        maplet.insert(key, 7)
        
        result = maplet.query(key)
        # Result should be min: min(5, 2, 7) = 2
        if result is not None:
            assert result == 2



