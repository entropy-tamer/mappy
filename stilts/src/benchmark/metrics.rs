#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark calculations
//! Performance metrics for compression benchmarks

use serde::{Deserialize, Serialize};

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub compression_speed_mbps: f64,
    pub decompression_speed_mbps: f64,
    pub dictionary_size: usize,
}

/// Benchmark metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    pub algorithm: String,
    pub stats: CompressionStats,
    pub memory_usage_bytes: usize,
}

impl CompressionStats {
    pub fn new(
        original_size: usize,
        compressed_size: usize,
        compression_time_ms: f64,
        decompression_time_ms: f64,
        dictionary_size: usize,
    ) -> Self {
        let compression_ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            0.0
        };

        let compression_speed_mbps = if compression_time_ms > 0.0 {
            (original_size as f64 / 1024.0 / 1024.0) / (compression_time_ms / 1000.0)
        } else {
            0.0
        };

        let decompression_speed_mbps = if decompression_time_ms > 0.0 {
            (compressed_size as f64 / 1024.0 / 1024.0) / (decompression_time_ms / 1000.0)
        } else {
            0.0
        };

        Self {
            original_size,
            compressed_size,
            compression_ratio,
            compression_speed_mbps,
            decompression_speed_mbps,
            dictionary_size,
        }
    }
}
