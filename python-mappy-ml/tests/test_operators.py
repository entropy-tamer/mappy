"""Tests for mappy operators (VectorConcatOperator, ByteArrayOperator)."""

import pytest

try:
    import python_mappy_ml
    HAS_PYTHON_MAPPY_ML = True
except ImportError:
    HAS_PYTHON_MAPPY_ML = False

pytestmark = pytest.mark.skipif(
    not HAS_PYTHON_MAPPY_ML,
    reason="python-mappy-ml not installed"
)


def test_vector_concat_operator():
    """Test VectorConcatOperator."""
    op = python_mappy_ml.PyVectorConcatOperator()
    assert op is not None


def test_byte_array_operator():
    """Test ByteArrayOperator."""
    op = python_mappy_ml.PyByteArrayOperator()
    assert op is not None



