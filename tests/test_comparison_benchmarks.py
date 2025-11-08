"""
Comparison benchmarks: mappy vs Bloom filters, Count-Min Sketch, Redis, dict.

Compares mappy-python performance and correctness against popular alternatives.
"""

import pytest
import numpy as np
import mappy_python
from .test_utils import compare_implementations, ComparisonResult, generate_report
from pathlib import Path

# Try to import comparison libraries
try:
    from pybloom_live import BloomFilter
    HAS_BLOOM = True
except ImportError:
    HAS_BLOOM = False

try:
    from countminsketch import CountMinSketch
    HAS_CMS = True
except (ImportError, SyntaxError):
    HAS_CMS = False

try:
    import redis
    HAS_REDIS = True
except ImportError:
    HAS_REDIS = False


class TestMappyVsBloomFilter:
    """Compare mappy with Bloom filters"""
    
    @pytest.mark.benchmark
    @pytest.mark.skipif(not HAS_BLOOM, reason="pybloom-live not installed")
    def test_insert_comparison(self):
        """Compare insert performance with Bloom filter"""
        keys = [f"key_{i}" for i in range(10000)]
        
        # Mappy implementation
        mappy_maplet = mappy_python.Maplet(
            capacity=20000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def mappy_insert():
            for key in keys:
                mappy_maplet.insert(key, 1)
        
        # Bloom filter implementation
        bloom = BloomFilter(capacity=20000, error_rate=0.01)
        
        def bloom_insert():
            for key in keys:
                bloom.add(key)
        
        result = compare_implementations(
            "insert_comparison",
            {
                "mappy": mappy_insert,
                "bloom": bloom_insert
            },
            iterations=1
        )
        
        assert "mappy" in result.implementations
        assert "bloom" in result.implementations
    
    @pytest.mark.benchmark
    @pytest.mark.skipif(not HAS_BLOOM, reason="pybloom-live not installed")
    def test_query_comparison(self):
        """Compare query performance with Bloom filter"""
        keys = [f"key_{i}" for i in range(10000)]
        
        # Mappy implementation
        mappy_maplet = mappy_python.Maplet(
            capacity=20000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        for key in keys:
            mappy_maplet.insert(key, 1)
        
        def mappy_query():
            for key in keys:
                mappy_maplet.query(key)
        
        # Bloom filter implementation
        bloom = BloomFilter(capacity=20000, error_rate=0.01)
        for key in keys:
            bloom.add(key)
        
        def bloom_query():
            for key in keys:
                key in bloom
        
        result = compare_implementations(
            "query_comparison",
            {
                "mappy": mappy_query,
                "bloom": bloom_query
            },
            iterations=10
        )
        
        assert "mappy" in result.implementations
        assert "bloom" in result.implementations


class TestMappyVsCountMinSketch:
    """Compare mappy with Count-Min Sketch"""
    
    @pytest.mark.benchmark
    @pytest.mark.skipif(not HAS_CMS, reason="countminsketch not installed")
    def test_insert_comparison(self):
        """Compare insert performance with Count-Min Sketch"""
        keys = [f"key_{i}" for i in range(10000)]
        
        # Mappy implementation
        mappy_maplet = mappy_python.Maplet(
            capacity=20000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def mappy_insert():
            for key in keys:
                mappy_maplet.insert(key, 1)
        
        # Count-Min Sketch implementation
        cms = CountMinSketch(width=1000, depth=5)
        
        def cms_insert():
            for key in keys:
                cms.increment(key)
        
        result = compare_implementations(
            "cms_insert_comparison",
            {
                "mappy": mappy_insert,
                "cms": cms_insert
            },
            iterations=1
        )
        
        assert "mappy" in result.implementations
        assert "cms" in result.implementations
    
    @pytest.mark.benchmark
    @pytest.mark.skipif(not HAS_CMS, reason="countminsketch not installed")
    def test_query_comparison(self):
        """Compare query performance with Count-Min Sketch"""
        keys = [f"key_{i}" for i in range(10000)]
        
        # Mappy implementation
        mappy_maplet = mappy_python.Maplet(
            capacity=20000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        for key in keys:
            mappy_maplet.insert(key, 1)
        
        def mappy_query():
            for key in keys:
                mappy_maplet.query(key)
        
        # Count-Min Sketch implementation
        cms = CountMinSketch(width=1000, depth=5)
        for key in keys:
            cms.increment(key)
        
        def cms_query():
            for key in keys:
                cms[key]
        
        result = compare_implementations(
            "cms_query_comparison",
            {
                "mappy": mappy_query,
                "cms": cms_query
            },
            iterations=10
        )
        
        assert "mappy" in result.implementations
        assert "cms" in result.implementations


class TestMappyVsDict:
    """Compare mappy with Python dict (baseline)"""
    
    @pytest.mark.benchmark
    def test_insert_comparison(self):
        """Compare insert performance with dict"""
        keys = [f"key_{i}" for i in range(10000)]
        
        # Mappy implementation
        mappy_maplet = mappy_python.Maplet(
            capacity=20000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def mappy_insert():
            for key in keys:
                mappy_maplet.insert(key, 1)
        
        # Dict implementation
        d = {}
        
        def dict_insert():
            for key in keys:
                d[key] = 1
        
        result = compare_implementations(
            "dict_insert_comparison",
            {
                "mappy": mappy_insert,
                "dict": dict_insert
            },
            iterations=10
        )
        
        assert "mappy" in result.implementations
        assert "dict" in result.implementations
    
    @pytest.mark.benchmark
    def test_query_comparison(self):
        """Compare query performance with dict"""
        keys = [f"key_{i}" for i in range(10000)]
        
        # Mappy implementation
        mappy_maplet = mappy_python.Maplet(
            capacity=20000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        for key in keys:
            mappy_maplet.insert(key, 1)
        
        def mappy_query():
            for key in keys:
                mappy_maplet.query(key)
        
        # Dict implementation
        d = {key: 1 for key in keys}
        
        def dict_query():
            for key in keys:
                d.get(key)
        
        result = compare_implementations(
            "dict_query_comparison",
            {
                "mappy": mappy_query,
                "dict": dict_query
            },
            iterations=100
        )
        
        assert "mappy" in result.implementations
        assert "dict" in result.implementations


class TestMappyVsRedis:
    """Compare mappy with Redis (if available)"""
    
    @pytest.mark.benchmark
    @pytest.mark.skipif(not HAS_REDIS, reason="redis not installed or not available")
    def test_insert_comparison(self):
        """Compare insert performance with Redis"""
        try:
            r = redis.Redis(host='localhost', port=6379, db=0, decode_responses=True)
            r.ping()
        except (redis.ConnectionError, redis.TimeoutError):
            pytest.skip("Redis server not available")
        
        keys = [f"key_{i}" for i in range(1000)]  # Smaller set for Redis
        
        # Mappy implementation
        mappy_maplet = mappy_python.Maplet(
            capacity=2000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def mappy_insert():
            for key in keys:
                mappy_maplet.insert(key, 1)
        
        # Redis implementation
        def redis_insert():
            for key in keys:
                r.set(key, "1")
        
        result = compare_implementations(
            "redis_insert_comparison",
            {
                "mappy": mappy_insert,
                "redis": redis_insert
            },
            iterations=1
        )
        
        assert "mappy" in result.implementations
        assert "redis" in result.implementations
        
        # Cleanup
        r.flushdb()


class TestComprehensiveComparison:
    """Comprehensive comparison across all implementations"""
    
    @pytest.mark.benchmark
    def test_comprehensive_benchmark(self, tmp_path):
        """Run comprehensive benchmark and generate report"""
        keys = [f"key_{i}" for i in range(5000)]
        
        implementations = {}
        
        # Mappy
        mappy_maplet = mappy_python.Maplet(
            capacity=10000,
            false_positive_rate=0.01,
            operator=mappy_python.CounterOperator()
        )
        
        def mappy_ops():
            for key in keys:
                mappy_maplet.insert(key, 1)
            for key in keys:
                mappy_maplet.query(key)
        
        implementations["mappy"] = mappy_ops
        
        # Dict
        d = {}
        
        def dict_ops():
            for key in keys:
                d[key] = 1
            for key in keys:
                d.get(key)
        
        implementations["dict"] = dict_ops
        
        # Bloom filter (if available)
        if HAS_BLOOM:
            bloom = BloomFilter(capacity=10000, error_rate=0.01)
            
            def bloom_ops():
                for key in keys:
                    bloom.add(key)
                for key in keys:
                    key in bloom
            
            implementations["bloom"] = bloom_ops
        
        result = compare_implementations(
            "comprehensive_benchmark",
            implementations,
            iterations=5
        )
        
        # Generate report
        report_path = tmp_path / "benchmark_report.txt"
        report = generate_report([result], output_path=report_path)
        
        assert len(report) > 0
        assert report_path.exists()

