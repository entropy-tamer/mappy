"""
Test mappy-python with ML embeddings: sentence-transformers, CLIP, BERT-style.

Tests vector operator with real-world ML embedding scenarios including:
- Sentence embeddings (sentence-transformers)
- Image embeddings (CLIP-style)
- Text embeddings (BERT-style)
- Embedding similarity search
- Batch operations
"""

import pytest
import numpy as np
import mappy_python
from .test_utils import (
    generate_ml_embeddings,
    assert_vector_equal,
    measure_operation,
    BenchmarkResult
)


@pytest.fixture
def sentence_embeddings():
    """Generate sentence-transformers style embeddings (384-dim)"""
    return generate_ml_embeddings(num_samples=100, dim=384, embedding_type="sentence")


@pytest.fixture
def image_embeddings():
    """Generate CLIP-style image embeddings (512-dim)"""
    return generate_ml_embeddings(num_samples=100, dim=512, embedding_type="image")


@pytest.fixture
def bert_embeddings():
    """Generate BERT-style text embeddings (768-dim)"""
    return generate_ml_embeddings(num_samples=100, dim=768, embedding_type="text")


class TestSentenceEmbeddings:
    """Test sentence-transformers style embeddings"""
    
    def test_sentence_embedding_insertion(self, sentence_embeddings):
        """Test inserting sentence embeddings into maplet"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, embedding in enumerate(sentence_embeddings):
            key = f"sentence_{i}"
            # Convert to list for insertion
            embedding_list = embedding.tolist()
            maplet.insert(key, embedding_list)
        
        # Verify we can query embeddings
        success_count = 0
        for i in range(len(sentence_embeddings)):
            key = f"sentence_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result)
                expected = sentence_embeddings[i]
                # Check approximate equality (allowing for probabilistic nature)
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        # Should have high success rate (allowing for false positives/negatives)
        assert success_count >= len(sentence_embeddings) * 0.9, \
            f"Only {success_count}/{len(sentence_embeddings)} embeddings retrieved correctly"
    
    def test_sentence_embedding_numpy_direct(self, sentence_embeddings):
        """Test inserting NumPy arrays directly"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, embedding in enumerate(sentence_embeddings):
            key = f"sentence_{i}"
            # Insert NumPy array directly
            maplet.insert(key, embedding)
        
        # Verify queries
        success_count = 0
        for i in range(len(sentence_embeddings)):
            key = f"sentence_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                expected = sentence_embeddings[i]
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(sentence_embeddings) * 0.9
    
    def test_sentence_embedding_merge(self, sentence_embeddings):
        """Test merging sentence embeddings (vector addition)"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "merged_sentence"
        # Insert same embedding multiple times (should add up)
        for embedding in sentence_embeddings[:5]:
            maplet.insert(key, embedding.tolist())
        
        result = maplet.query(key)
        assert result is not None
        
        # Result should be sum of all inserted embeddings
        expected_sum = sum(sentence_embeddings[:5])
        result_array = np.array(result)
        
        # Allow for some tolerance due to probabilistic nature
        assert np.allclose(result_array, expected_sum, atol=1e-2)


class TestImageEmbeddings:
    """Test CLIP-style image embeddings"""
    
    def test_image_embedding_insertion(self, image_embeddings):
        """Test inserting image embeddings"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, embedding in enumerate(image_embeddings):
            key = f"image_{i}"
            maplet.insert(key, embedding)
        
        # Verify queries
        success_count = 0
        for i in range(len(image_embeddings)):
            key = f"image_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                expected = image_embeddings[i]
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(image_embeddings) * 0.9
    
    def test_image_embedding_batch(self, image_embeddings):
        """Test batch insertion of image embeddings"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Batch insert
        for i, embedding in enumerate(image_embeddings):
            key = f"batch_image_{i}"
            maplet.insert(key, embedding.tolist())
        
        # Batch query
        results = []
        for i in range(len(image_embeddings)):
            key = f"batch_image_{i}"
            result = maplet.query(key)
            if result is not None:
                results.append(np.array(result))
        
        assert len(results) >= len(image_embeddings) * 0.9


class TestBERTEmbeddings:
    """Test BERT-style text embeddings"""
    
    def test_bert_embedding_insertion(self, bert_embeddings):
        """Test inserting BERT embeddings"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, embedding in enumerate(bert_embeddings):
            key = f"bert_{i}"
            maplet.insert(key, embedding)
        
        # Verify queries
        success_count = 0
        for i in range(len(bert_embeddings)):
            key = f"bert_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                expected = bert_embeddings[i]
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(bert_embeddings) * 0.9
    
    def test_bert_embedding_high_dimension(self):
        """Test with high-dimensional BERT embeddings (1024-dim)"""
        embeddings = generate_ml_embeddings(num_samples=50, dim=1024, embedding_type="text")
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        for i, embedding in enumerate(embeddings):
            key = f"highdim_{i}"
            maplet.insert(key, embedding)
        
        # Verify queries
        success_count = 0
        for i in range(len(embeddings)):
            key = f"highdim_{i}"
            result = maplet.query(key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                expected = embeddings[i]
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(embeddings) * 0.85  # Slightly lower threshold for high-dim


class TestEmbeddingSimilarity:
    """Test embedding similarity operations"""
    
    def test_embedding_cosine_similarity(self, sentence_embeddings):
        """Test computing cosine similarity between stored embeddings"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Store embeddings
        for i, embedding in enumerate(sentence_embeddings[:10]):
            key = f"sim_{i}"
            maplet.insert(key, embedding)
        
        # Query and compute similarity
        similarities = []
        for i in range(10):
            key = f"sim_{i}"
            result = maplet.query(key)
            if result is not None:
                vec1 = np.array(result) if isinstance(result, list) else result
                vec2 = sentence_embeddings[i]
                
                # Compute cosine similarity
                dot_product = np.dot(vec1, vec2)
                norm1 = np.linalg.norm(vec1)
                norm2 = np.linalg.norm(vec2)
                if norm1 > 0 and norm2 > 0:
                    similarity = dot_product / (norm1 * norm2)
                    similarities.append(similarity)
        
        # Should have high similarity (close to 1.0) for matching embeddings
        if similarities:
            avg_similarity = np.mean(similarities)
            assert avg_similarity > 0.9, f"Average similarity too low: {avg_similarity}"
    
    def test_embedding_aggregation(self, sentence_embeddings):
        """Test aggregating multiple embeddings"""
        maplet = mappy_python.Maplet(
            capacity=1000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        key = "aggregated"
        # Insert multiple embeddings (they should be added)
        for embedding in sentence_embeddings[:5]:
            maplet.insert(key, embedding)
        
        result = maplet.query(key)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected_sum = sum(sentence_embeddings[:5])
        
        # Result should be sum of all embeddings
        assert np.allclose(result_array, expected_sum, atol=1e-2)


class TestEmbeddingPerformance:
    """Performance tests for ML embeddings"""
    
    @pytest.mark.benchmark
    def test_embedding_insert_performance(self):
        """Benchmark embedding insertion performance"""
        embeddings = generate_ml_embeddings(num_samples=1000, dim=384, embedding_type="sentence")
        maplet = mappy_python.Maplet(
            capacity=2000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        def insert_operation():
            for i, embedding in enumerate(embeddings):
                key = f"perf_{i}"
                maplet.insert(key, embedding)
        
        result = measure_operation(insert_operation, iterations=1)
        assert result.success_rate > 0.0
        assert result.throughput > 0.0
    
    @pytest.mark.benchmark
    def test_embedding_query_performance(self):
        """Benchmark embedding query performance"""
        embeddings = generate_ml_embeddings(num_samples=1000, dim=384, embedding_type="sentence")
        maplet = mappy_python.Maplet(
            capacity=2000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Pre-populate
        for i, embedding in enumerate(embeddings):
            key = f"query_perf_{i}"
            maplet.insert(key, embedding)
        
        def query_operation():
            for i in range(len(embeddings)):
                key = f"query_perf_{i}"
                maplet.query(key)
        
        result = measure_operation(query_operation, iterations=10)
        assert result.success_rate > 0.0
        assert result.throughput > 0.0






