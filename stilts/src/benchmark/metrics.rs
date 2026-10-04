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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_stats_new() {
        let stats = CompressionStats::new(1000, 500, 100.0, 50.0, 128);
        assert_eq!(stats.original_size, 1000);
        assert_eq!(stats.compressed_size, 500);
        assert_eq!(stats.compression_ratio, 0.5);
        assert_eq!(stats.dictionary_size, 128);
    }

    #[test]
    fn test_compression_stats_zero_original_size() {
        let stats = CompressionStats::new(0, 0, 0.0, 0.0, 0);
        assert_eq!(stats.original_size, 0);
        assert_eq!(stats.compressed_size, 0);
        assert_eq!(stats.compression_ratio, 0.0);
        assert_eq!(stats.compression_speed_mbps, 0.0);
        assert_eq!(stats.decompression_speed_mbps, 0.0);
    }

    #[test]
    fn test_compression_stats_zero_time() {
        let stats = CompressionStats::new(1000, 500, 0.0, 0.0, 0);
        assert_eq!(stats.compression_speed_mbps, 0.0);
        assert_eq!(stats.decompression_speed_mbps, 0.0);
    }

    #[test]
    fn test_compression_stats_calculations() {
        let stats = CompressionStats::new(1024 * 1024, 512 * 1024, 1000.0, 500.0, 256);
        assert!((stats.compression_ratio - 0.5).abs() < 0.01);
        assert!((stats.compression_speed_mbps - 1.024).abs() < 0.5);
        assert!((stats.decompression_speed_mbps - 0.512).abs() < 0.5);
    }

    #[test]
    fn test_benchmark_metrics_creation() {
        let stats = CompressionStats::new(1000, 500, 100.0, 50.0, 128);
        let metrics = BenchmarkMetrics {
            algorithm: "test".to_string(),
            stats: stats.clone(),
            memory_usage_bytes: 500,
        };
        assert_eq!(metrics.algorithm, "test");
        assert_eq!(metrics.memory_usage_bytes, 500);
        assert_eq!(metrics.stats.original_size, 1000);
    }

    #[test]
    fn test_compression_stats_serialization() {
        let stats = CompressionStats::new(1000, 500, 100.0, 50.0, 128);
        let serialized = serde_json::to_string(&stats).expect("Failed to serialize");
        let deserialized: CompressionStats =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.original_size, 1000);
        assert_eq!(deserialized.compressed_size, 500);
    }

    #[test]
    fn test_benchmark_metrics_serialization() {
        let stats = CompressionStats::new(1000, 500, 100.0, 50.0, 128);
        let metrics = BenchmarkMetrics {
            algorithm: "test".to_string(),
            stats,
            memory_usage_bytes: 500,
        };
        let serialized = serde_json::to_string(&metrics).expect("Failed to serialize");
        let deserialized: BenchmarkMetrics =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.algorithm, "test");
        assert_eq!(deserialized.memory_usage_bytes, 500);
    }
}
