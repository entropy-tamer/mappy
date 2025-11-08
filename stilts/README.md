# Stilts - Tag Compression Library

A comprehensive Rust library for compressing tag data with multiple algorithms, format parsers, benchmarking, and visualization.

## Features

- **Multiple Compression Algorithms:**
  - Huffman coding (frequency-based)
  - Arithmetic coding
  - Custom dictionary with variable-length codes

- **Format Support:**
  - Space-separated tags
  - Comma-separated tags
  - JSON arrays
  - Compressed codes

- **Benchmarking:**
  - Performance metrics (compression ratio, speed, memory)
  - Comparison with external libraries (zlib, gzip, lz4, dict)

- **Visualization:**
  - Compression ratio charts
  - Speed comparison charts
  - HTML reports with embedded charts

- **Python Bindings:**
  - Full PyO3 integration
  - Easy-to-use API

## Usage

### Rust

```rust
use stilts::{HuffmanCompressor, SpaceSeparatedParser, TagParser};

// Parse tags
let parser = SpaceSeparatedParser::new();
let tags = parser.parse("tag1 tag2 tag3")?;

// Compress
let mut compressor = HuffmanCompressor::new();
compressor.build_from_corpus(&tags)?;
let compressed = compressor.compress(&tags)?;

// Decompress
let decompressed = compressor.decompress(&compressed)?;
```

### Python

```python
import stilts_python

compressor = stilts_python.StiltsCompressor()
tags = ["tag1", "tag2", "tag3"]

# Compress
compressed = compressor.compress(tags, algorithm="huffman")

# Decompress
decompressed = compressor.decompress(compressed, algorithm="huffman")

# Benchmark all algorithms
results = compressor.compare_algorithms(tags)

# Generate HTML report with charts
report_path = compressor.plot_comparison(tags, "report.html")
```

## Examples

Run the examples:

```bash
cargo run --example basic_compression
cargo run --example format_examples
```

## Benchmarks

Run Criterion benchmarks:

```bash
cargo bench
```

## Project Structure

```
stilts/
├── src/
│   ├── compression/     # Compression algorithms
│   ├── formats/         # Format parsers/serializers
│   ├── benchmark/       # Benchmarking framework
│   └── plotting/        # Visualization utilities
├── benches/             # Criterion benchmarks
├── examples/            # Example usage
└── python/              # Python bindings
```

## Integration with Mappy

Stilts includes comprehensive integration with mappy for storing compressed tags:

### Mappy Integration Capabilities

- **Mappy Storage Integration**: Store compressed tags in mappy using various compression algorithms
- **Storage Comparison**: Benchmark compressed tags in mappy vs other data structures (dict, zlib, gzip, lz4)
- **Memory Analysis**: Detailed memory usage metrics for all storage methods
- **Comprehensive Plots**: Visualizations showing:
  - Storage size comparison
  - Compression ratio comparison
  - Memory usage comparison
  - Compression ratio vs memory usage scatter plots

### Usage with Mappy

```rust
use stilts::mappy_integration::MappyTagStorage;
use stilts::formats::SpaceSeparatedParser;

// Parse tags
let parser = SpaceSeparatedParser::new();
let tags = parser.parse("tag1 tag2 tag3")?;

// Compress with Huffman for mappy storage
let mut storage = MappyTagStorage::with_huffman();
let compressed = storage.compress_tags(&tags)?;

// Store in mappy (compressed bytes can be stored using VectorOperator)
// Then decompress when retrieving
let decompressed = storage.decompress_tags(&compressed)?;
```

### Benchmarking Storage Methods

```rust
use stilts::benchmark::mappy_comparison::MappyComparisonRunner;

let tags = vec!["tag1".to_string(), "tag2".to_string()];
let comparisons = MappyComparisonRunner::compare_all_storage(&tags, 10)?;

// Generate comprehensive report with charts
use stilts::plotting::ReportGenerator;
let report_path = ReportGenerator::generate_storage_report(&comparisons, "storage_report.html")?;
```

### Running Storage Benchmarks

```bash
# With mappy integration enabled
cargo bench --features mappy-integration

# Run the integration example
cargo run --example mappy_integration --features mappy-integration
```

The storage comparison includes:

- **mappy_uncompressed**: Tags stored in mappy without compression
- **mappy_huffman**: Huffman-compressed tags in mappy
- **mappy_arithmetic**: Arithmetic-compressed tags in mappy
- **mappy_dictionary**: Dictionary-compressed tags in mappy
- **dict**: Python dict baseline
- **dict_zlib**: zlib-compressed tags in dict

All comparisons include memory analysis and performance metrics.

## License

MIT
