# Stilts Reports Directory

This directory contains all generated benchmark reports, charts, and visualizations.

## Generated Files

### HTML Reports
- `storage_report.html` - Basic storage comparison report
- `comprehensive_storage_report.html` - Comprehensive report across all test cases

### Storage Comparison Charts
- `storage_size.png` - Storage size comparison chart
- `storage_compression_ratio.png` - Compression ratio comparison
- `storage_memory.png` - Memory usage comparison
- `storage_ratio_vs_memory.png` - Compression ratio vs memory scatter plot

### Algorithm Comparison Charts
- `algorithm_compression_ratio.png` - Compression ratio comparison across algorithms
- `algorithm_speed_comparison.png` - Speed comparison chart
- `algorithm_ratio_vs_speed.png` - Compression ratio vs speed scatter plot

## Generating Reports

All reports and charts are automatically generated in this directory when running:

```bash
# Basic integration example
cargo run --example mappy_integration --features mappy-integration

# Comprehensive benchmark
cargo run --example comprehensive_benchmark --features mappy-integration

# Python bindings
python -c "import stilts_python; c = stilts_python.StiltsCompressor(); c.plot_comparison(['tag1', 'tag2'], 'report.html')"
```

The `reports/` directory is automatically created if it doesn't exist.

