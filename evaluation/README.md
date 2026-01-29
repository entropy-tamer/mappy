# Mappy PyTorch DataLoader Evaluation Suite

Comprehensive evaluation suite for testing `mappy` as a PyTorch DataLoader across multiple ML training tasks with full metrics tracking.

## Overview

This evaluation suite provides:

- **PyTorch DataLoader Integration**: Seamless integration with PyTorch's DataLoader API
- **Multiple ML Tasks**: Image classification, NLP, recommendation systems, image diffusion, image captioning
- **Comprehensive Metrics**: Throughput, latency, memory, GPU utilization, and accuracy tracking
- **Comparison Benchmarks**: Compare mappy vs standard DataLoader performance
- **Rich Reporting**: HTML and JSON reports with interactive visualizations

## Installation

```bash
# Install dependencies
cd services/mappy/evaluation
pip install -r requirements.txt

# Ensure mappy-python is installed
cd ../mappy-python
pip install -e .
```

## Quick Start

### Run All Benchmarks

```bash
python examples/run_all_benchmarks.py
```

This will:

1. Run all 6 ML tasks with mappy DataLoader
2. Collect comprehensive metrics
3. Generate HTML and JSON reports in `reports/` directory

### Run Single Task

```bash
python examples/run_single_task.py --task image_classification
```

Available tasks:

- `image_classification` - CIFAR-10 and ImageNet-style classification
- `nlp_classification` - Text classification
- `nlp_language_modeling` - Language modeling
- `recommendation` - Collaborative filtering
- `image_diffusion` - Stable Diffusion-style training
- `image_captioning` - COCO-style image captioning

### Compare Mappy vs Baseline

```bash
python examples/compare_dataloaders.py
```

This runs both mappy and standard DataLoader and generates comparison reports.

## Usage

### Programmatic Usage

```python
from evaluation.benchmark_runner import BenchmarkRunner
from evaluation.reporting import generate_report

# Create runner
runner = BenchmarkRunner(use_mappy=True, cache_data=True)

# Run all tasks
results = runner.run_all_tasks()

# Generate report
report_paths = generate_report(results, output_dir="reports")
print(f"HTML report: {report_paths['html']}")
```

### Custom Task Configuration

```python
from evaluation.benchmark_runner import BenchmarkRunner

runner = BenchmarkRunner(use_mappy=True, cache_data=True)

# Custom configuration
task_configs = {
    "image_classification": {
        "dataset_name": "cifar10",
        "batch_size": 64,
        "num_epochs": 2,
    },
    "nlp_classification": {
        "batch_size": 32,
        "num_epochs": 1,
    },
}

results = runner.run_all_tasks(task_configs=task_configs)
```

## Architecture

### Core Components

- **`dataloader.py`**: PyTorch DataLoader wrapper with caching support
- **`metrics.py`**: Comprehensive metrics collection
- **`benchmark_runner.py`**: Orchestration and task management
- **`reporting.py`**: HTML/JSON report generation

### Task Implementations

- **`tasks/image_classification.py`**: Image classification tasks
- **`tasks/nlp_tasks.py`**: NLP tasks (classification, language modeling)
- **`tasks/recommendation.py`**: Recommendation systems
- **`tasks/image_diffusion.py`**: Image diffusion training
- **`tasks/image_captioning.py`**: Image captioning

### Utilities

- **`utils/data_utils.py`**: Data preprocessing helpers
- **`utils/gpu_utils.py`**: GPU monitoring utilities

## Metrics Collected

### Performance Metrics

- **Throughput**: Samples/second, batches/second
- **Latency**: First batch time, average batch time, min/max batch times
- **Memory**: Peak memory, average memory, memory per sample
- **GPU**: Utilization percentage, memory usage, idle time

### Accuracy Metrics

- **Classification**: Accuracy, precision, recall, F1 score
- **Language Modeling**: Perplexity
- **Captioning**: BLEU, METEOR, CIDEr scores
- **Diffusion**: FID score (proxy)
- **Recommendation**: RMSE, precision@k, recall@k

### DataLoader Metrics

- Worker efficiency
- Queue utilization
- Prefetch effectiveness
- Cache hit rates

## Optimization Strategy

Following the stilts optimization pattern:

1. **Cache-First Strategy**: Pre-decompress and cache all data before training
2. **Move Mappy Out of Hot Path**: Only use mappy for storage, not during training
3. **Efficient Batch Processing**: Support variable batch sizes and dynamic batching

This ensures minimal overhead while maintaining the benefits of mappy's approximate storage.

## Reports

Reports are generated in both HTML and JSON formats:

- **HTML Reports**: Interactive visualizations with Plotly charts
- **JSON Reports**: Machine-readable format for programmatic analysis

Reports include:

- Performance metrics charts
- Accuracy comparisons
- Memory usage analysis
- Mappy vs baseline comparisons (if applicable)

## Examples

See `examples/` directory for:

- `run_all_benchmarks.py` - Run all tasks
- `run_single_task.py` - Run single task
- `compare_dataloaders.py` - Compare mappy vs baseline

## Success Criteria

The evaluation suite validates:

- ✅ All ML tasks run successfully with mappy DataLoader
- ✅ Performance metrics collected for all dimensions
- ✅ Accuracy impact < 1% compared to baseline
- ✅ Throughput within 2x of standard DataLoader
- ✅ Comprehensive HTML reports generated

## Troubleshooting

### Import Errors

If you see import errors for `mappy_python`:

```bash
cd services/mappy/mappy-python
pip install -e .
```

### GPU Monitoring

GPU metrics require `pynvml`. Install with:

```bash
pip install pynvml
```

### Memory Issues

For large datasets, consider:

- Reducing batch size
- Disabling caching (`cache_data=False`)
- Using fewer workers

## License

MIT License - see LICENSE file in parent directory.
