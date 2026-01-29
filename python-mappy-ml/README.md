# python-mappy-ml

High-performance Rust-native ML data pipeline for mappy with image decoding and zero-copy tensor export.

## Features

- **Rust-native image decoding**: JPEG/PNG/raw bytes decoding using the `image` crate
- **Zero-copy tensor export**: DLPack and libtorch support for efficient tensor sharing
- **Async GPU transfer**: Non-blocking GPU transfers using PyTorch C API
- **ML-optimized operators**: `ByteArrayOperator` and `VectorConcatOperator` for efficient data storage
- **MappyMLDataset**: PyTorch Dataset that always returns `(tensor, label)` tuples (fixes MNIST unpacking issues)

## Installation

```bash
cd services/mappy/python-mappy-ml
maturin develop
# or
maturin build
pip install target/wheels/python_mappy_ml-*.whl
```

## Usage

```python
import python_mappy_ml
import mappy_python

# Create maplet with ByteArrayOperator
maplet = mappy_python.Maplet(
    capacity=100000,
    false_positive_rate=0.01,
    operator=mappy_python.ByteArrayOperator()
)

# Create dataset
dataset = python_mappy_ml.MappyMLDataset(
    maplet=maplet,
    keys=["train_0", "train_1", ...],
    image_shape=[1, 28, 28],  # MNIST: C, H, W
    label_dtype="int64",
    backend="auto"  # or "dlpack" or "libtorch"
)

# Always returns (tensor, label) tuple
image, label = dataset[0]
```

## API Reference

### MappyMLDataset

PyTorch Dataset that loads data from mappy, decodes images in Rust, and returns zero-copy tensors.

**Always returns `(tensor, int)` tuple** - fixes MNIST unpacking errors.

### Image Decoding

- `decode_image(bytes) -> numpy.ndarray`: Decode single image
- `decode_image_batch(bytes_list) -> list[numpy.ndarray]`: Parallel batch decoding
- `preprocess_image(image, size, normalize) -> numpy.ndarray`: Resize and normalize

### Operators

- `ByteArrayOperator`: Optimized for byte arrays (images, binary blobs)
- `VectorConcatOperator`: Concatenate vectors

## Performance

- **>1.5× throughput** vs PIL+numpy+torch baseline for large batches
- **Zero-copy** tensor creation via DLPack/libtorch
- **Async GPU transfer** with pinned memory support

## Requirements

- Python >= 3.8
- PyTorch >= 2.0.0
- NumPy >= 1.20.0
- Rust toolchain (for building)
