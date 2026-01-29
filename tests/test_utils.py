"""Test utilities for comprehensive mappy-python testing suite.

Provides dataset loading, performance measurement, comparison framework,
and reporting utilities for ML embeddings, real datasets, and benchmarks.
"""

import contextlib
import json
import statistics
import time
from collections.abc import Callable
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import numpy as np


@dataclass
class BenchmarkResult:
    """Container for benchmark results."""

    name: str
    operation: str
    count: int
    total_time: float
    avg_time: float
    min_time: float
    max_time: float
    median_time: float
    p95_time: float
    p99_time: float
    throughput: float  # operations per second
    memory_usage: int | None = None
    error_count: int = 0
    success_rate: float = 1.0

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class ComparisonResult:
    """Container for comparison results between different implementations."""

    test_name: str
    implementations: dict[str, BenchmarkResult]
    winner: str | None = None
    speedup: float | None = None

    def to_dict(self) -> dict[str, Any]:
        result = asdict(self)
        result["implementations"] = {
            k: v.to_dict() if hasattr(v, "to_dict") else v
            for k, v in result["implementations"].items()
        }
        return result


class PerformanceTimer:
    """Context manager for timing operations."""

    def __init__(self):
        self.start_time = None
        self.end_time = None
        self.elapsed = None

    def __enter__(self):
        self.start_time = time.perf_counter()
        return self

    def __exit__(self, *args):
        self.end_time = time.perf_counter()
        self.elapsed = self.end_time - self.start_time


def measure_operation(
    operation: Callable,
    iterations: int = 1000,
    warmup: int = 10,
    collect_timings: bool = True,
) -> BenchmarkResult:
    """Measure performance of an operation.

    Args:
        operation: Callable to measure
        iterations: Number of iterations to run
        warmup: Number of warmup iterations
        collect_timings: Whether to collect individual timings

    Returns:
        BenchmarkResult with performance metrics

    """
    # Warmup
    for _ in range(warmup):
        with contextlib.suppress(Exception):
            operation()

    timings = []
    error_count = 0

    for _ in range(iterations):
        start = time.perf_counter()
        try:
            operation()
            elapsed = time.perf_counter() - start
            if collect_timings:
                timings.append(elapsed)
        except Exception:
            error_count += 1

    if not timings:
        return BenchmarkResult(
            name="unknown",
            operation="unknown",
            count=iterations,
            total_time=0.0,
            avg_time=0.0,
            min_time=0.0,
            max_time=0.0,
            median_time=0.0,
            p95_time=0.0,
            p99_time=0.0,
            throughput=0.0,
            error_count=error_count,
            success_rate=0.0,
        )

    total_time = sum(timings)
    avg_time = total_time / len(timings)
    min_time = min(timings)
    max_time = max(timings)
    median_time = statistics.median(timings)

    sorted_timings = sorted(timings)
    p95_idx = int(len(sorted_timings) * 0.95)
    p99_idx = int(len(sorted_timings) * 0.99)
    p95_time = (
        sorted_timings[p95_idx]
        if p95_idx < len(sorted_timings)
        else sorted_timings[-1]
    )
    p99_time = (
        sorted_timings[p99_idx]
        if p99_idx < len(sorted_timings)
        else sorted_timings[-1]
    )

    throughput = len(timings) / total_time if total_time > 0 else 0.0
    success_rate = len(timings) / iterations

    return BenchmarkResult(
        name="operation",
        operation="unknown",
        count=iterations,
        total_time=total_time,
        avg_time=avg_time,
        min_time=min_time,
        max_time=max_time,
        median_time=median_time,
        p95_time=p95_time,
        p99_time=p99_time,
        throughput=throughput,
        error_count=error_count,
        success_rate=success_rate,
    )


def load_mnist_vectors(num_samples: int = 1000, dim: int = 784) -> list[np.ndarray]:
    """Generate synthetic MNIST-like vectors.

    Args:
        num_samples: Number of samples to generate
        dim: Dimension of each vector (default 784 for 28x28 images)

    Returns:
        List of numpy arrays representing flattened images

    """
    # Generate synthetic MNIST-like data (normalized pixel values)
    vectors = []
    for _ in range(num_samples):
        # Simulate normalized pixel values (0-1 range)
        vector = np.random.rand(dim).astype(np.float32)
        vectors.append(vector)
    return vectors


def load_cifar10_vectors(num_samples: int = 1000, dim: int = 3072) -> list[np.ndarray]:
    """Generate synthetic CIFAR-10-like vectors.

    Args:
        num_samples: Number of samples to generate
        dim: Dimension of each vector (default 3072 for 32x32x3 images)

    Returns:
        List of numpy arrays representing flattened images

    """
    vectors = []
    for _ in range(num_samples):
        # Simulate RGB pixel values (0-255 range, normalized)
        vector = (np.random.rand(dim) * 255).astype(np.float32) / 255.0
        vectors.append(vector)
    return vectors


def load_word_embeddings(num_samples: int = 1000, dim: int = 300) -> list[np.ndarray]:
    """Generate synthetic word embedding vectors (Word2Vec/GloVe style).

    Args:
        num_samples: Number of word embeddings to generate
        dim: Dimension of embeddings (default 300)

    Returns:
        List of numpy arrays representing word embeddings

    """
    vectors = []
    for _ in range(num_samples):
        # Generate normalized embedding vectors
        vector = np.random.randn(dim).astype(np.float32)
        # Normalize to unit length (common in word embeddings)
        norm = np.linalg.norm(vector)
        if norm > 0:
            vector = vector / norm
        vectors.append(vector)
    return vectors


def load_faiss_vectors(num_samples: int = 1000, dim: int = 128) -> list[np.ndarray]:
    """Generate FAISS-compatible vectors for similarity search.

    Args:
        num_samples: Number of vectors to generate
        dim: Dimension of vectors (default 128)

    Returns:
        List of numpy arrays compatible with FAISS

    """
    vectors = []
    for _ in range(num_samples):
        # Generate random vectors (FAISS typically uses float32)
        vector = np.random.randn(dim).astype(np.float32)
        vectors.append(vector)
    return vectors


def generate_ml_embeddings(
    num_samples: int = 100,
    dim: int = 384,
    embedding_type: str = "sentence",
) -> list[np.ndarray]:
    """Generate ML embedding vectors (sentence-transformers style).

    Args:
        num_samples: Number of embeddings to generate
        dim: Dimension of embeddings (default 384 for sentence-transformers)
        embedding_type: Type of embedding ("sentence", "image", "text")

    Returns:
        List of numpy arrays representing embeddings

    """
    vectors = []
    for _ in range(num_samples):
        if embedding_type == "sentence":
            # Sentence embeddings are typically normalized
            vector = np.random.randn(dim).astype(np.float32)
            norm = np.linalg.norm(vector)
            if norm > 0:
                vector = vector / norm
        elif embedding_type == "image":
            # Image embeddings (CLIP-style)
            vector = np.random.randn(dim).astype(np.float32)
            norm = np.linalg.norm(vector)
            if norm > 0:
                vector = vector / norm
        else:  # text/BERT-style
            # BERT embeddings (not necessarily normalized)
            vector = np.random.randn(dim).astype(np.float32)
        vectors.append(vector)
    return vectors


def compare_implementations(
    test_name: str,
    implementations: dict[str, Callable],
    iterations: int = 1000,
) -> ComparisonResult:
    """Compare performance of multiple implementations.

    Args:
        test_name: Name of the test
        implementations: Dictionary mapping implementation names to callables
        iterations: Number of iterations to run

    Returns:
        ComparisonResult with comparison data

    """
    results = {}
    for name, operation in implementations.items():
        result = measure_operation(operation, iterations=iterations)
        result.name = test_name
        result.operation = name
        results[name] = result

    # Find winner (fastest average time)
    if results:
        winner = min(results.items(), key=lambda x: x[1].avg_time)[0]
        baseline_time = results[winner].avg_time
        speedup = {}
        for name, result in results.items():
            if name != winner and baseline_time > 0:
                speedup[name] = result.avg_time / baseline_time
    else:
        winner = None
        speedup = None

    return ComparisonResult(
        test_name=test_name,
        implementations=results,
        winner=winner,
        speedup=speedup,
    )


def generate_report(
    results: list[ComparisonResult],
    output_path: Path | None = None,
) -> str:
    """Generate a human-readable benchmark report.

    Args:
        results: List of comparison results
        output_path: Optional path to save JSON report

    Returns:
        Formatted report string

    """
    report_lines = []
    report_lines.append("=" * 80)
    report_lines.append("MAPPY-PYTHON COMPREHENSIVE BENCHMARK REPORT")
    report_lines.append("=" * 80)
    report_lines.append("")

    json_data = []

    for result in results:
        report_lines.append(f"\nTest: {result.test_name}")
        report_lines.append("-" * 80)

        if result.winner:
            report_lines.append(f"Winner: {result.winner}")
            if result.speedup:
                for name, speedup in result.speedup.items():
                    report_lines.append(f"  {name}: {speedup:.2f}x slower")
            report_lines.append("")

        for impl_name, impl_result in result.implementations.items():
            report_lines.append(f"  {impl_name}:")
            report_lines.append(f"    Operations: {impl_result.count}")
            report_lines.append(f"    Total Time: {impl_result.total_time:.4f}s")
            report_lines.append(f"    Avg Time: {impl_result.avg_time*1000:.4f}ms")
            median_ms = impl_result.median_time * 1000
            report_lines.append(f"    Median Time: {median_ms:.4f}ms")
            p95_ms = impl_result.p95_time * 1000
            report_lines.append(f"    P95 Time: {p95_ms:.4f}ms")
            p99_ms = impl_result.p99_time * 1000
            report_lines.append(f"    P99 Time: {p99_ms:.4f}ms")
            report_lines.append(f"    Throughput: {impl_result.throughput:.2f} ops/s")
            success_pct = impl_result.success_rate * 100
            report_lines.append(f"    Success Rate: {success_pct:.2f}%")
            if impl_result.memory_usage:
                mem_mb = impl_result.memory_usage / 1024 / 1024
                report_lines.append(f"    Memory: {mem_mb:.2f} MB")
            report_lines.append("")

        json_data.append(result.to_dict())

    report_lines.append("=" * 80)
    report = "\n".join(report_lines)

    if output_path:
        output_path.write_text(report)
        json_path = output_path.with_suffix(".json")
        json_path.write_text(json.dumps(json_data, indent=2))

    return report


def get_memory_usage() -> int:
    """Get current memory usage (if psutil is available).

    Returns:
        Memory usage in bytes, or 0 if unavailable

    """
    try:
        import os

        import psutil
        process = psutil.Process(os.getpid())
        return process.memory_info().rss
    except ImportError:
        return 0


def assert_approx_equal(
    actual: float,
    expected: float,
    tolerance: float = 0.01,
    msg: str | None = None,
):
    """Assert that two floats are approximately equal.

    Args:
        actual: Actual value
        expected: Expected value
        tolerance: Tolerance for comparison
        msg: Optional error message

    """
    diff = abs(actual - expected)
    if diff > tolerance:
        error_msg = (
            f"Values not approximately equal: {actual} != {expected} "
            f"(diff: {diff})"
        )
        if msg:
            error_msg = f"{msg}: {error_msg}"
        raise AssertionError(error_msg)


def assert_vector_equal(
    actual: np.ndarray,
    expected: np.ndarray,
    tolerance: float = 1e-5,
    msg: str | None = None,
):
    """Assert that two numpy arrays are approximately equal.

    Args:
        actual: Actual vector
        expected: Expected vector
        tolerance: Tolerance for comparison
        msg: Optional error message

    """
    if actual.shape != expected.shape:
        error_msg = f"Shape mismatch: {actual.shape} != {expected.shape}"
        if msg:
            error_msg = f"{msg}: {error_msg}"
        raise AssertionError(error_msg)

    diff = np.abs(actual - expected)
    max_diff = np.max(diff)
    if max_diff > tolerance:
        error_msg = (
            f"Vectors not approximately equal: max diff = {max_diff} > "
            f"{tolerance}"
        )
        if msg:
            error_msg = f"{msg}: {error_msg}"
        raise AssertionError(error_msg)
