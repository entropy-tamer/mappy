"""Benchmark modules for comparing mappy with other data loading approaches."""

from .dataset_comparison import (
    ComparisonResult,
    DatasetComparisonBenchmark,
    run_dataset_comparison_benchmark,
)

__all__ = [
    "ComparisonResult",
    "DatasetComparisonBenchmark",
    "run_dataset_comparison_benchmark",
]

