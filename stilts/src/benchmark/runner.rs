//! Benchmark runner for compression algorithms

use std::time::Instant;
use anyhow::Result;
use crate::compression::{Compressor, HuffmanCompressor, ArithmeticCompressor, DictionaryCompressor};
use crate::benchmark::metrics::{BenchmarkMetrics, CompressionStats};

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

