"""
Test mappy-python for numerical computing: linear algebra, signal processing, statistics.

Tests vector operator with scientific computing scenarios.
"""

import pytest
import numpy as np
import mappy_python
from .test_utils import assert_vector_equal, measure_operation


class TestLinearAlgebra:
    """Test linear algebra operations"""
    
    def test_vector_addition(self):
        """Test vector addition operations"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "vector_sum"
        vec1 = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
        vec2 = np.array([5.0, 4.0, 3.0, 2.0, 1.0])
        
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = vec1 + vec2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_matrix_vector_operations(self):
        """Test operations with matrix-like vectors"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Flattened matrix operations
        matrix1 = np.random.rand(10, 10).flatten()
        matrix2 = np.random.rand(10, 10).flatten()
        
        key = "matrix_sum"
        maplet.insert(key, matrix1)
        maplet.insert(key, matrix2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = matrix1 + matrix2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_dot_product_accumulation(self):
        """Test accumulating dot product results"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "dot_accum"
        vectors = [np.random.rand(100) for _ in range(10)]
        
        for vec in vectors:
            maplet.insert(key, vec)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = sum(vectors)
        
        assert np.allclose(result_array, expected, atol=1e-5)


class TestSignalProcessing:
    """Test signal processing operations"""
    
    def test_signal_accumulation(self):
        """Test accumulating signal samples"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "signal"
        # Generate signal samples
        t = np.linspace(0, 1, 1000)
        signal1 = np.sin(2 * np.pi * 10 * t)  # 10 Hz sine wave
        signal2 = np.sin(2 * np.pi * 20 * t)  # 20 Hz sine wave
        
        maplet.insert(key, signal1)
        maplet.insert(key, signal2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = signal1 + signal2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_fft_spectrum_accumulation(self):
        """Test accumulating FFT spectra"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "fft_spectrum"
        # Generate FFT spectra
        signal1 = np.random.rand(512)
        signal2 = np.random.rand(512)
        
        fft1 = np.fft.fft(signal1).real  # Take real part
        fft2 = np.fft.fft(signal2).real
        
        maplet.insert(key, fft1)
        maplet.insert(key, fft2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = fft1 + fft2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_waveform_storage(self):
        """Test storing and retrieving waveforms"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Store multiple waveforms
        waveforms = []
        for i in range(10):
            t = np.linspace(0, 1, 100)
            freq = 10 + i * 5
            waveform = np.sin(2 * np.pi * freq * t)
            waveforms.append(waveform)
            maplet.insert(f"waveform_{i}", waveform)
        
        # Retrieve and verify
        success_count = 0
        for i, expected in enumerate(waveforms):
            result = maplet.query(f"waveform_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                if np.allclose(result_array, expected, atol=1e-5):
                    success_count += 1
        
        assert success_count >= len(waveforms) * 0.9


class TestStatisticalComputing:
    """Test statistical computing operations"""
    
    def test_statistical_aggregation(self):
        """Test aggregating statistical samples"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "stats"
        # Generate statistical samples
        samples = [np.random.normal(0, 1, 1000) for _ in range(10)]
        
        for sample in samples:
            maplet.insert(key, sample)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = sum(samples)
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_histogram_accumulation(self):
        """Test accumulating histogram bins"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "histogram"
        # Generate histograms
        data1 = np.random.normal(0, 1, 1000)
        data2 = np.random.normal(0, 1, 1000)
        
        hist1, _ = np.histogram(data1, bins=50)
        hist2, _ = np.histogram(data2, bins=50)
        
        maplet.insert(key, hist1.astype(float))
        maplet.insert(key, hist2.astype(float))
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = hist1.astype(float) + hist2.astype(float)
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_correlation_accumulation(self):
        """Test accumulating correlation vectors"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "correlation"
        # Generate correlation vectors
        vecs = [np.random.rand(100) for _ in range(5)]
        
        for vec in vecs:
            maplet.insert(key, vec)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = sum(vecs)
        
        assert np.allclose(result_array, expected, atol=1e-5)


class TestNumericalPrecision:
    """Test numerical precision and edge cases"""
    
    def test_small_values(self):
        """Test with very small numerical values"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "small"
        vec1 = np.array([1e-10, 1e-12, 1e-14])
        vec2 = np.array([2e-10, 2e-12, 2e-14])
        
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = vec1 + vec2
        
        # Use relative tolerance for small values
        assert np.allclose(result_array, expected, rtol=1e-3)
    
    def test_large_values(self):
        """Test with very large numerical values"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "large"
        vec1 = np.array([1e10, 1e12, 1e14])
        vec2 = np.array([2e10, 2e12, 2e14])
        
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = vec1 + vec2
        
        assert np.allclose(result_array, expected, rtol=1e-5)
    
    def test_mixed_precision(self):
        """Test with mixed precision values"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "mixed"
        vec1 = np.array([1.0, 1e-5, 1e10, 1e-10])
        vec2 = np.array([2.0, 2e-5, 2e10, 2e-10])
        
        maplet.insert(key, vec1)
        maplet.insert(key, vec2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = vec1 + vec2
        
        assert np.allclose(result_array, expected, rtol=1e-3)


class TestPerformance:
    """Performance tests for numerical computing"""
    
    @pytest.mark.benchmark
    def test_large_vector_performance(self):
        """Benchmark with large vectors"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Large vectors (10K elements)
        large_vec = np.random.rand(10000)
        
        def insert_operation():
            maplet.insert("large_vec", large_vec)
        
        result = measure_operation(insert_operation, iterations=100)
        assert result.success_rate > 0.0
    
    @pytest.mark.benchmark
    def test_many_vectors_performance(self):
        """Benchmark with many vectors"""
        maplet = mappy_python.Maplet(
            capacity=2000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        vectors = [np.random.rand(100) for _ in range(1000)]
        
        def insert_operation():
            for i, vec in enumerate(vectors):
                maplet.insert(f"vec_{i}", vec)
        
        result = measure_operation(insert_operation, iterations=1)
        assert result.success_rate > 0.0



