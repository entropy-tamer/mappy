//! Integration with mappy for storing compressed tags

use crate::compression::{
    ArithmeticCompressor, Compressor, DictionaryCompressor, HuffmanCompressor,
};
use anyhow::Result;

/// Storage metrics for mappy integration
#[derive(Debug, Clone)]
pub struct MappyStorageMetrics {
    pub key: String,
    pub original_size: usize,
    pub compressed_size: usize,
    pub mappy_storage_size: usize,
    pub compression_ratio: f64,
    pub total_storage_ratio: f64, // compressed_size / mappy_storage_size
}

/// Helper for storing compressed tags in mappy
pub struct MappyTagStorage {
    compressor: Box<dyn Compressor>,
}

impl MappyTagStorage {
    /// Create with Huffman compressor
    pub fn with_huffman() -> Self {
        Self {
            compressor: Box::new(HuffmanCompressor::new()),
        }
    }

    /// Create with Arithmetic compressor
    pub fn with_arithmetic() -> Self {
        Self {
            compressor: Box::new(ArithmeticCompressor::new()),
        }
    }

    /// Create with Dictionary compressor
    pub fn with_dictionary() -> Self {
        Self {
            compressor: Box::new(DictionaryCompressor::new()),
        }
    }

    /// Build corpus from tags (call once before compressing multiple times)
    pub fn build_corpus(&mut self, tags: &[String]) -> Result<()> {
        match self.compressor.algorithm_name() {
            "huffman" => {
                let mut huffman = HuffmanCompressor::new();
                huffman.build_from_corpus(tags)?;
                self.compressor = Box::new(huffman);
            }
            "arithmetic" => {
                let mut arithmetic = ArithmeticCompressor::new();
                arithmetic.build_from_corpus(tags)?;
                self.compressor = Box::new(arithmetic);
            }
            "dictionary" => {
                let mut dict = DictionaryCompressor::new();
                dict.build_from_corpus(tags)?;
                self.compressor = Box::new(dict);
            }
            _ => {}
        }
        Ok(())
    }

    /// Compress tags for mappy storage (corpus must be built first)
    pub fn compress_tags(&mut self, tags: &[String]) -> Result<Vec<u8>> {
        self.compressor.compress(tags)
    }

    /// Compress tags and build corpus if needed (convenience method)
    pub fn compress_tags_with_corpus(&mut self, tags: &[String]) -> Result<Vec<u8>> {
        self.build_corpus(tags)?;
        self.compress_tags(tags)
    }

    /// Decompress tags from mappy storage
    pub fn decompress_tags(&self, data: &[u8]) -> Result<Vec<String>> {
        self.compressor.decompress(data)
    }

    /// Get algorithm name
    pub fn algorithm_name(&self) -> &'static str {
        self.compressor.algorithm_name()
    }
}
