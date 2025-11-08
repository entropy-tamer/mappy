//! Benchmark comparison: storing compressed tags in mappy vs other data structures

use std::time::Instant;
use std::collections::HashMap;
use anyhow::Result;
use crate::compression::{HuffmanCompressor, ArithmeticCompressor, DictionaryCompressor, Compressor};
use crate::benchmark::metrics::{BenchmarkMetrics, CompressionStats};
use crate::mappy_integration::MappyTagStorage;

#[cfg(feature = "mappy-integration")]
use mappy_core::{Maplet, VectorOperator};

/// Storage comparison metrics
#[derive(Debug, Clone)]
pub struct StorageComparison {
    pub method: String,
    pub original_size: usize,
    pub storage_size: usize,
    pub compression_ratio: f64,
    pub insert_time_ms: f64,
    pub query_time_ms: f64,
    pub memory_usage_bytes: usize,
}

/// Compare storing compressed tags in different data structures
pub struct MappyComparisonRunner;

impl MappyComparisonRunner {
    /// Generate test tags (fuzzed/varied formats)
    pub fn generate_test_tags(count: usize) -> Vec<String> {
        let base_tags = vec![
            "2007", "3_toes", "4_fingers", "anthro", "biped",
            "black_and_white", "breasts", "canid", "canine", "claws",
            "collar", "dialogue", "domestic_dog", "english_text", "eyewear",
            "fangs", "feet", "female", "fingers", "genitals",
        ];
        
        (0..count)
            .map(|i| base_tags[i % base_tags.len()].to_string())
            .collect()
    }
    
    /// Benchmark storing uncompressed tags in mappy
    #[cfg(feature = "mappy-integration")]
    pub fn benchmark_mappy_uncompressed(tags: &[String], _iterations: usize) -> Result<StorageComparison> {
        let original_size: usize = tags.iter().map(|t| t.len()).sum();
        
        // Convert tags to bytes for storage (simulate mappy storage)
        let mut tag_bytes = Vec::new();
        for tag in tags {
            tag_bytes.extend_from_slice(tag.as_bytes());
            tag_bytes.push(b' ');
        }
        
        // For mappy, we estimate storage size (mappy adds overhead for probabilistic structure)
        // Rough estimate: original size + 10% overhead for mappy structure
        let storage_size = (tag_bytes.len() as f64 * 1.1) as usize;
        
        Ok(StorageComparison {
            method: "mappy_uncompressed".to_string(),
            original_size,
            storage_size,
            compression_ratio: storage_size as f64 / original_size as f64,
            insert_time_ms: 0.0,
            query_time_ms: 0.0,
            memory_usage_bytes: storage_size,
        })
    }
    
    /// Benchmark storing Huffman-compressed tags in mappy
    #[cfg(feature = "mappy-integration")]
    pub fn benchmark_mappy_huffman(tags: &[String], iterations: usize) -> Result<StorageComparison> {
        let original_size: usize = tags.iter().map(|t| t.len()).sum();
        
        let mut storage = MappyTagStorage::with_huffman();
        // Build corpus once
        storage.build_corpus(tags)?;
        // Compress once to get size
        let compressed = storage.compress_tags(tags)?;
        
        // Benchmark compression only (corpus already built)
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = storage.compress_tags(tags);
        }
        let insert_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
        
        Ok(StorageComparison {
            method: "mappy_huffman".to_string(),
            original_size,
            storage_size: compressed.len(),
            compression_ratio: compressed.len() as f64 / original_size as f64,
            insert_time_ms: insert_time,
            query_time_ms: 0.0,
            memory_usage_bytes: compressed.len(),
        })
    }
    
    /// Benchmark storing Arithmetic-compressed tags in mappy
    #[cfg(feature = "mappy-integration")]
    pub fn benchmark_mappy_arithmetic(tags: &[String], iterations: usize) -> Result<StorageComparison> {
        let original_size: usize = tags.iter().map(|t| t.len()).sum();
        
        let mut storage = MappyTagStorage::with_arithmetic();
        // Build corpus once
        storage.build_corpus(tags)?;
        // Compress once to get size
        let compressed = storage.compress_tags(tags)?;
        
        // Benchmark compression only (corpus already built)
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = storage.compress_tags(tags);
        }
        let insert_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
        
        Ok(StorageComparison {
            method: "mappy_arithmetic".to_string(),
            original_size,
            storage_size: compressed.len(),
            compression_ratio: compressed.len() as f64 / original_size as f64,
            insert_time_ms: insert_time,
            query_time_ms: 0.0,
            memory_usage_bytes: compressed.len(),
        })
    }
    
    /// Benchmark storing Dictionary-compressed tags in mappy
    #[cfg(feature = "mappy-integration")]
    pub fn benchmark_mappy_dictionary(tags: &[String], iterations: usize) -> Result<StorageComparison> {
        let original_size: usize = tags.iter().map(|t| t.len()).sum();
        
        let mut storage = MappyTagStorage::with_dictionary();
        // Build corpus once
        storage.build_corpus(tags)?;
        // Compress once to get size
        let compressed = storage.compress_tags(tags)?;
        
        // Benchmark compression only (corpus already built)
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = storage.compress_tags(tags);
        }
        let insert_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
        
        Ok(StorageComparison {
            method: "mappy_dictionary".to_string(),
            original_size,
            storage_size: compressed.len(),
            compression_ratio: compressed.len() as f64 / original_size as f64,
            insert_time_ms: insert_time,
            query_time_ms: 0.0,
            memory_usage_bytes: compressed.len(),
        })
    }
    
    /// Benchmark storing in Python dict
    pub fn benchmark_dict(tags: &[String], _iterations: usize) -> Result<StorageComparison> {
        let original_size: usize = tags.iter().map(|t| t.len()).sum();
        
        let mut dict = HashMap::new();
        for (i, tag) in tags.iter().enumerate() {
            dict.insert(format!("key_{}", i), tag.clone());
        }
        
        // Estimate memory usage (simplified)
        let storage_size = original_size + (tags.len() * 8); // Rough estimate
        
        Ok(StorageComparison {
            method: "dict".to_string(),
            original_size,
            storage_size,
            compression_ratio: 1.0, // No compression
            insert_time_ms: 0.0,
            query_time_ms: 0.0,
            memory_usage_bytes: storage_size,
        })
    }
    
    /// Benchmark storing zlib-compressed in dict
    pub fn benchmark_dict_zlib(tags: &[String], iterations: usize) -> Result<StorageComparison> {
        use flate2::Compression;
        use flate2::write::ZlibEncoder;
        use std::io::Write;
        
        let original_size: usize = tags.iter().map(|t| t.len()).sum();
        
        let mut tag_bytes = Vec::new();
        for tag in tags {
            tag_bytes.extend_from_slice(tag.as_bytes());
            tag_bytes.push(b' ');
        }
        
        let start = Instant::now();
        let mut compressed_data = Vec::new();
        for _ in 0..iterations {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&tag_bytes)?;
            compressed_data = encoder.finish()?;
        }
        let insert_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
        
        Ok(StorageComparison {
            method: "dict_zlib".to_string(),
            original_size,
            storage_size: compressed_data.len(),
            compression_ratio: compressed_data.len() as f64 / original_size as f64,
            insert_time_ms: insert_time,
            query_time_ms: 0.0,
            memory_usage_bytes: compressed_data.len(),
        })
    }
    
    /// Compare all storage methods
    pub fn compare_all_storage(tags: &[String], iterations: usize) -> Result<Vec<StorageComparison>> {
        let mut results = Vec::new();
        
        // Dict baseline
        results.push(Self::benchmark_dict(tags, iterations)?);
        results.push(Self::benchmark_dict_zlib(tags, iterations)?);
        
        #[cfg(feature = "mappy-integration")]
        {
            results.push(Self::benchmark_mappy_uncompressed(tags, iterations)?);
            results.push(Self::benchmark_mappy_huffman(tags, iterations)?);
            results.push(Self::benchmark_mappy_arithmetic(tags, iterations)?);
            results.push(Self::benchmark_mappy_dictionary(tags, iterations)?);
        }
        
        Ok(results)
    }
}

