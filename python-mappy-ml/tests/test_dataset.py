"""Tests for MappyMLDataset."""

import pytest
import numpy as np

try:
    import python_mappy_ml
    import mappy_python
    HAS_PYTHON_MAPPY_ML = True
except ImportError:
    HAS_PYTHON_MAPPY_ML = False

pytestmark = pytest.mark.skipif(
    not HAS_PYTHON_MAPPY_ML,
    reason="python-mappy-ml not installed"
)


def test_dataset_always_returns_tuple():
    """Test that MappyMLDataset.__getitem__ always returns (tensor, int) tuple."""
    # Create a simple maplet with test data
    maplet = mappy_python.Maplet(
        capacity=100,
        false_positive_rate=0.01,
        operator=mappy_python.VectorOperator()
    )
    
    # Store a test image (as bytes for now)
    import pickle
    test_image = np.random.randint(0, 255, (28, 28, 1), dtype=np.uint8)
    test_label = 5
    serialized = pickle.dumps((test_image, test_label))
    data_list = [float(b) for b in serialized]
    maplet.insert("test_0", data_list)
    
    # Create dataset
    dataset = python_mappy_ml.MappyMLDataset(
        maplet=maplet,
        keys=["test_0"],
        image_shape=[1, 28, 28],
        label_dtype="int64",
    )
    
    # Get item - should always return 2-tuple
    result = dataset[0]
    assert isinstance(result, tuple), f"Expected tuple, got {type(result)}"
    assert len(result) == 2, f"Expected tuple of length 2, got {len(result)}"
    
    image, label = result
    # Image should be a tensor (PyTorch tensor)
    # Label should be an integer
    assert isinstance(label, (int, np.integer)), f"Expected int label, got {type(label)}"



