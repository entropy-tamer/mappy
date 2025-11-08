//! Stilts - Tag Compression Library
//!
//! A comprehensive tag compression library supporting multiple algorithms:
//! - Huffman coding (frequency-based)
//! - Arithmetic coding
//! - Custom dictionary with variable-length codes
//!
//! # Example
//!
//! ```rust
//! use stilts::{HuffmanCompressor, SpaceSeparatedParser};
//!
//! let parser = SpaceSeparatedParser::new();
//! let tags = parser.parse("tag1 tag2 tag3").unwrap();
//!
//! let compressor = HuffmanCompressor::new();
//! let compressed = compressor.compress(&tags).unwrap();
//! let decompressed = compressor.decompress(&compressed).unwrap();
//! ```

pub mod compression;
pub mod formats;
pub mod benchmark;
pub mod plotting;

#[cfg(feature = "mappy-integration")]
pub mod mappy_integration;

pub use compression::{Compressor, HuffmanCompressor, ArithmeticCompressor, DictionaryCompressor};
pub use formats::{TagParser, SpaceSeparatedParser, CommaSeparatedParser, JsonParser};

