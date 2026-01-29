# Validation and Benchmark Integration

## Overview

This document describes the expanded validation system and benchmark infrastructure added to the evaluation suite.

## Configuration Validation

### Validation Module (`validation.py`)

Comprehensive validation for:
- **Task configurations**: Validates task-specific parameters (batch size, epochs, dataset names, etc.)
- **Maplet configuration**: Validates capacity, false positive rate, and dataset size compatibility
- **DataLoader configuration**: Validates batch size, num_workers, and dataset compatibility
- **Data format**: Validates data type compatibility for different tasks
- **Benchmark configuration**: Validates complete benchmark setup

### Usage

```python
from evaluation.validation import ConfigValidator

# Validate task config
result = ConfigValidator.validate_task_config(
    task_name="image_classification",
    config={"batch_size": 32, "num_epochs": 1, "dataset_name": "cifar10"},
    dataset_size=1000
)

if not result.is_valid:
    print(result.errors)
```

## Dataset Comparison Benchmarks

### Benchmark Module (`benchmarks/dataset_comparison.py`)

Compares mappy DataLoader against:
- **Standard PyTorch DataLoader**: Baseline comparison
- **HuggingFace datasets**: Alternative data loading approach

### Metrics Collected

- Throughput (samples/second)
- Average batch latency (ms)
- First batch time (ms)
- Peak memory usage (MB)
- Total time (seconds)

### Usage

```python
from benchmarks.dataset_comparison import run_dataset_comparison_benchmark

results = run_dataset_comparison_benchmark(
    dataset_size=1000,
    batch_size=32,
    num_workers=0,
    num_iterations=3
)
```

## Empirical Analysis

### Analysis Module (`benchmarks/analysis.py`)

Provides:
- **Statistical summary**: Mean, std, min, max, median for all metrics
- **Performance comparisons**: Ratios and improvements vs baseline
- **Statistical tests**: T-test approximations for significance
- **Insights**: Human-readable recommendations
- **Visualizations**: Plotly charts for throughput, memory, latency

### Usage

```python
from benchmarks.analysis import analyze_benchmark_results

analysis = analyze_benchmark_results(results, output_dir="reports")

# Print insights
for insight in analysis["insights"]:
    print(insight)
```

## Running Benchmarks

### Command Line

```bash
# Run dataset comparison benchmark
python examples/run_dataset_comparison.py \
    --dataset-size 1000 \
    --batch-size 32 \
    --num-workers 0 \
    --iterations 3 \
    --output-dir reports
```

### Programmatic

```python
from benchmarks.dataset_comparison import DatasetComparisonBenchmark
from benchmarks.analysis import analyze_benchmark_results

# Create benchmark
benchmark = DatasetComparisonBenchmark(
    dataset_size=1000,
    batch_size=32,
    num_workers=0
)

# Run comparison
results = benchmark.run_comparison(num_iterations=3)

# Analyze
analysis = analyze_benchmark_results(results, output_dir="reports")
```

## Output Files

Benchmarks generate:
- `comparison_results.json`: Raw benchmark results
- `benchmark_analysis.json`: Statistical analysis and insights
- `throughput_comparison.html`: Throughput visualization
- `memory_comparison.html`: Memory usage visualization

## Integration with Tasks

Validation can be integrated into task initialization:

```python
from evaluation.validation import ConfigValidator

class MyTask:
    def __init__(self, **config):
        # Validate configuration
        validation = ConfigValidator.validate_task_config(
            task_name="my_task",
            config=config
        )
        
        if not validation.is_valid:
            raise ValueError(f"Invalid configuration: {validation}")
        
        # Use validated config
        self.batch_size = config["batch_size"]
        # ...
```

## Ruff Code Quality

All code is checked with ruff for:
- Import organization
- Type annotations (modern Python 3.10+ syntax)
- Line length (88 chars)
- Docstring formatting
- Error handling best practices

Run checks:
```bash
ruff check .
ruff check . --fix  # Auto-fix issues
```

