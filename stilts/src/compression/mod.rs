//! Compression algorithms for tag compression

pub mod huffman;
pub mod arithmetic;
pub mod dictionary;

use anyhow::Result;

/// Trait for compression algorithms
pub trait Compressor: Send + Sync {
    /// Compress a list of tags into bytes
    fn compress(&self, tags: &[String]) -> Result<Vec<u8>>;
    
    /// Decompress bytes back into a list of tags
    fn decompress(&self, data: &[u8]) -> Result<Vec<String>>;
    
    /// Calculate compression ratio
    fn compression_ratio(&self, original: &[u8], compressed: &[u8]) -> f64 {
        if original.is_empty() {
            return 0.0;
        }
        compressed.len() as f64 / original.len() as f64
    }
    
    /// Get the algorithm name
    fn algorithm_name(&self) -> &'static str;
}

pub use huffman::HuffmanCompressor;
pub use arithmetic::ArithmeticCompressor;
pub use dictionary::DictionaryCompressor;

