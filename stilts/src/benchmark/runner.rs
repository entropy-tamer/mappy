#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark calculations
//! Benchmark runner for compression algorithms

use crate::benchmark::metrics::{BenchmarkMetrics, CompressionStats};
use crate::compression::{
    ArithmeticCompressor, Compressor, DictionaryCompressor, HuffmanCompressor,
};
use anyhow::Result;
use std::time::Instant;

/// Benchmark runner
pub struct BenchmarkRunner;

impl BenchmarkRunner {
    /// Benchmark a compressor
    pub fn benchmark<C: Compressor>(
        compressor: &C,
        tags: &[String],
        iterations: usize,
    ) -> Result<BenchmarkMetrics> {
        // Serialize original tags to get size
        let mut original_bytes = Vec::new();
        for tag in tags {
            original_bytes.extend_from_slice(tag.as_bytes());
            original_bytes.push(b' ');
        }
        let original_size = original_bytes.len();

        // Warmup
        for _ in 0..3 {
            let _ = compressor.compress(tags);
        }

        // Benchmark compression
        let start = Instant::now();
        let mut compressed_data = Vec::new();
        for _ in 0..iterations {
            compressed_data = compressor.compress(tags)?;
        }
        let compression_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        // Benchmark decompression
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = compressor.decompress(&compressed_data)?;
        }
        let decompression_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        let compressed_size = compressed_data.len();
        let dictionary_size = 0; // TODO: Get actual dictionary size

        let stats = CompressionStats::new(
            original_size,
            compressed_size,
            compression_time,
            decompression_time,
            dictionary_size,
        );

        Ok(BenchmarkMetrics {
            algorithm: compressor.algorithm_name().to_string(),
            stats,
            memory_usage_bytes: compressed_size,
        })
    }

    /// Benchmark all algorithms
    pub fn benchmark_all(tags: &[String], iterations: usize) -> Result<Vec<BenchmarkMetrics>> {
        let mut results = Vec::new();

        // Huffman
        let mut huffman = HuffmanCompressor::new();
        huffman.build_from_corpus(tags)?;
        results.push(Self::benchmark(&huffman, tags, iterations)?);

        // Arithmetic
        let mut arithmetic = ArithmeticCompressor::new();
        arithmetic.build_from_corpus(tags)?;
        results.push(Self::benchmark(&arithmetic, tags, iterations)?);

        // Dictionary
        let mut dictionary = DictionaryCompressor::new();
        dictionary.build_from_corpus(tags)?;
        results.push(Self::benchmark(&dictionary, tags, iterations)?);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_runner_huffman() {
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        let mut huffman = HuffmanCompressor::new();
        huffman
            .build_from_corpus(&tags)
            .expect("Failed to build corpus");

        let result = BenchmarkRunner::benchmark(&huffman, &tags, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.algorithm, "huffman");
        assert!(metrics.stats.original_size > 0);
    }

    #[test]
    fn test_benchmark_runner_arithmetic() {
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        let mut arithmetic = ArithmeticCompressor::new();
        arithmetic
            .build_from_corpus(&tags)
            .expect("Failed to build corpus");

        let result = BenchmarkRunner::benchmark(&arithmetic, &tags, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.algorithm, "arithmetic");
        assert!(metrics.stats.original_size > 0);
    }

    #[test]
    fn test_benchmark_runner_dictionary() {
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        let mut dictionary = DictionaryCompressor::new();
        dictionary
            .build_from_corpus(&tags)
            .expect("Failed to build corpus");

        let result = BenchmarkRunner::benchmark(&dictionary, &tags, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.algorithm, "dictionary");
        assert!(metrics.stats.original_size > 0);
    }

    #[test]
    fn test_benchmark_runner_multiple_iterations() {
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let mut huffman = HuffmanCompressor::new();
        huffman
            .build_from_corpus(&tags)
            .expect("Failed to build corpus");

        let result = BenchmarkRunner::benchmark(&huffman, &tags, 5);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.stats.compression_speed_mbps >= 0.0);
        assert!(metrics.stats.decompression_speed_mbps >= 0.0);
    }

    #[test]
    fn test_benchmark_runner_all() {
        let tags = vec![
            "tag1".to_string(),
            "tag2".to_string(),
            "tag3".to_string(),
            "tag4".to_string(),
        ];

        let results = BenchmarkRunner::benchmark_all(&tags, 1);
        assert!(results.is_ok());
        let metrics = results.unwrap();
        assert_eq!(metrics.len(), 3);
        assert!(metrics.iter().any(|m| m.algorithm == "huffman"));
        assert!(metrics.iter().any(|m| m.algorithm == "arithmetic"));
        assert!(metrics.iter().any(|m| m.algorithm == "dictionary"));
    }

    #[test]
    fn test_benchmark_runner_single_tag() {
        let tags = vec!["single_tag".to_string()];
        let mut huffman = HuffmanCompressor::new();
        huffman
            .build_from_corpus(&tags)
            .expect("Failed to build corpus");

        let result = BenchmarkRunner::benchmark(&huffman, &tags, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.stats.original_size > 0);
    }

    #[test]
    fn test_benchmark_runner_compression_ratio() {
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        let mut huffman = HuffmanCompressor::new();
        huffman
            .build_from_corpus(&tags)
            .expect("Failed to build corpus");

        let result = BenchmarkRunner::benchmark(&huffman, &tags, 1);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.stats.compression_ratio >= 0.0);
        assert!(metrics.stats.compression_ratio <= 1.0);
    }
}
