#!/usr/bin/env python3
"""Benchmark Rust-native pipeline vs PIL+numpy+torch baseline."""

import time
import numpy as np
from PIL import Image
import torch

try:
    import python_mappy_ml
    HAS_RUST = True
except ImportError:
    HAS_RUST = False
    print("Warning: python-mappy-ml not available, only baseline will be tested")


def benchmark_baseline(num_images=1000):
    """Benchmark PIL+numpy+torch baseline."""
    images = []
    for i in range(num_images):
        img = Image.new('RGB', (224, 224), color=(i % 255, (i * 2) % 255, (i * 3) % 255))
        images.append(img)
    
    start = time.time()
    tensors = []
    for img in images:
        # PIL -> numpy -> torch
        img_np = np.array(img)
        img_tensor = torch.from_numpy(img_np).permute(2, 0, 1).float() / 255.0
        tensors.append(img_tensor)
    elapsed = time.time() - start
    
    return elapsed, len(tensors)


def benchmark_rust(num_images=1000):
    """Benchmark Rust-native pipeline."""
    if not HAS_RUST:
        return None, 0
    
    # Create image bytes
    images_bytes = []
    for i in range(num_images):
        img = Image.new('RGB', (224, 224), color=(i % 255, (i * 2) % 255, (i * 3) % 255))
        import io
        buf = io.BytesIO()
        img.save(buf, format='PNG')
        images_bytes.append(buf.getvalue())
    
    start = time.time()
    tensors = []
    for img_bytes in images_bytes:
        # Rust decode -> tensor
        decoded = python_mappy_ml.decode_image(img_bytes)
        # Convert to tensor (would use to_tensor in real usage)
        img_tensor = torch.from_numpy(decoded).permute(2, 0, 1).float() / 255.0
        tensors.append(img_tensor)
    elapsed = time.time() - start
    
    return elapsed, len(tensors)


if __name__ == "__main__":
    num_images = 1000
    
    print(f"Benchmarking with {num_images} images...")
    
    # Baseline
    baseline_time, baseline_count = benchmark_baseline(num_images)
    print(f"Baseline (PIL+numpy+torch): {baseline_time:.3f}s, {baseline_count} images")
    
    # Rust
    if HAS_RUST:
        rust_time, rust_count = benchmark_rust(num_images)
        if rust_time:
            print(f"Rust-native: {rust_time:.3f}s, {rust_count} images")
            speedup = baseline_time / rust_time
            print(f"Speedup: {speedup:.2f}x")
    else:
        print("Rust-native: Not available")



