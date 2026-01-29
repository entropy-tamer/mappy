#![allow(clippy::cast_precision_loss)] // Acceptable for compression ratio calculations
//! Custom dictionary compression with variable-length codes

use crate::compression::Compressor;
use anyhow::{Context, Result};
use bitvec::prelude::*;
use std::collections::HashMap;

/// Dictionary-based compressor with variable-length codes
pub struct DictionaryCompressor {
    dictionary: HashMap<String, u32>,
    reverse_dictionary: HashMap<u32, String>,
    next_code: u32,
}

impl DictionaryCompressor {
    /// Create a new dictionary compressor
    pub fn new() -> Self {
        Self {
            dictionary: HashMap::new(),
            reverse_dictionary: HashMap::new(),
            next_code: 0,
        }
    }

    /// Build dictionary from corpus
    pub fn build_from_corpus(&mut self, corpus: &[String]) -> Result<()> {
        // Count frequencies
        let mut frequencies = HashMap::new();
        for tag in corpus {
            *frequencies.entry(tag.clone()).or_insert(0) += 1;
        }

        // Sort by frequency (descending)
        let mut sorted: Vec<_> = frequencies.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        // Assign codes based on frequency (shorter codes for more frequent tags)
        self.dictionary.clear();
        self.reverse_dictionary.clear();
        self.next_code = 0;

        for (tag, _freq) in sorted {
            if !self.dictionary.contains_key(&tag) {
                self.dictionary.insert(tag.clone(), self.next_code);
                self.reverse_dictionary.insert(self.next_code, tag);
                self.next_code += 1;
            }
        }

        Ok(())
    }

    fn code_to_bits(&self, code: u32) -> BitVec<u8, Lsb0> {
        // Use variable-length encoding:
        // - Codes 0-127: 1 byte
        // - Codes 128-16383: 2 bytes
        // - Codes 16384+: 4 bytes
        let mut bits = BitVec::new();

        if code < 128 {
            bits.extend_from_bitslice((code as u8).view_bits::<Lsb0>());
        } else if code < 16384 {
            // Use 2 bytes with continuation bit
            let byte1 = ((code & 0x7F) | 0x80) as u8;
            let byte2 = ((code >> 7) & 0xFF) as u8;
            bits.extend_from_bitslice(byte1.view_bits::<Lsb0>());
            bits.extend_from_bitslice(byte2.view_bits::<Lsb0>());
        } else {
            // Use 4 bytes
            bits.extend_from_bitslice(code.to_le_bytes().view_bits::<Lsb0>());
        }

        bits
    }

    fn bits_to_code(&self, bits: &mut &BitSlice<u8, Lsb0>) -> Result<u32> {
        if bits.is_empty() {
            anyhow::bail!("Insufficient bits for code");
        }

        let first_byte = bits[0..8].load::<u8>();

        if (first_byte & 0x80) == 0 {
            // 1 byte code
            *bits = &bits[8..];
            Ok(first_byte as u32)
        } else if bits.len() >= 16 {
            // 2 byte code
            let byte1 = first_byte & 0x7F;
            let byte2 = bits[8..16].load::<u8>();
            *bits = &bits[16..];
            Ok((byte2 as u32) << 7 | (byte1 as u32))
        } else if bits.len() >= 32 {
            // 4 byte code
            let code = bits[0..32].load::<u32>();
            *bits = &bits[32..];
            Ok(code)
        } else {
            anyhow::bail!("Invalid code encoding");
        }
    }

    fn encode_tags(&self, tags: &[String]) -> Result<BitVec<u8, Lsb0>> {
        let mut result = BitVec::new();

        // Encode dictionary size
        result.extend_from_bitslice(
            (self.dictionary.len() as u32)
                .to_le_bytes()
                .view_bits::<Lsb0>(),
        );

        // Encode dictionary entries
        for (tag, code) in &self.dictionary {
            // Encode tag length and tag
            let tag_bytes = tag.as_bytes();
            result.extend_from_bitslice((tag_bytes.len() as u32).to_le_bytes().view_bits::<Lsb0>());
            result.extend_from_bitslice(tag_bytes.view_bits::<Lsb0>());
            result.extend_from_bitslice(code.to_le_bytes().view_bits::<Lsb0>());
        }

        // Encode number of tags
        result.extend_from_bitslice((tags.len() as u32).to_le_bytes().view_bits::<Lsb0>());

        // Encode tags using codes
        for tag in tags {
            let code = self
                .dictionary
                .get(tag)
                .with_context(|| format!("Tag not in dictionary: {}", tag))?;
            result.extend_from_bitslice(&self.code_to_bits(*code));
        }

        Ok(result)
    }

    fn decode_tags(&self, bits: &BitSlice<u8, Lsb0>) -> Result<Vec<String>> {
        let mut pos = 0;
        let mut dictionary = HashMap::new();
        let mut reverse_dict = HashMap::new();

        // Decode dictionary size
        if pos + 32 > bits.len() {
            anyhow::bail!("Insufficient data for dictionary size");
        }
        let dict_size = bits[pos..pos + 32].load::<u32>() as usize;
        pos += 32;

        // Decode dictionary
        for _ in 0..dict_size {
            // Decode tag length
            if pos + 32 > bits.len() {
                anyhow::bail!("Insufficient data for tag length");
            }
            let tag_len = bits[pos..pos + 32].load::<u32>() as usize;
            pos += 32;

            // Decode tag
            if pos + tag_len * 8 > bits.len() {
                anyhow::bail!("Insufficient data for tag");
            }
            let tag_bytes: Vec<u8> = (0..tag_len)
                .map(|i| bits[pos + i * 8..pos + (i + 1) * 8].load::<u8>())
                .collect();
            let tag = String::from_utf8(tag_bytes)?;
            pos += tag_len * 8;

            // Decode code
            if pos + 32 > bits.len() {
                anyhow::bail!("Insufficient data for code");
            }
            let code = bits[pos..pos + 32].load::<u32>();
            pos += 32;

            dictionary.insert(tag.clone(), code);
            reverse_dict.insert(code, tag);
        }

        // Decode number of tags
        if pos + 32 > bits.len() {
            anyhow::bail!("Insufficient data for tag count");
        }
        let count = bits[pos..pos + 32].load::<u32>() as usize;
        pos += 32;

        // Decode tags
        let mut result = Vec::new();
        let mut remaining_bits = &bits[pos..];

        for _ in 0..count {
            let code = self.bits_to_code(&mut remaining_bits)?;
            let tag = reverse_dict
                .get(&code)
                .with_context(|| format!("Code not in dictionary: {}", code))?;
            result.push(tag.clone());
        }

        Ok(result)
    }
}

impl Clone for DictionaryCompressor {
    fn clone(&self) -> Self {
        Self {
            dictionary: self.dictionary.clone(),
            reverse_dictionary: self.reverse_dictionary.clone(),
            next_code: self.next_code,
        }
    }
}

impl Default for DictionaryCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for DictionaryCompressor {
    fn compress(&self, tags: &[String]) -> Result<Vec<u8>> {
        // If dictionary is already built, use it; otherwise build it
        let compressor = if self.dictionary.is_empty() {
            let mut new_compressor = self.clone();
            new_compressor.build_from_corpus(tags)?;
            new_compressor
        } else {
            // Dictionary already built, use existing
            self.clone()
        };

        // Encode tags
        let bits = compressor.encode_tags(tags)?;

        // Convert to bytes
        Ok(bits.into_vec())
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<String>> {
        let bits = data.view_bits::<Lsb0>();
        self.decode_tags(bits)
    }

    fn algorithm_name(&self) -> &'static str {
        "dictionary"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_basic() {
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag1".to_string()];

        let mut compressor = DictionaryCompressor::new();
        compressor.build_from_corpus(&tags).unwrap();

        let compressed = compressor.compress(&tags).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(tags, decompressed);
    }
}
