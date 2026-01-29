"""PyTorch DataLoader wrapper for mappy storage.

Implements efficient data flow: mappy -> contiguous memory -> tensor view -> batch
This provides zero-copy tensor views and efficient GPU batching.
"""

import logging
import pickle
import threading
import time
from collections.abc import Callable
from typing import Any

import numpy as np
import torch
from torch.utils.data import DataLoader, Dataset

try:
    import mappy_python
except ImportError:
    mappy_python = None

logger = logging.getLogger(__name__)


class ContiguousMappyDataset(Dataset):
    """PyTorch Dataset that loads data from mappy into contiguous memory.

    Data flow: mappy -> contiguous buffer -> tensor views -> batches
    This provides zero-copy tensor views and efficient GPU transfer.
    """

    def __init__(
        self,
        maplet: Any,  # mappy_python.PyMaplet
        keys: list[str],
        image_shape: tuple[int, ...],
        label_dtype: torch.dtype = torch.long,
        transform: Callable | None = None,
    ):
        """Initialize ContiguousMappyDataset.

        Args:
            maplet: mappy_python.PyMaplet instance containing the data
            keys: List of keys to retrieve from maplet
            image_shape: Shape of each image (e.g., (1, 28, 28) for MNIST)
            label_dtype: Data type for labels
            transform: Optional transform to apply to loaded data

        """
        if mappy_python is None:
            msg = "mappy_python is required. Install with: pip install -e services/mappy/mappy-python"
            raise ImportError(msg)

        if maplet is None:
            msg = "maplet cannot be None"
            raise ValueError(msg)

        self.maplet = maplet
        self.keys = keys
        self.image_shape = image_shape
        self.label_dtype = label_dtype
        self.transform = transform

        # Calculate sizes
        self.image_size = int(np.prod(image_shape))
        self.num_samples = len(keys)

        # Load all data into contiguous memory buffers
        logger.info(f"Loading {self.num_samples} samples into contiguous memory...")
        self._load_contiguous_data()
        logger.info(f"Loaded data into contiguous buffers: images={self.images.shape}, labels={self.labels.shape}")

    def _load_contiguous_data(self):
        """Load all data from mappy into contiguous memory buffers."""
        # Pre-allocate contiguous buffers
        images_buffer = np.zeros((self.num_samples, self.image_size), dtype=np.float32)
        labels_buffer = np.zeros(self.num_samples, dtype=np.int64)

        failed_indices = []
        
        # Diagnostic: check first few items to understand data format
        if len(self.keys) > 0:
            sample_key = self.keys[0]
            sample_data = self.maplet.query(sample_key)
            logger.debug(f"Sample data from mappy - key: {sample_key}, type: {type(sample_data)}, "
                        f"len: {len(sample_data) if hasattr(sample_data, '__len__') else 'N/A'}")
            if isinstance(sample_data, list) and len(sample_data) > 0:
                logger.debug(f"Sample data first 10 values: {sample_data[:10]}")
                sample_bytes = bytes([int(x) & 0xFF for x in sample_data[:20]])
                logger.debug(f"Sample reconstructed bytes (first 20): {list(sample_bytes)}")
                try:
                    sample_unpickled = pickle.loads(bytes([int(x) & 0xFF for x in sample_data]))
                    logger.debug(f"Sample unpickled type: {type(sample_unpickled)}, "
                               f"len: {len(sample_unpickled) if hasattr(sample_unpickled, '__len__') else 'N/A'}")
                except Exception as e:
                    logger.debug(f"Sample unpickling failed: {e}")
        
        for idx, key in enumerate(self.keys):
            try:
                # Query from mappy
                data = self.maplet.query(key)
                if data is None:
                    failed_indices.append((idx, key, "query returned None"))
                    continue

                # Handle different data formats from mappy
                # Data might be: list of floats, bytes, or already unpickled
                if isinstance(data, bytes):
                    # Data is already bytes - try to unpickle directly
                    data_bytes = data
                elif isinstance(data, list) and len(data) > 0:
                    # Data is list of floats representing pickled bytes
                    if all(isinstance(x, int | float) for x in data):
                        # Reconstruct bytes from list of floats
                        data_bytes = bytes([int(x) & 0xFF for x in data])
                    else:
                        failed_indices.append((idx, key, "data contains non-numeric values"))
                        continue
                else:
                    # Data might already be unpickled
                    if isinstance(data, tuple) and len(data) == 2:
                        # Data is already a tuple - use it directly
                        image, label = data
                        # Skip to image processing
                        if isinstance(image, torch.Tensor):
                            image_np = image.detach().cpu().numpy()
                        elif isinstance(image, np.ndarray):
                            image_np = image
                        else:
                            failed_indices.append((idx, key, f"unexpected image type: {type(image)}"))
                            continue
                        
                        image_flat = image_np.flatten().astype(np.float32)
                        if len(image_flat) != self.image_size:
                            failed_indices.append((idx, key, f"image size mismatch: {len(image_flat)} != {self.image_size}"))
                            continue
                        
                        images_buffer[idx] = image_flat
                        labels_buffer[idx] = int(label)
                        continue
                    else:
                        failed_indices.append((idx, key, f"unexpected data format: type={type(data)}, len={len(data) if hasattr(data, '__len__') else 'N/A'}"))
                        continue

                # Now unpickle the bytes
                try:
                    unpickled = pickle.loads(data_bytes)
                    
                    # Verify we got a tuple
                    if not isinstance(unpickled, tuple):
                        # If we got bytes, it means we stored bytes directly (not a tuple)
                        # This shouldn't happen, but let's handle it
                        if isinstance(unpickled, bytes):
                            # Try to unpickle the bytes again (maybe it's double-pickled?)
                            try:
                                unpickled = pickle.loads(unpickled)
                                if not isinstance(unpickled, tuple):
                                    failed_indices.append((idx, key, f"double-unpickled data is still not a tuple, got {type(unpickled)}"))
                                    continue
                            except Exception:
                                failed_indices.append((idx, key, f"unpickle returned bytes - data may be corrupted or stored incorrectly"))
                                continue
                        else:
                            failed_indices.append((idx, key, f"unpickled data is not a tuple, got {type(unpickled)}"))
                            continue
                    
                    # Verify tuple has exactly 2 elements
                    if len(unpickled) != 2:
                        # Try to handle edge cases - maybe it's (image, label, extra_data)?
                        if len(unpickled) > 2:
                            # Take first two elements
                            logger.warning(f"Key {key}: tuple has {len(unpickled)} elements, using first 2")
                            image, label = unpickled[0], unpickled[1]
                        else:
                            failed_indices.append((idx, key, f"unpickled tuple has {len(unpickled)} elements, expected 2"))
                            continue
                    else:
                        image, label = unpickled
                    
                except pickle.UnpicklingError as e:
                    failed_indices.append((idx, key, f"pickle unpickle error: {e}"))
                    continue
                except Exception as e:
                    failed_indices.append((idx, key, f"unpickle error: {type(e).__name__}: {e}"))
                    continue

                # Extract image and label
                if isinstance(image, torch.Tensor):
                    image_np = image.detach().cpu().numpy()
                elif isinstance(image, np.ndarray):
                    image_np = image
                else:
                    failed_indices.append((idx, key, f"unexpected image type: {type(image)}"))
                    continue

                # Flatten and store in contiguous buffer
                image_flat = image_np.flatten().astype(np.float32)
                if len(image_flat) != self.image_size:
                    failed_indices.append((idx, key, f"image size mismatch: {len(image_flat)} != {self.image_size}"))
                    continue

                images_buffer[idx] = image_flat
                labels_buffer[idx] = int(label)

                if (idx + 1) % 10000 == 0:
                    logger.debug(f"Loaded {idx + 1}/{self.num_samples} samples...")

            except Exception as e:
                failed_indices.append((idx, key, str(e)))
                continue

        if failed_indices:
            logger.error(f"Failed to load {len(failed_indices)} items out of {self.num_samples}")
            
            # Group errors by type for better diagnostics
            error_counts = {}
            for _, _, error_msg in failed_indices:
                error_type = error_msg.split(':')[0] if ':' in error_msg else error_msg
                error_counts[error_type] = error_counts.get(error_type, 0) + 1
            
            logger.error(f"Error breakdown: {error_counts}")
            
            if len(failed_indices) <= 20:
                logger.error(f"All failed indices: {failed_indices}")
            else:
                logger.error(f"First 20 failed indices: {failed_indices[:20]}")
                logger.error(f"Last 10 failed indices: {failed_indices[-10:]}")
            
            # If too many failures, raise error
            if len(failed_indices) > self.num_samples * 0.05:  # More than 5% failure
                raise RuntimeError(f"Failed to load {len(failed_indices)} items ({100*len(failed_indices)/self.num_samples:.1f}%). "
                                 f"This suggests data was not properly stored in maplet.")
            else:
                # If only a few failures, log warning but continue
                logger.warning(f"Continuing with {self.num_samples - len(failed_indices)} items despite {len(failed_indices)} failures")
                # Remove failed indices from buffers (set to zero)
                for idx, _, _ in failed_indices:
                    images_buffer[idx] = 0
                    labels_buffer[idx] = 0

        # Create tensor views (zero-copy from numpy arrays)
        # Reshape images to original shape
        images_reshaped = images_buffer.reshape((self.num_samples,) + self.image_shape)
        self.images = torch.from_numpy(images_reshaped)  # Zero-copy view
        self.labels = torch.from_numpy(labels_buffer).to(self.label_dtype)  # Zero-copy view

        logger.debug(f"Created tensor views: images={self.images.shape}, labels={self.labels.shape}")

    def __len__(self) -> int:
        """Return dataset size."""
        return self.num_samples

    def __getitem__(self, idx: int) -> tuple[torch.Tensor, torch.Tensor]:
        """Get item by index (returns tensor views, no copy)."""
        image = self.images[idx]
        label = self.labels[idx]

        if self.transform is not None:
            image = self.transform(image)

        return image, label


class MappyDataset(Dataset):
    """Legacy PyTorch Dataset for backward compatibility.

    Uses individual queries per item. For better performance, use ContiguousMappyDataset.
    """

    def __init__(
        self,
        maplet: Any,
        keys: list[str],
        transform: Callable | None = None,
        cache_data: bool = True,
        decompress_fn: Callable[[bytes], Any] | None = None,
    ):
        """Initialize MappyDataset (legacy)."""
        if mappy_python is None:
            msg = "mappy_python is required. Install with: pip install -e services/mappy/mappy-python"
            raise ImportError(msg)

        if maplet is None:
            msg = "maplet cannot be None"
            raise ValueError(msg)

        self.maplet = maplet
        self.keys = keys
        self.transform = transform
        self.cache_data = cache_data
        self.decompress_fn = decompress_fn

        self._cache: dict[int, Any] = {}
        self._cache_lock = threading.Lock()

        if self.cache_data:
            self._populate_cache()

    def _populate_cache(self):
        """Pre-populate cache."""
        logger.debug(f"Populating cache for {len(self.keys)} items...")
        for idx in range(len(self.keys)):
            try:
                data = self._load_item(idx)
                with self._cache_lock:
                    self._cache[idx] = data
            except Exception as e:
                logger.warning(f"Failed to cache item {idx}: {e}")

    def _load_item(self, idx: int) -> Any:
        """Load a single item from mappy."""
        with self._cache_lock:
            if idx in self._cache:
                return self._cache[idx]

        key = self.keys[idx]
        data = self.maplet.query(key)
        if data is None:
            raise KeyError(f"Key '{key}' not found in maplet")

        if isinstance(data, list) and all(isinstance(x, int | float) for x in data):
            data_bytes = bytes([int(x) & 0xFF for x in data])
            data = pickle.loads(data_bytes)

        if self.transform is not None:
            data = self.transform(data)

        if self.cache_data:
            with self._cache_lock:
                self._cache[idx] = data

        return data

    def __len__(self) -> int:
        return len(self.keys)

    def __getitem__(self, idx: int) -> Any:
        return self._load_item(idx)


def create_contiguous_collate_fn(image_shape: tuple[int, ...]):
    """Create collate function for contiguous dataset.

    Professional ML best practice: Efficient batching with proper tensor stacking.
    Uses torch.stack which is optimized for contiguous memory layouts.

    Returns:
        Collate function that efficiently batches tensor views
    """
    def collate_fn(batch: list[tuple[torch.Tensor, torch.Tensor]]) -> tuple[torch.Tensor, torch.Tensor]:
        """Collate function for contiguous dataset.
        
        Professional best practices:
        - torch.stack() is optimized for contiguous tensors
        - No unnecessary copies (tensors are views)
        - Efficient memory layout for GPU transfer
        """
        images, labels = zip(*batch, strict=False)
        # Stack creates a new tensor, but from contiguous views this is efficient
        # torch.stack preserves memory layout and is optimized for GPU transfer
        images_batch = torch.stack(images)
        labels_batch = torch.stack(labels)
        return images_batch, labels_batch
    return collate_fn


def _serialize_value(
    value: Any,
    compress: bool = False,
    compress_fn: Callable[[Any], bytes] | None = None,
) -> list[float]:
    """Serialize a value to a list of floats for storage in mappy."""
    if isinstance(value, np.ndarray):
        return value.tolist()
    elif isinstance(value, torch.Tensor):
        return value.detach().cpu().numpy().tolist()
    elif isinstance(value, list | tuple):
        return list(value) if isinstance(value, tuple) else value
    else:
        serialized = pickle.dumps(value)
        if compress and compress_fn is not None:
            serialized = compress_fn(serialized)
        return [float(b) for b in serialized]


def store_data_in_mappy(
    maplet: Any,
    data_items: list[tuple[str, Any]] | dict[str, Any],
    compress: bool = False,
    compress_fn: Callable[[Any], bytes] | None = None,
) -> list[str]:
    """Store data in mappy maplet and return keys."""
    if isinstance(data_items, dict):
        items = list(data_items.items())
    else:
        items = data_items

    keys = []
    failed_keys = []
    total_items = len(items)

    logger.debug(f"Storing {total_items} items in maplet...")
    
    for idx, (key, value) in enumerate(items):
        try:
            data = _serialize_value(value, compress=compress, compress_fn=compress_fn)
            maplet.insert(key, data)
            
            # Verify insert
            verify_result = None
            for attempt in range(3):
                verify_result = maplet.query(key)
                if verify_result is not None:
                    break
                time.sleep(0.001)
            
            if verify_result is None or len(verify_result) != len(data):
                failed_keys.append(key)
                continue
            
            keys.append(key)
            
            if (idx + 1) % 10000 == 0:
                logger.debug(f"Stored and verified {idx + 1}/{total_items} items...")
                
        except Exception as e:
            failed_keys.append(key)
            logger.warning(f"Failed to store key '{key}': {e}")
            continue

    if failed_keys:
        logger.warning(f"Failed to store {len(failed_keys)} keys out of {total_items} total")

    if len(keys) < total_items * 0.95:
        logger.error(f"Storage success rate too low: {len(keys)}/{total_items} ({100*len(keys)/total_items:.1f}%)")
        raise RuntimeError(f"Failed to store sufficient data: only {len(keys)}/{total_items} items stored successfully.")
    
    logger.info(f"Successfully stored {len(keys)} items in maplet ({100*len(keys)/total_items:.1f}% success rate)")
    return keys
