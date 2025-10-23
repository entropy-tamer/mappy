//! Space-efficient value encoding
//! 
//! Implements variable-length encoding for values to optimize memory usage,
//! particularly for counter values as described in the research paper.

use crate::{MapletError, MapletResult};

/// Variable-length encoding for unsigned integers
pub struct VarIntEncoder;

impl VarIntEncoder {
    /// Encode a u64 value using variable-length encoding
    #[must_use]
    pub fn encode_u64(value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        let mut val = value;
        
        while val >= 128 {
            result.push(u8::try_from((val & 0x7F) | 0x80).unwrap_or(0));
            val >>= 7;
        }
        result.push(u8::try_from(val).unwrap_or(0));
        result
    }
    
    /// Decode a u64 value from variable-length encoding
    /// 
    /// # Errors
    /// 
    /// Returns an error if the data is invalid or truncated
    pub fn decode_u64(data: &[u8]) -> MapletResult<(u64, usize)> {
        if data.is_empty() {
            return Err(MapletError::SerializationError("Empty data".to_string()));
        }
        
        let mut result = 0u64;
        let mut shift = 0;
        let mut bytes_read = 0;
        
        for &byte in data {
            bytes_read += 1;
            result |= u64::from(byte & 0x7F) << shift;
            
            if (byte & 0x80) == 0 {
                // Last byte
                return Ok((result, bytes_read));
            }
            
            shift += 7;
            if shift >= 64 {
                return Err(MapletError::SerializationError("Value too large".to_string()));
            }
        }
        
        Err(MapletError::SerializationError("Incomplete encoding".to_string()))
    }
    
    /// Encode a u32 value using variable-length encoding
    #[must_use]
    pub fn encode_u32(value: u32) -> Vec<u8> {
        Self::encode_u64(u64::from(value))
    }
    
    /// Decode a u32 value from variable-length encoding
    /// 
    /// # Errors
    /// 
    /// Returns an error if the data is invalid or the value is too large for u32
    pub fn decode_u32(data: &[u8]) -> MapletResult<(u32, usize)> {
        let (value, bytes_read) = Self::decode_u64(data)?;
        if value > u64::from(u32::MAX) {
            return Err(MapletError::SerializationError("Value too large for u32".to_string()));
        }
        Ok((u32::try_from(value).unwrap_or(0), bytes_read))
    }
}

/// Exponential encoding for counter values
/// 
/// Uses a more space-efficient encoding for values that grow exponentially,
/// as described in the research paper for k-mer counting applications.
pub struct ExponentialEncoder {
    /// Base for exponential encoding
    #[allow(dead_code)]
    base: f64,
    /// Precision for floating-point values
    #[allow(dead_code)]
    precision: u32,
}

impl ExponentialEncoder {
    /// Create a new exponential encoder
    #[must_use]
    pub const fn new(base: f64, precision: u32) -> Self {
        Self { base, precision }
    }
    
    /// Encode a counter value using exponential encoding
    #[must_use]
    pub fn encode_counter(&self, value: u64) -> Vec<u8> {
        if value == 0 {
            return vec![0];
        }
        
        // For simplicity, just use varint encoding for now
        // In a real implementation, this would use exponential encoding
        VarIntEncoder::encode_u64(value)
    }
    
    /// Decode a counter value from exponential encoding
    /// 
    /// # Errors
    /// 
    /// Returns an error if the data is invalid or truncated
    pub fn decode_counter(&self, data: &[u8]) -> MapletResult<(u64, usize)> {
        if data.is_empty() {
            return Err(MapletError::SerializationError("Empty data".to_string()));
        }
        
        if data[0] == 0 {
            return Ok((0, 1));
        }
        
        // For simplicity, just use varint decoding for now
        // In a real implementation, this would use exponential decoding
        VarIntEncoder::decode_u64(data)
    }
}

/// Compact encoding for small values
pub struct CompactEncoder;

impl CompactEncoder {
    /// Encode a small value (≤8 bytes) inline
    pub fn encode_inline<T: Copy + bytemuck::Pod>(value: &T) -> [u8; 8] {
        let mut result = [0u8; 8];
        let bytes = bytemuck::bytes_of(value);
        result[..bytes.len()].copy_from_slice(bytes);
        result
    }
    
    /// Decode a small value from inline encoding
    /// # Errors
    /// 
    /// Returns an error if the data cannot be decoded
    pub fn decode_inline<T: Copy + bytemuck::Pod>(data: &[u8; 8]) -> MapletResult<T> {
        let size = std::mem::size_of::<T>();
        if size > 8 {
            return Err(MapletError::SerializationError("Type too large for inline encoding".to_string()));
        }
        
        let slice = &data[..size];
        bytemuck::try_from_bytes(slice)
            .copied()
            .map_err(|e| MapletError::SerializationError(format!("Decode error: {e}")))
    }
    
    /// Check if a value can be encoded inline
    pub const fn can_encode_inline<T>(_value: &T) -> bool {
        std::mem::size_of::<T>() <= 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encoding() {
        // Test small values
        assert_eq!(VarIntEncoder::encode_u64(0), vec![0]);
        assert_eq!(VarIntEncoder::encode_u64(127), vec![127]);
        
        // Test medium values
        assert_eq!(VarIntEncoder::encode_u64(128), vec![0x80, 0x01]);
        assert_eq!(VarIntEncoder::encode_u64(16383), vec![0xFF, 0x7F]);
        
        // Test round-trip encoding
        for value in [0, 1, 127, 128, 16383, 16384, 1_000_000, u64::MAX] {
            let encoded = VarIntEncoder::encode_u64(value);
            let (decoded, bytes_read) = VarIntEncoder::decode_u64(&encoded).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(bytes_read, encoded.len());
        }
    }

    #[test]
    fn test_exponential_encoding() {
        let encoder = ExponentialEncoder::new(2.0, 16);
        
        // Test round-trip encoding
        for value in [0, 1, 2, 4, 8, 16, 100, 1000, 10000] {
            let encoded_data = encoder.encode_counter(value);
            let (decoded, bytes_read) = encoder.decode_counter(&encoded_data).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(bytes_read, encoded_data.len());
        }
    }

    #[test]
    fn test_compact_encoding() {
        // Test inline encoding for small values
        let value: u32 = 0x1234_5678;
        let encoded = CompactEncoder::encode_inline(&value);
        let decoded: u32 = CompactEncoder::decode_inline(&encoded).unwrap();
        assert_eq!(decoded, value);
        
        // Test inline encoding for u64
        let value: u64 = 0x1234_5678_9ABC_DEF0;
        let encoded = CompactEncoder::encode_inline(&value);
        let decoded: u64 = CompactEncoder::decode_inline(&encoded).unwrap();
        assert_eq!(decoded, value);
    }
}
