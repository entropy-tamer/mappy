"""
Test mappy-python with real-world datasets: CIFAR-10, MNIST, word embeddings, FAISS vectors.

Tests vector operator with realistic ML dataset scenarios.
"""

import pytest
import numpy as np
import mappy_python
from .test_utils import (
    load_mnist_vectors,
    load_cifar10_vectors,
    load_word_embeddings,
    load_faiss_vectors,
    assert_vector_equal
)


class TestMNIST:
    """Test with MNIST-like vectors (784-dim flattened images)"""
    
    def test_mnist_insertion(self):
        """Test inserting MNIST vectors"""
        vectors = load_mnist_vectors(num_samples=100, dim=784)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, vector in enumerate(vectors):
            key = f"mnist_{i}"
            maplet.insert(key, vector)
        
        # Verify queries
        success_count = 0
        for i in range(len(vectors)):
            key = f"mnist_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                expected = vectors[i]
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(vectors) * 0.9
    
    def test_mnist_batch_operations(self):
        """Test batch operations with MNIST vectors"""
        vectors = load_mnist_vectors(num_samples=500, dim=784)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Batch insert
        for i, vector in enumerate(vectors):
            key = f"batch_mnist_{i}"
            maplet.insert(key, vector.tolist())
        
        # Batch query
        results = []
        for i in range(len(vectors)):
            key = f"batch_mnist_{i}"
            result = maplet.query(key)
            if result is not None:
                results.append(np.array(result))
        
        assert len(results) >= len(vectors) * 0.9


class TestCIFAR10:
    """Test with CIFAR-10-like vectors (3072-dim RGB images)"""
    
    def test_cifar10_insertion(self):
        """Test inserting CIFAR-10 vectors"""
        vectors = load_cifar10_vectors(num_samples=100, dim=3072)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, vector in enumerate(vectors):
            key = f"cifar10_{i}"
            maplet.insert(key, vector)
        
        # Verify queries
        success_count = 0
        for i in range(len(vectors)):
            key = f"cifar10_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                expected = vectors[i]
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(vectors) * 0.9
    
    def test_cifar10_high_dimensionality(self):
        """Test with high-dimensional CIFAR-10 vectors"""
        vectors = load_cifar10_vectors(num_samples=50, dim=3072)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, vector in enumerate(vectors):
            key = f"cifar10_hd_{i}"
            maplet.insert(key, vector)
        
        # Verify we can handle high-dimensional vectors
        success_count = 0
        for i in range(len(vectors)):
            key = f"cifar10_hd_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                assert result_array.shape == vectors[i].shape
                success_count += 1
        
        assert success_count >= len(vectors) * 0.85


class TestWordEmbeddings:
    """Test with word embeddings (Word2Vec/GloVe style)"""
    
    def test_word_embedding_insertion(self):
        """Test inserting word embeddings"""
        vectors = load_word_embeddings(num_samples=100, dim=300)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        words = [f"word_{i}" for i in range(len(vectors))]
        for word, vector in zip(words, vectors):
            maplet.insert(word, vector)
        
        # Verify queries
        success_count = 0
        for word, expected_vector in zip(words, vectors):
            result = maplet.query(word)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                # Word embeddings are normalized, so check cosine similarity
                dot_product = np.dot(result_array, expected_vector)
                norm1 = np.linalg.norm(result_array)
                norm2 = np.linalg.norm(expected_vector)
                if norm1 > 0 and norm2 > 0:
                    similarity = dot_product / (norm1 * norm2)
                    if similarity > 0.9:  # High similarity for normalized vectors
                        success_count += 1
        
        assert success_count >= len(vectors) * 0.9
    
    def test_word_embedding_similarity_search(self):
        """Test similarity search with word embeddings"""
        vectors = load_word_embeddings(num_samples=200, dim=300)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        words = [f"word_{i}" for i in range(len(vectors))]
        for word, vector in zip(words, vectors):
            maplet.insert(word, vector)
        
        # Find similar words using cosine similarity
        query_vector = vectors[0]
        similarities = []
        
        for word in words[:50]:  # Check first 50
            result = maplet.query(word)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                dot_product = np.dot(result_array, query_vector)
                norm1 = np.linalg.norm(result_array)
                norm2 = np.linalg.norm(query_vector)
                if norm1 > 0 and norm2 > 0:
                    similarity = dot_product / (norm1 * norm2)
                    similarities.append(similarity)
        
        # Should find some similar vectors
        assert len(similarities) > 0


class TestFAISSVectors:
    """Test with FAISS-compatible vectors for similarity search"""
    
    def test_faiss_vector_insertion(self):
        """Test inserting FAISS-compatible vectors"""
        vectors = load_faiss_vectors(num_samples=100, dim=128)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, vector in enumerate(vectors):
            key = f"faiss_{i}"
            maplet.insert(key, vector)
        
        # Verify queries
        success_count = 0
        for i in range(len(vectors)):
            key = f"faiss_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                expected = vectors[i]
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(vectors) * 0.9
    
    def test_faiss_vector_similarity(self):
        """Test similarity operations with FAISS vectors"""
        vectors = load_faiss_vectors(num_samples=100, dim=128)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, vector in enumerate(vectors):
            key = f"faiss_sim_{i}"
            maplet.insert(key, vector)
        
        # Compute L2 distance (FAISS-style)
        query_vector = vectors[0]
        distances = []
        
        for i in range(min(50, len(vectors))):
            key = f"faiss_sim_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                # L2 distance
                distance = np.linalg.norm(result_array - query_vector)
                distances.append(distance)
        
        # Should find some vectors
        assert len(distances) > 0
    
    def test_faiss_vector_aggregation(self):
        """Test aggregating FAISS vectors"""
        vectors = load_faiss_vectors(num_samples=50, dim=128)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "aggregated_faiss"
        # Insert multiple vectors (should be added)
        for vector in vectors[:10]:
            maplet.insert(key, vector)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected_sum = sum(vectors[:10])
        
        # Result should be sum of all vectors
        assert np.allclose(result_array, expected_sum, atol=1e-2)


class TestDatasetPerformance:
    """Performance tests with real datasets"""
    
    @pytest.mark.benchmark
    def test_mnist_performance(self):
        """Benchmark MNIST vector operations"""
        vectors = load_mnist_vectors(num_samples=1000, dim=784)
        maplet = mappy_python.Maplet(
            capacity=2000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Insert all vectors
        for i, vector in enumerate(vectors):
            key = f"perf_mnist_{i}"
            maplet.insert(key, vector)
        
        # Query all vectors
        success_count = 0
        for i in range(len(vectors)):
            key = f"perf_mnist_{i}"
            result = maplet.query(key)
            if result is not None:
                success_count += 1
        
        assert success_count >= len(vectors) * 0.9
    
    @pytest.mark.benchmark
    def test_cifar10_performance(self):
        """Benchmark CIFAR-10 vector operations"""
        vectors = load_cifar10_vectors(num_samples=500, dim=3072)
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Insert all vectors
        for i, vector in enumerate(vectors):
            key = f"perf_cifar10_{i}"
            maplet.insert(key, vector)
        
        # Query all vectors
        success_count = 0
        for i in range(len(vectors)):
            key = f"perf_cifar10_{i}"
            result = maplet.query(key)
            if result is not None:
                success_count += 1
        
        assert success_count >= len(vectors) * 0.85  # Lower threshold for high-dim






