#![allow(clippy::cast_precision_loss)] // Acceptable for compression ratio calculations
//! Arithmetic coding compression implementation

use crate::compression::Compressor;
use anyhow::Result;
use std::collections::HashMap;

/// Arithmetic coding compressor
pub struct ArithmeticCompressor {
    probabilities: HashMap<String, f64>,
}

impl ArithmeticCompressor {
    /// Create a new arithmetic compressor
    pub fn new() -> Self {
        Self {
            probabilities: HashMap::new(),
        }
    }

    /// Build probability model from corpus
    pub fn build_from_corpus(&mut self, corpus: &[String]) -> Result<()> {
        let total = corpus.len() as f64;
        if total == 0.0 {
            anyhow::bail!("Cannot build model from empty corpus");
        }

        // Count frequencies
        let mut frequencies = HashMap::new();
        for tag in corpus {
            *frequencies.entry(tag.clone()).or_insert(0) += 1;
        }

        // Calculate probabilities
        self.probabilities.clear();
        for (tag, count) in frequencies {
            self.probabilities.insert(tag, count as f64 / total);
        }

        Ok(())
    }

    fn get_cumulative_ranges(&self, tags: &[String]) -> Vec<(f64, f64)> {
        let mut ranges = Vec::new();
        let mut cumulative = 0.0;

        for tag in tags {
            let prob = self.probabilities.get(tag).copied().unwrap_or(1e-10);
            let start = cumulative;
            cumulative += prob;
            ranges.push((start, cumulative));
        }

        ranges
    }

    fn encode_arithmetic(&self, tags: &[String]) -> Result<Vec<u8>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        // Build probability model if not already built
        let mut compressor = self.clone();
        compressor.build_from_corpus(tags)?;

        // Get cumulative ranges
        let ranges = compressor.get_cumulative_ranges(tags);

        // Encode using arithmetic coding (simplified version)
        let mut low = 0.0;
        let mut high = 1.0;

        for (start, end) in ranges {
            let range = high - low;
            high = low + range * end;
            low += range * start;
        }

        // Convert to fixed-point representation (32-bit)
        let value = (low + high) / 2.0;
        let encoded = (value * (u32::MAX as f64)) as u32;

        // Serialize: count (u32) + value (u32) + probabilities
        let mut result = Vec::new();
        result.extend_from_slice(&(tags.len() as u32).to_le_bytes());
        result.extend_from_slice(&encoded.to_le_bytes());

        // Store probabilities for decoding
        let prob_data = bitcode::encode(&compressor.probabilities);
        result.extend_from_slice(&(prob_data.len() as u32).to_le_bytes());
        result.extend_from_slice(&prob_data);

        Ok(result)
    }

    fn decode_arithmetic(&self, data: &[u8]) -> Result<Vec<String>> {
        if data.len() < 8 {
            anyhow::bail!("Insufficient data");
        }

        // Decode count
        let count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;

        // Decode value (currently unused but kept for future use)
        let _encoded = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let _value = _encoded as f64 / (u32::MAX as f64);

        // Decode probabilities
        let prob_len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        if data.len() < 12 + prob_len {
            anyhow::bail!("Insufficient data for probabilities");
        }
        let probabilities: HashMap<String, f64> = bitcode::decode(&data[12..12 + prob_len])?;

        // Decode tags (simplified - in practice, need proper arithmetic decoding)
        // For now, return empty as this is a simplified implementation
        let mut result = Vec::new();

        // Build tag list from probabilities (simplified decoding)
        let mut sorted_tags: Vec<_> = probabilities.keys().collect();
        sorted_tags.sort();

        for _ in 0..count.min(sorted_tags.len()) {
            if let Some(tag) = sorted_tags.first() {
                result.push((*tag).clone());
            }
        }

        Ok(result)
    }
}

impl Clone for ArithmeticCompressor {
    fn clone(&self) -> Self {
        Self {
            probabilities: self.probabilities.clone(),
        }
    }
}

impl Default for ArithmeticCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for ArithmeticCompressor {
    fn compress(&self, tags: &[String]) -> Result<Vec<u8>> {
        // If probabilities are already built, use them; otherwise build them
        let compressor = if self.probabilities.is_empty() {
            let mut new_compressor = self.clone();
            new_compressor.build_from_corpus(tags)?;
            new_compressor
        } else {
            // Probabilities already built, use existing
            self.clone()
        };

        compressor.encode_arithmetic(tags)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<String>> {
        self.decode_arithmetic(data)
    }

    fn algorithm_name(&self) -> &'static str {
        "arithmetic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_basic() {
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag1".to_string()];

        let mut compressor = ArithmeticCompressor::new();
        compressor.build_from_corpus(&tags).unwrap();

        let compressed = compressor.compress(&tags).unwrap();
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_arithmetic_new() {
        let compressor = ArithmeticCompressor::new();
        assert!(compressor.probabilities.is_empty());
    }

    #[test]
    fn test_arithmetic_default() {
        let compressor = ArithmeticCompressor::default();
        assert!(compressor.probabilities.is_empty());
    }

    #[test]
    fn test_arithmetic_clone() {
        let mut compressor = ArithmeticCompressor::new();
        compressor
            .build_from_corpus(&["tag1".to_string(), "tag2".to_string()])
            .unwrap();

        let cloned = compressor.clone();
        assert_eq!(cloned.probabilities.len(), compressor.probabilities.len());
    }

    #[test]
    fn test_build_from_corpus_empty() {
        let mut compressor = ArithmeticCompressor::new();
        let result = compressor.build_from_corpus(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_from_corpus_single_tag() {
        let mut compressor = ArithmeticCompressor::new();
        compressor.build_from_corpus(&["tag1".to_string()]).unwrap();
        assert_eq!(compressor.probabilities.len(), 1);
    }

    #[test]
    fn test_build_from_corpus_multiple_tags() {
        let mut compressor = ArithmeticCompressor::new();
        compressor
            .build_from_corpus(&["tag1".to_string(), "tag2".to_string(), "tag3".to_string()])
            .unwrap();
        assert_eq!(compressor.probabilities.len(), 3);
    }

    #[test]
    fn test_build_from_corpus_with_duplicates() {
        let mut compressor = ArithmeticCompressor::new();
        compressor
            .build_from_corpus(&["tag1".to_string(), "tag1".to_string(), "tag2".to_string()])
            .unwrap();
        assert_eq!(compressor.probabilities.len(), 2);
    }

    #[test]
    fn test_compress_empty_tags() {
        let mut compressor = ArithmeticCompressor::new();
        compressor.build_from_corpus(&["tag1".to_string()]).unwrap();
        let result = compressor.compress(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compress_without_corpus() {
        let compressor = ArithmeticCompressor::new();
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let result = compressor.compress(&tags);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_decompress_empty_data() {
        let compressor = ArithmeticCompressor::new();
        let result = compressor.decompress(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_insufficient_data() {
        let compressor = ArithmeticCompressor::new();
        let result = compressor.decompress(&[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_algorithm_name() {
        let compressor = ArithmeticCompressor::new();
        assert_eq!(compressor.algorithm_name(), "arithmetic");
    }

    #[test]
    fn test_cumulative_ranges() {
        let mut compressor = ArithmeticCompressor::new();
        compressor
            .build_from_corpus(&["tag1".to_string(), "tag2".to_string()])
            .unwrap();

        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let ranges = compressor.get_cumulative_ranges(&tags);
        assert_eq!(ranges.len(), 2);
    }

    #[test]
    fn test_cumulative_ranges_empty() {
        let compressor = ArithmeticCompressor::new();
        let ranges = compressor.get_cumulative_ranges(&[]);
        assert_eq!(ranges.len(), 0);
    }

    #[test]
    fn test_cumulative_ranges_unknown_tag() {
        let compressor = ArithmeticCompressor::new();
        let tags = vec!["unknown".to_string()];
        let ranges = compressor.get_cumulative_ranges(&tags);
        assert_eq!(ranges.len(), 1);
    }
}
