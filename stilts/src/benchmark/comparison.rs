#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark calculations
//! Comparison with external compression libraries

use crate::benchmark::metrics::{BenchmarkMetrics, CompressionStats};
use anyhow::Result;
use flate2::Compression;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use std::io::{Read, Write};
use std::time::Instant;

/// Comparison runner for external libraries
pub struct ComparisonRunner;

impl ComparisonRunner {
    /// Benchmark zlib compression
    pub fn benchmark_zlib(data: &[u8], iterations: usize) -> Result<BenchmarkMetrics> {
        let original_size = data.len();

        // Warmup
        for _ in 0..3 {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data)?;
            let _ = encoder.finish()?;
        }

        // Benchmark compression
        let start = Instant::now();
        let mut compressed_data = Vec::new();
        for _ in 0..iterations {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data)?;
            compressed_data = encoder.finish()?;
        }
        let compression_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        // Benchmark decompression
        let start = Instant::now();
        for _ in 0..iterations {
            let mut decoder = ZlibDecoder::new(&compressed_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
        }
        let decompression_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        let compressed_size = compressed_data.len();
        let stats = CompressionStats::new(
            original_size,
            compressed_size,
            compression_time,
            decompression_time,
            0,
        );

        Ok(BenchmarkMetrics {
            algorithm: "zlib".to_string(),
            stats,
            memory_usage_bytes: compressed_size,
        })
    }

    /// Benchmark gzip compression
    pub fn benchmark_gzip(data: &[u8], iterations: usize) -> Result<BenchmarkMetrics> {
        let original_size = data.len();

        // Warmup
        for _ in 0..3 {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data)?;
            let _ = encoder.finish()?;
        }

        // Benchmark compression
        let start = Instant::now();
        let mut compressed_data = Vec::new();
        for _ in 0..iterations {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data)?;
            compressed_data = encoder.finish()?;
        }
        let compression_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        // Benchmark decompression
        let start = Instant::now();
        for _ in 0..iterations {
            let mut decoder = GzDecoder::new(&compressed_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
        }
        let decompression_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        let compressed_size = compressed_data.len();
        let stats = CompressionStats::new(
            original_size,
            compressed_size,
            compression_time,
            decompression_time,
            0,
        );

        Ok(BenchmarkMetrics {
            algorithm: "gzip".to_string(),
            stats,
            memory_usage_bytes: compressed_size,
        })
    }

    /// Benchmark lz4 compression
    pub fn benchmark_lz4(data: &[u8], iterations: usize) -> Result<BenchmarkMetrics> {
        let original_size = data.len();

        // Warmup
        for _ in 0..3 {
            let _ = lz4::block::compress(data, None, false)?;
        }

        // Benchmark compression
        let start = Instant::now();
        let mut compressed_data = Vec::new();
        for _ in 0..iterations {
            compressed_data = lz4::block::compress(data, None, false)?;
        }
        let compression_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        // Benchmark decompression
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = lz4::block::decompress(&compressed_data, Some(original_size as i32))?;
        }
        let decompression_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        let compressed_size = compressed_data.len();
        let stats = CompressionStats::new(
            original_size,
            compressed_size,
            compression_time,
            decompression_time,
            0,
        );

        Ok(BenchmarkMetrics {
            algorithm: "lz4".to_string(),
            stats,
            memory_usage_bytes: compressed_size,
        })
    }

    /// Benchmark Python dict (baseline - just measure serialization)
    pub fn benchmark_dict(data: &[u8], _iterations: usize) -> Result<BenchmarkMetrics> {
        let original_size = data.len();

        // For dict, we just measure the size (no compression)
        let stats = CompressionStats::new(
            original_size,
            original_size, // No compression
            0.0,           // No compression time
            0.0,           // No decompression time
            0,
        );

        Ok(BenchmarkMetrics {
            algorithm: "dict".to_string(),
            stats,
            memory_usage_bytes: original_size,
        })
    }

    /// Compare all external libraries
    pub fn compare_all(data: &[u8], iterations: usize) -> Result<Vec<BenchmarkMetrics>> {
        let mut results = Vec::new();

        results.push(Self::benchmark_dict(data, iterations)?);
        results.push(Self::benchmark_zlib(data, iterations)?);
        results.push(Self::benchmark_gzip(data, iterations)?);
        results.push(Self::benchmark_lz4(data, iterations)?);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_dict() {
        let data = b"test data for compression";
        let result = ComparisonRunner::benchmark_dict(data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.algorithm, "dict");
        assert_eq!(metrics.stats.original_size, data.len());
        assert_eq!(metrics.stats.compressed_size, data.len());
        assert_eq!(metrics.stats.compression_ratio, 1.0);
    }

    #[test]
    fn test_benchmark_zlib() {
        let data = b"test data for zlib compression testing";
        let result = ComparisonRunner::benchmark_zlib(data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.algorithm, "zlib");
        assert!(metrics.stats.original_size > 0);
        assert!(metrics.stats.compressed_size > 0);
        assert!(metrics.stats.compression_ratio > 0.0);
    }

    #[test]
    fn test_benchmark_gzip() {
        let data = b"test data for gzip compression testing";
        let result = ComparisonRunner::benchmark_gzip(data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.algorithm, "gzip");
        assert!(metrics.stats.original_size > 0);
        assert!(metrics.stats.compressed_size > 0);
        assert!(metrics.stats.compression_ratio > 0.0);
    }

    #[test]
    fn test_benchmark_lz4() {
        let data = b"test data for lz4 compression testing";
        let result = ComparisonRunner::benchmark_lz4(data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.algorithm, "lz4");
        assert!(metrics.stats.original_size > 0);
        assert!(metrics.stats.compressed_size > 0);
    }

    #[test]
    fn test_benchmark_compare_all() {
        let data = b"test data for comparison testing";
        let result = ComparisonRunner::compare_all(data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.len(), 4);
        let algorithms: Vec<&str> = metrics.iter().map(|m| m.algorithm.as_str()).collect();
        assert!(algorithms.contains(&"dict"));
        assert!(algorithms.contains(&"zlib"));
        assert!(algorithms.contains(&"gzip"));
        assert!(algorithms.contains(&"lz4"));
    }

    #[test]
    fn test_benchmark_empty_data() {
        let data = b"";
        let result = ComparisonRunner::benchmark_dict(data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.stats.original_size, 0);
        assert_eq!(metrics.stats.compressed_size, 0);
    }

    #[test]
    fn test_benchmark_single_byte() {
        let data = b"a";
        let result = ComparisonRunner::benchmark_dict(data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.stats.original_size, 1);
    }

    #[test]
    fn test_benchmark_large_data() {
        let data = vec![b'a'; 10000];
        let result = ComparisonRunner::benchmark_dict(&data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.stats.original_size, 10000);
    }

    #[test]
    fn test_benchmark_multiple_iterations() {
        let data = b"test data for iterations";
        let result = ComparisonRunner::benchmark_zlib(data, 5);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.stats.compression_speed_mbps >= 0.0);
        assert!(metrics.stats.decompression_speed_mbps >= 0.0);
    }

    #[test]
    fn test_benchmark_compression_ratios() {
        let data = b"repeated repeated repeated data data data";
        let result = ComparisonRunner::compare_all(data, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();

        let dict_metrics = metrics
            .iter()
            .find(|m| m.algorithm == "dict")
            .expect("dict not found");
        assert_eq!(dict_metrics.stats.compression_ratio, 1.0);

        let zlib_metrics = metrics
            .iter()
            .find(|m| m.algorithm == "zlib")
            .expect("zlib not found");
        assert!(zlib_metrics.stats.compression_ratio < 1.0);
    }
}
