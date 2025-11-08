"""
Test mappy-python for data analysis: time series, feature engineering, dimensionality reduction.

Tests vector operator with data analysis scenarios.
"""

import pytest
import numpy as np
import mappy_python
from .test_utils import assert_vector_equal, measure_operation


class TestTimeSeries:
    """Test time series data operations"""
    
    def test_time_series_accumulation(self):
        """Test accumulating time series data"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "timeseries"
        # Generate time series
        t = np.linspace(0, 10, 1000)
        series1 = np.sin(2 * np.pi * 0.1 * t) + np.random.normal(0, 0.1, len(t))
        series2 = np.cos(2 * np.pi * 0.1 * t) + np.random.normal(0, 0.1, len(t))
        
        maplet.insert(key, series1)
        maplet.insert(key, series2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = series1 + series2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_multiple_time_series(self):
        """Test storing multiple time series"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Generate multiple time series
        t = np.linspace(0, 10, 100)
        series_list = []
        for i in range(10):
            freq = 0.1 + i * 0.05
            series = np.sin(2 * np.pi * freq * t)
            series_list.append(series)
            maplet.insert(f"series_{i}", series)
        
        # Verify retrieval
        success_count = 0
        for i, expected in enumerate(series_list):
            result = maplet.query(f"series_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                if np.allclose(result_array, expected, atol=1e-5):
                    success_count += 1
        
        assert success_count >= len(series_list) * 0.9
    
    def test_time_series_features(self):
        """Test extracting and storing time series features"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "features"
        # Generate time series
        t = np.linspace(0, 10, 1000)
        series = np.sin(2 * np.pi * 0.1 * t) + np.random.normal(0, 0.1, len(t))
        
        # Extract features (mean, std, min, max, etc.)
        features1 = np.array([
            np.mean(series),
            np.std(series),
            np.min(series),
            np.max(series),
            np.median(series)
        ])
        
        # Another set of features
        series2 = np.cos(2 * np.pi * 0.1 * t) + np.random.normal(0, 0.1, len(t))
        features2 = np.array([
            np.mean(series2),
            np.std(series2),
            np.min(series2),
            np.max(series2),
            np.median(series2)
        ])
        
        maplet.insert(key, features1)
        maplet.insert(key, features2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = features1 + features2
        
        assert np.allclose(result_array, expected, atol=1e-5)


class TestFeatureEngineering:
    """Test feature engineering operations"""
    
    def test_feature_aggregation(self):
        """Test aggregating engineered features"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "aggregated_features"
        # Generate feature vectors
        features1 = np.random.rand(50)
        features2 = np.random.rand(50)
        
        maplet.insert(key, features1)
        maplet.insert(key, features2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = features1 + features2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_one_hot_encoding_accumulation(self):
        """Test accumulating one-hot encoded features"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "onehot"
        # Generate one-hot encodings
        onehot1 = np.array([1, 0, 0, 1, 0, 1, 0, 0])
        onehot2 = np.array([0, 1, 0, 0, 1, 0, 1, 0])
        
        maplet.insert(key, onehot1)
        maplet.insert(key, onehot2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = onehot1 + onehot2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_polynomial_features(self):
        """Test storing polynomial features"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "poly"
        # Generate polynomial features
        x = np.array([1.0, 2.0, 3.0])
        poly1 = np.array([x[0], x[0]**2, x[0]**3, x[1], x[1]**2, x[1]**3])
        poly2 = np.array([x[2], x[2]**2, x[2]**3, x[0], x[0]**2, x[0]**3])
        
        maplet.insert(key, poly1)
        maplet.insert(key, poly2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = poly1 + poly2
        
        assert np.allclose(result_array, expected, atol=1e-5)


class TestDimensionalityReduction:
    """Test dimensionality reduction operations"""
    
    def test_pca_components(self):
        """Test storing PCA components"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "pca"
        # Generate PCA-like components
        components1 = np.random.rand(10)
        components2 = np.random.rand(10)
        
        maplet.insert(key, components1)
        maplet.insert(key, components2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = components1 + components2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_tsne_embeddings(self):
        """Test storing t-SNE embeddings"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "tsne"
        # Generate t-SNE-like embeddings (2D)
        embedding1 = np.random.rand(2)
        embedding2 = np.random.rand(2)
        
        maplet.insert(key, embedding1)
        maplet.insert(key, embedding2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = embedding1 + embedding2
        
        assert np.allclose(result_array, expected, atol=1e-5)
    
    def test_autoencoder_latent_space(self):
        """Test storing autoencoder latent representations"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "latent"
        # Generate latent space vectors
        latent1 = np.random.rand(32)  # 32-dim latent space
        latent2 = np.random.rand(32)
        
        maplet.insert(key, latent1)
        maplet.insert(key, latent2)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = latent1 + latent2
        
        assert np.allclose(result_array, expected, atol=1e-5)


class TestDataAnalysisPerformance:
    """Performance tests for data analysis"""
    
    @pytest.mark.benchmark
    def test_large_time_series_performance(self):
        """Benchmark with large time series"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Large time series (100K points)
        t = np.linspace(0, 100, 100000)
        series = np.sin(2 * np.pi * 0.1 * t)
        
        def insert_operation():
            maplet.insert("large_series", series)
        
        result = measure_operation(insert_operation, iterations=10)
        assert result.success_rate > 0.0
    
    @pytest.mark.benchmark
    def test_feature_engineering_performance(self):
        """Benchmark feature engineering operations"""
        maplet = mappy_python.Maplet(
            capacity=2000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Generate many feature vectors
        features = [np.random.rand(100) for _ in range(1000)]
        
        def insert_operation():
            for i, feat in enumerate(features):
                maplet.insert(f"feat_{i}", feat)
        
        result = measure_operation(insert_operation, iterations=1)
        assert result.success_rate > 0.0



