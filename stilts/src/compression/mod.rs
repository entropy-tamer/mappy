#![allow(clippy::cast_precision_loss)] // Acceptable for compression ratio calculations
//! Compression algorithms for tag compression

pub mod arithmetic;
pub mod dictionary;
pub mod huffman;

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

pub use arithmetic::ArithmeticCompressor;
pub use dictionary::DictionaryCompressor;
pub use huffman::HuffmanCompressor;

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCompressor;

    impl Compressor for MockCompressor {
        fn compress(&self, tags: &[String]) -> anyhow::Result<Vec<u8>> {
            Ok(tags.join(",").as_bytes().to_vec())
        }

        fn decompress(&self, data: &[u8]) -> anyhow::Result<Vec<String>> {
            let str_data = std::str::from_utf8(data).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))?;
            Ok(str_data.split(',').map(|s| s.to_string()).collect())
        }

        fn algorithm_name(&self) -> &'static str {
            "mock"
        }
    }

    #[test]
    fn test_compressor_trait_compress() {
        let compressor = MockCompressor;
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let result = compressor.compress(&tags);
        assert!(result.is_ok());
        let compressed = result.unwrap();
        assert_eq!(compressed, b"tag1,tag2");
    }

    #[test]
    fn test_compressor_trait_decompress() {
        let compressor = MockCompressor;
        let data = b"tag1,tag2";
        let result = compressor.decompress(data);
        assert!(result.is_ok());
        let decompressed = result.unwrap();
        assert_eq!(decompressed, vec!["tag1".to_string(), "tag2".to_string()]);
    }

    #[test]
    fn test_compressor_trait_algorithm_name() {
        let compressor = MockCompressor;
        assert_eq!(compressor.algorithm_name(), "mock");
    }

    #[test]
    fn test_compression_ratio() {
        let compressor = MockCompressor;
        let original = b"tag1,tag2,tag3";
        let compressed = b"tag1,tag2";
        let ratio = compressor.compression_ratio(original, compressed);
        assert!((ratio - 2.0 / 3.0).abs() < 0.1);
    }

    #[test]
    fn test_compression_ratio_empty_original() {
        let compressor = MockCompressor;
        let original = b"";
        let compressed = b"";
        let ratio = compressor.compression_ratio(original, compressed);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_compression_ratio_perfect() {
        let compressor = MockCompressor;
        let original = b"data";
        let compressed = b"data";
        let ratio = compressor.compression_ratio(original, compressed);
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_compression_ratio_half() {
        let compressor = MockCompressor;
        let original = b"dddd";
        let compressed = b"dd";
        let ratio = compressor.compression_ratio(original, compressed);
        assert_eq!(ratio, 0.5);
    }
}
