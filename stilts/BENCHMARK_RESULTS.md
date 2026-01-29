# Stilts Benchmark Results

## Empirical Data Summary

### Test Case 1: Long Tag String (53 tags, 492 bytes original)

**Compression Results:**

- **Huffman**: 43 bytes (9.77% ratio) - **BEST**
- Dictionary: 925 bytes (210.23% ratio)
- Arithmetic: 930 bytes (211.36% ratio)

**Storage Comparison:**

- **mappy_huffman**: 43 bytes stored (0.0977 ratio, 43 bytes memory) - **BEST**
- mappy_uncompressed: 542 bytes stored (1.2318 ratio, 542 bytes memory)
- dict_zlib: 295 bytes stored (0.6705 ratio, 295 bytes memory)
- dict: 864 bytes stored (1.0000 ratio, 864 bytes memory)
- mappy_arithmetic: 930 bytes stored (2.1136 ratio, 930 bytes memory)
- mappy_dictionary: 925 bytes stored (2.1023 ratio, 925 bytes memory)

### Test Case 2: Compressed Codes Format (55 tags, 151 bytes original)

**Compression Results:**

- **Huffman**: 45 bytes (46.39% ratio) - **BEST**
- Dictionary: 600 bytes (618.56% ratio)
- Arithmetic: 605 bytes (623.71% ratio)

### Test Case 3: Large Generated Dataset (500 tags, 3575 bytes total)

**Compression Results:**

- **Huffman**: 279 bytes (7.80% ratio) - **BEST**
- Arithmetic: 336 bytes (9.40% ratio)
- Dictionary: 811 bytes (22.69% ratio)

### Test Case 4: Comma-Separated Format (10 tags, 62 bytes original)

**Compression Results:**

- **Huffman**: 9 bytes (16.98% ratio) - **BEST**
- Dictionary: 151 bytes (284.91% ratio)
- Arithmetic: 156 bytes (294.34% ratio)

## Key Findings

1. **Huffman coding consistently provides the best compression** across all test cases
2. **Mappy + Huffman compression** achieves the smallest storage size (43 bytes vs 542 bytes uncompressed)
3. **Compression ratios vary significantly** based on tag format:
   - Long tag strings: ~10% (Huffman)
   - Compressed codes: ~46% (Huffman) - already compressed format
   - Large datasets: ~8% (Huffman)
   - Small comma-separated: ~17% (Huffman)

4. **Memory efficiency**: Huffman-compressed tags in mappy use only 8.74% of original size for the long tag string example

## Generated Reports

- `storage_report.html` - Storage comparison report with charts
- `comprehensive_storage_report.html` - Comprehensive report across all test cases
- `algorithm_compression_ratio.png` - Compression ratio comparison chart
- `algorithm_speed_comparison.png` - Speed comparison chart
- `algorithm_ratio_vs_speed.png` - Ratio vs speed scatter plot
- `storage_size.png` - Storage size comparison
- `storage_compression_ratio.png` - Storage compression ratio comparison
- `storage_memory.png` - Memory usage comparison
- `storage_ratio_vs_memory.png` - Compression ratio vs memory scatter plot

## Recommendations

1. **Use Huffman compression** for storing tags in mappy - provides best compression ratio
2. **For already-compressed formats** (like the compressed codes), compression may not be as effective
3. **Large datasets benefit most** from compression (7.80% ratio for 500 tags)
4. **Mappy + Huffman** provides the best overall storage efficiency
