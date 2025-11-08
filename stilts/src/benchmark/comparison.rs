//! Comparison with external compression libraries

use std::time::Instant;
use anyhow::Result;
use flate2::Compression;
use flate2::write::{ZlibEncoder, GzEncoder};
use flate2::read::{ZlibDecoder, GzDecoder};
use std::io::{Write, Read};
use crate::benchmark::metrics::{BenchmarkMetrics, CompressionStats};

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
            0.0, // No compression time
            0.0, // No decompression time
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

