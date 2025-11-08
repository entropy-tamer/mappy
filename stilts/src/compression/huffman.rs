//! Huffman coding compression implementation

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use anyhow::{Result, Context};
use bitvec::prelude::*;
use crate::compression::Compressor;

#[derive(Debug, Clone, Eq, PartialEq)]
struct HuffmanNode {
    frequency: usize,
    tag: Option<String>,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.frequency.cmp(&self.frequency)
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Huffman coding compressor
#[derive(Clone)]
pub struct HuffmanCompressor {
    codes: HashMap<String, BitVec<u8, Lsb0>>,
    reverse_codes: HashMap<BitVec<u8, Lsb0>, String>,
}

impl HuffmanCompressor {
    /// Create a new Huffman compressor from a corpus of tags
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
            reverse_codes: HashMap::new(),
        }
    }
    
    /// Build Huffman codes from a corpus of tags
    pub fn build_from_corpus(&mut self, corpus: &[String]) -> Result<()> {
        // Count frequencies
        let mut frequencies = HashMap::new();
        for tag in corpus {
            *frequencies.entry(tag.clone()).or_insert(0) += 1;
        }
        
        // Build Huffman tree
        let root = self.build_tree(&frequencies)?;
        
        // Generate codes
        self.codes.clear();
        self.reverse_codes.clear();
        self.generate_codes(&root, BitVec::new());
        
        Ok(())
    }
    
    fn build_tree(&self, frequencies: &HashMap<String, usize>) -> Result<HuffmanNode> {
        if frequencies.is_empty() {
            anyhow::bail!("Cannot build tree from empty frequency table");
        }
        
        let mut heap = BinaryHeap::new();
        
        // Create leaf nodes
        for (tag, freq) in frequencies {
            heap.push(HuffmanNode {
                frequency: *freq,
                tag: Some(tag.clone()),
                left: None,
                right: None,
            });
        }
        
        // Build tree
        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            
            let merged = HuffmanNode {
                frequency: left.frequency + right.frequency,
                tag: None,
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
            };
            
            heap.push(merged);
        }
        
        Ok(heap.pop().context("Failed to build Huffman tree")?)
    }
    
    fn generate_codes(&mut self, node: &HuffmanNode, mut code: BitVec<u8, Lsb0>) {
        if let Some(ref tag) = node.tag {
            // Leaf node
            self.codes.insert(tag.clone(), code.clone());
            self.reverse_codes.insert(code, tag.clone());
        } else {
            // Internal node
            if let Some(ref left) = node.left {
                let mut left_code = code.clone();
                left_code.push(false);
                self.generate_codes(left, left_code);
            }
            
            if let Some(ref right) = node.right {
                code.push(true);
                self.generate_codes(right, code);
            }
        }
    }
    
    fn encode_tags(&self, tags: &[String]) -> Result<BitVec<u8, Lsb0>> {
        let mut result = BitVec::new();
        
        // Encode number of tags (u32)
        let count = tags.len() as u32;
        result.extend_from_bitslice(&count.view_bits::<Lsb0>());
        
        // Encode each tag
        for tag in tags {
            let code = self.codes.get(tag)
                .with_context(|| format!("Tag not found in dictionary: {}", tag))?;
            result.extend_from_bitslice(code);
        }
        
        Ok(result)
    }
    
    fn decode_tags(&self, bits: &BitSlice<u8, Lsb0>) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut pos = 0;
        
        // Decode number of tags
        if pos + 32 > bits.len() {
            anyhow::bail!("Insufficient data for tag count");
        }
        let mut count_bytes = [0u8; 4];
        for i in 0..4 {
            if pos + i * 8 + 8 > bits.len() {
                anyhow::bail!("Insufficient data for tag count");
            }
            count_bytes[i] = bits[pos + i*8..pos + (i+1)*8].load::<u8>();
        }
        let count = u32::from_le_bytes(count_bytes) as usize;
        pos += 32;
        
        // Decode each tag
        for _ in 0..count {
            let mut current_code: BitVec<u8, Lsb0> = BitVec::new();
            let mut found = false;
            
            // Try to match codes
            for (code, tag) in &self.reverse_codes {
                if pos + code.len() <= bits.len() {
                    let slice = &bits[pos..pos + code.len()];
                    if slice == code.as_bitslice() {
                        result.push(tag.clone());
                        pos += code.len();
                        found = true;
                        break;
                    }
                }
            }
            
            if !found {
                anyhow::bail!("Failed to decode tag at position {}", pos);
            }
        }
        
        Ok(result)
    }
}

impl Default for HuffmanCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for HuffmanCompressor {
    fn compress(&self, tags: &[String]) -> Result<Vec<u8>> {
        // If corpus is already built, use it; otherwise build it
        let compressor = if self.codes.is_empty() {
            let mut new_compressor = HuffmanCompressor {
                codes: self.codes.clone(),
                reverse_codes: self.reverse_codes.clone(),
            };
            new_compressor.build_from_corpus(tags)?;
            new_compressor
        } else {
            // Corpus already built, use existing codes
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
        "huffman"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_huffman_basic() {
        let tags = vec![
            "tag1".to_string(),
            "tag2".to_string(),
            "tag1".to_string(),
        ];
        
        let mut compressor = HuffmanCompressor::new();
        compressor.build_from_corpus(&tags).unwrap();
        
        let compressed = compressor.compress(&tags).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        
        assert_eq!(tags, decompressed);
    }
}

