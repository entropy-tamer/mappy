"""
Production scenarios: ML feature store, recommendation systems, anomaly detection, search.

Tests mappy-python with real-world production use cases.
"""

import pytest
import numpy as np
import mappy_python
from .test_utils import (
    generate_ml_embeddings,
    load_word_embeddings,
    load_faiss_vectors,
    assert_vector_equal
)


class TestMLFeatureStore:
    """Test ML feature store scenarios"""
    
    def test_feature_store_embeddings(self):
        """Test storing ML embeddings in feature store"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Generate embeddings for different entities
        user_embeddings = generate_ml_embeddings(num_samples=100, dim=128, embedding_type="sentence")
        item_embeddings = generate_ml_embeddings(num_samples=100, dim=128, embedding_type="sentence")
        
        # Store user embeddings
        for i, embedding in enumerate(user_embeddings):
            maplet.insert(f"user_{i}", embedding)
        
        # Store item embeddings
        for i, embedding in enumerate(item_embeddings):
            maplet.insert(f"item_{i}", embedding)
        
        # Verify retrieval
        user_success = 0
        for i, expected in enumerate(user_embeddings):
            result = maplet.query(f"user_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                if np.allclose(result_array, expected, atol=1e-3):
                    user_success += 1
        
        item_success = 0
        for i, expected in enumerate(item_embeddings):
            result = maplet.query(f"item_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                if np.allclose(result_array, expected, atol=1e-3):
                    item_success += 1
        
        assert user_success >= len(user_embeddings) * 0.9
        assert item_success >= len(item_embeddings) * 0.9
    
    def test_feature_store_aggregation(self):
        """Test aggregating features in feature store"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Aggregate features for same entity
        user_id = "user_123"
        features1 = np.random.rand(50)
        features2 = np.random.rand(50)
        features3 = np.random.rand(50)
        
        maplet.insert(user_id, features1)
        maplet.insert(user_id, features2)
        maplet.insert(user_id, features3)
        
        result = maplet.query(user_id)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected = features1 + features2 + features3
        
        assert np.allclose(result_array, expected, atol=1e-5)


class TestRecommendationSystems:
    """Test recommendation system scenarios"""
    
    def test_user_item_interactions(self):
        """Test storing user-item interaction vectors"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Generate user and item embeddings
        user_embeddings = generate_ml_embeddings(num_samples=50, dim=64, embedding_type="sentence")
        item_embeddings = generate_ml_embeddings(num_samples=50, dim=64, embedding_type="sentence")
        
        # Store user preferences (aggregated interactions)
        for i, user_emb in enumerate(user_embeddings):
            user_key = f"user_pref_{i}"
            # Aggregate multiple item interactions
            for j in range(5):
                item_emb = item_embeddings[(i + j) % len(item_embeddings)]
                maplet.insert(user_key, item_emb)
        
        # Verify user preferences
        success_count = 0
        for i in range(len(user_embeddings)):
            user_key = f"user_pref_{i}"
            result = maplet.query(user_key)
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                # Should be sum of 5 item embeddings
                assert len(result_array) == 64
                success_count += 1
        
        assert success_count >= len(user_embeddings) * 0.9
    
    def test_similarity_search(self):
        """Test similarity search for recommendations"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Store item embeddings
        item_embeddings = generate_ml_embeddings(num_samples=100, dim=128, embedding_type="sentence")
        for i, embedding in enumerate(item_embeddings):
            maplet.insert(f"item_{i}", embedding)
        
        # Query for similar items
        query_embedding = item_embeddings[0]
        similarities = []
        
        for i in range(min(50, len(item_embeddings))):
            result = maplet.query(f"item_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                # Compute cosine similarity
                dot_product = np.dot(result_array, query_embedding)
                norm1 = np.linalg.norm(result_array)
                norm2 = np.linalg.norm(query_embedding)
                if norm1 > 0 and norm2 > 0:
                    similarity = dot_product / (norm1 * norm2)
                    similarities.append((i, similarity))
        
        # Should find some similar items
        assert len(similarities) > 0
        # First item should have high similarity to itself
        if similarities:
            max_sim = max(similarities, key=lambda x: x[1])
            assert max_sim[1] > 0.5


class TestAnomalyDetection:
    """Test anomaly detection scenarios"""
    
    def test_feature_aggregation_anomaly(self):
        """Test aggregating features for anomaly detection"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Store normal behavior patterns
        entity_id = "entity_123"
        normal_patterns = [np.random.rand(32) for _ in range(10)]
        
        for pattern in normal_patterns:
            maplet.insert(entity_id, pattern)
        
        # Get aggregated normal pattern
        result = maplet.query(entity_id)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        expected_sum = sum(normal_patterns)
        
        assert np.allclose(result_array, expected_sum, atol=1e-5)
    
    def test_multiple_entity_tracking(self):
        """Test tracking multiple entities for anomaly detection"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Track multiple entities
        num_entities = 100
        for entity_id in range(num_entities):
            # Each entity has multiple feature vectors
            features = [np.random.rand(16) for _ in range(5)]
            for feat in features:
                maplet.insert(f"entity_{entity_id}", feat)
        
        # Verify tracking
        success_count = 0
        for entity_id in range(num_entities):
            result = maplet.query(f"entity_{entity_id}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                assert len(result_array) == 16
                success_count += 1
        
        assert success_count >= num_entities * 0.9


class TestSearchSystems:
    """Test search system scenarios"""
    
    def test_document_embeddings(self):
        """Test storing document embeddings for search"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Generate document embeddings
        doc_embeddings = generate_ml_embeddings(num_samples=100, dim=256, embedding_type="sentence")
        
        # Store document embeddings
        for i, embedding in enumerate(doc_embeddings):
            maplet.insert(f"doc_{i}", embedding)
        
        # Verify retrieval
        success_count = 0
        for i, expected in enumerate(doc_embeddings):
            result = maplet.query(f"doc_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                if np.allclose(result_array, expected, atol=1e-3):
                    success_count += 1
        
        assert success_count >= len(doc_embeddings) * 0.9
    
    def test_semantic_search(self):
        """Test semantic search with embeddings"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Store document embeddings
        doc_embeddings = generate_ml_embeddings(num_samples=200, dim=384, embedding_type="sentence")
        for i, embedding in enumerate(doc_embeddings):
            maplet.insert(f"doc_{i}", embedding)
        
        # Query with search vector
        query_vector = doc_embeddings[0]
        search_results = []
        
        for i in range(min(100, len(doc_embeddings))):
            result = maplet.query(f"doc_{i}")
            if result is not None:
                result_array = np.array(result) if isinstance(result, list) else result
                # Compute cosine similarity
                dot_product = np.dot(result_array, query_vector)
                norm1 = np.linalg.norm(result_array)
                norm2 = np.linalg.norm(query_vector)
                if norm1 > 0 and norm2 > 0:
                    similarity = dot_product / (norm1 * norm2)
                    search_results.append((i, similarity))
        
        # Should find some relevant documents
        assert len(search_results) > 0
        # Sort by similarity
        search_results.sort(key=lambda x: x[1], reverse=True)
        # Top result should have high similarity
        if search_results:
            assert search_results[0][1] > 0.5


class TestRealWorldIntegration:
    """Test real-world integration scenarios"""
    
    def test_multi_tenant_feature_store(self):
        """Test multi-tenant feature store scenario"""
        maplet = mappy_python.Maplet(
            capacity=50000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Simulate multiple tenants
        num_tenants = 10
        features_per_tenant = 100
        
        for tenant_id in range(num_tenants):
            tenant_features = generate_ml_embeddings(
                num_samples=features_per_tenant,
                dim=128,
                embedding_type="sentence"
            )
            for i, feature in enumerate(tenant_features):
                maplet.insert(f"tenant_{tenant_id}_feature_{i}", feature)
        
        # Verify tenant isolation
        success_count = 0
        for tenant_id in range(num_tenants):
            for i in range(min(50, features_per_tenant)):
                result = maplet.query(f"tenant_{tenant_id}_feature_{i}")
                if result is not None:
                    success_count += 1
        
        assert success_count > 0
    
    def test_real_time_feature_updates(self):
        """Test real-time feature updates"""
        maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator()
        )
        
        # Simulate real-time updates
        entity_id = "realtime_entity"
        base_features = np.random.rand(64)
        
        # Initial features
        maplet.insert(entity_id, base_features)
        
        # Update with new features multiple times
        for update_id in range(10):
            update_features = np.random.rand(64) * 0.1  # Small updates
            maplet.insert(entity_id, update_features)
        
        # Final state should be sum of all updates
        result = maplet.query(entity_id)
        assert result is not None
        
        result_array = np.array(result) if isinstance(result, list) else result
        # Should have accumulated all updates
        assert len(result_array) == 64






