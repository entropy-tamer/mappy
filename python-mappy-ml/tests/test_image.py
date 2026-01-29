"""Tests for image decoding functions."""

import pytest
import numpy as np

try:
    import python_mappy_ml
    HAS_PYTHON_MAPPY_ML = True
except ImportError:
    HAS_PYTHON_MAPPY_ML = False

pytestmark = pytest.mark.skipif(
    not HAS_PYTHON_MAPPY_ML,
    reason="python-mappy-ml not installed"
)


def test_decode_image():
    """Test decode_image function."""
    # Create a simple test image (PNG bytes)
    from PIL import Image
    import io
    
    # Create a 28x28 grayscale image
    img = Image.new('L', (28, 28), color=128)
    buf = io.BytesIO()
    img.save(buf, format='PNG')
    image_bytes = buf.getvalue()
    
    # Decode
    decoded = python_mappy_ml.decode_image(image_bytes)
    assert decoded is not None
    # Should be a numpy array
    assert hasattr(decoded, 'shape')


def test_preprocess_image():
    """Test preprocess_image function."""
    # Create a test numpy array
    test_image = np.random.randint(0, 255, (28, 28, 3), dtype=np.uint8)
    
    # Preprocess
    preprocessed = python_mappy_ml.preprocess_image(
        test_image,
        size=(224, 224),
        normalize=True
    )
    assert preprocessed is not None
    assert hasattr(preprocessed, 'shape')



