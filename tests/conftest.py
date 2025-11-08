"""
Pytest configuration and fixtures for mappy-python tests
"""

import asyncio
import random
import string
import time
from typing import Generator, List, Tuple

import numpy as np
import pytest


class Stats:
    """Helper class to convert stats dict to object with attributes"""
    def __init__(self, stats_dict):
        if isinstance(stats_dict, dict):
            self.item_count = stats_dict.get("item_count", 0)
            self.load_factor = stats_dict.get("load_factor", 0.0)
            self.memory_usage = stats_dict.get("memory_usage", 0)
            self.false_positive_rate = stats_dict.get("false_positive_rate", stats_dict.get("error_rate", 0.0))
            self.error_rate = stats_dict.get("error_rate", 0.0)
        else:
            # If it's already an object, just copy attributes
            self.item_count = getattr(stats_dict, "item_count", 0)
            self.load_factor = getattr(stats_dict, "load_factor", 0.0)
            self.memory_usage = getattr(stats_dict, "memory_usage", 0)
            self.false_positive_rate = getattr(stats_dict, "false_positive_rate", getattr(stats_dict, "error_rate", 0.0))
            self.error_rate = getattr(stats_dict, "error_rate", 0.0)


@pytest.fixture
def sample_strings() -> List[str]:
    """Generate sample string data for testing"""
    return [
        "user:123",
        "session:abc",
        "key:xyz",
        "data:test",
        "cache:item",
    ] + [
        "".join(random.choices(string.ascii_letters + string.digits, k=20))
        for _ in range(95)
    ]


@pytest.fixture
def sample_integers() -> List[int]:
    """Generate sample integer data for testing"""
    return [random.randint(1, 1000) for _ in range(100)]


@pytest.fixture
def sample_floats() -> List[float]:
    """Generate sample float data for testing"""
    return [random.uniform(0.0, 100.0) for _ in range(100)]


@pytest.fixture
def sample_numpy_arrays() -> List[np.ndarray]:
    """Generate sample NumPy arrays for testing"""
    return [
        np.random.rand(10),
        np.random.randint(0, 100, 20),
        np.array([1.0, 2.0, 3.0, 4.0, 5.0]),
        np.zeros(15),
        np.ones(8),
    ]


@pytest.fixture
def dna_sequences() -> List[str]:
    """Generate DNA sequences for k-mer testing"""
    bases = ['A', 'T', 'C', 'G']
    return [
        ''.join(random.choices(bases, k=100))
        for _ in range(10)
    ]


@pytest.fixture
def network_traffic_data() -> List[Tuple[str, int]]:
    """Generate network traffic data for testing"""
    ip_addresses = [
        f"192.168.1.{i}" for i in range(1, 11)
    ] + [
        f"10.0.0.{i}" for i in range(1, 11)
    ]

    return [
        (ip, random.randint(100, 10000))
        for ip in ip_addresses
        for _ in range(5)  # Multiple entries per IP
    ]


@pytest.fixture
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
def performance_benchmark_data():
    """Generate large dataset for performance testing"""
    return {
        'keys': [f"key_{i}" for i in range(10000)],
        'values': [random.randint(1, 100) for _ in range(10000)],
        'batch_size': 1000,
    }


@pytest.fixture
def memory_test_data():
    """Generate data for memory usage testing"""
    return {
        'small': [(f"small_{i}", i) for i in range(100)],
        'medium': [(f"medium_{i}", i) for i in range(1000)],
        'large': [(f"large_{i}", i) for i in range(10000)],
    }


def pytest_configure(config):
    """Configure pytest with custom markers"""
    config.addinivalue_line(
        "markers", "slow: marks tests as slow (deselect with '-m \"not slow\"')"
    )
    config.addinivalue_line(
        "markers", "integration: marks tests as integration tests"
    )
    config.addinivalue_line(
        "markers", "benchmark: marks tests as benchmark tests"
    )
    config.addinivalue_line(
        "markers", "memory: marks tests that check memory usage"
    )
    config.addinivalue_line(
        "markers", "stress: marks tests as stress tests"
    )


def pytest_collection_modifyitems(config, items):
    """Modify test collection to add markers based on test names"""
    for item in items:
        if "benchmark" in item.nodeid:
            item.add_marker(pytest.mark.benchmark)
        if "memory" in item.nodeid:
            item.add_marker(pytest.mark.memory)
        if "integration" in item.nodeid:
            item.add_marker(pytest.mark.integration)
        if "slow" in item.nodeid or "performance" in item.nodeid:
            item.add_marker(pytest.mark.slow)

