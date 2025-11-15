//! Common types and error handling for maplet operations

use thiserror::Error;

/// Error types for maplet operations
#[derive(Error, Debug)]
pub enum MapletError {
    #[error("Maplet is at capacity")]
    CapacityExceeded,

    #[error("Invalid false-positive rate: {0} (must be between 0 and 1)")]
    InvalidErrorRate(f64),

    #[error("Invalid capacity: {0} (must be > 0)")]
    InvalidCapacity(usize),

    #[error("Hash collision limit exceeded")]
    CollisionLimitExceeded,

    #[error("Merge operation failed: {0}")]
    MergeFailed(String),

    #[error("Resize operation failed: {0}")]
    ResizeFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for maplet operations
pub type MapletResult<T> = std::result::Result<T, MapletError>;

/// Statistics for a maplet
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MapletStats {
    /// Current capacity of the maplet
    pub capacity: usize,

    /// Current number of items stored
    pub len: usize,

    /// Load factor (len / capacity)
    pub load_factor: f64,

    /// Configured false-positive rate
    pub false_positive_rate: f64,

    /// Estimated memory usage in bytes
    pub memory_usage: usize,

    /// Number of hash collisions encountered
    pub num_collisions: u64,

    /// Number of slots used in the filter
    pub slots_used: usize,

    /// Average chain length for collision resolution
    pub avg_chain_length: f64,
}

impl MapletStats {
    /// Create new statistics
    #[must_use]
    pub fn new(capacity: usize, len: usize, false_positive_rate: f64) -> Self {
        Self {
            capacity,
            len,
            load_factor: if capacity > 0 {
                #[allow(clippy::cast_precision_loss)]
                {
                    len as f64 / capacity as f64
                }
            } else {
                0.0
            },
            false_positive_rate,
            memory_usage: 0,
            num_collisions: 0,
            slots_used: 0,
            avg_chain_length: 0.0,
        }
    }

    /// Update statistics with new values
    pub fn update(
        &mut self,
        len: usize,
        memory_usage: usize,
        num_collisions: u64,
        slots_used: usize,
    ) {
        self.len = len;
        self.load_factor = if self.capacity > 0 {
            #[allow(clippy::cast_precision_loss)]
            {
                len as f64 / self.capacity as f64
            }
        } else {
            0.0
        };
        self.memory_usage = memory_usage;
        self.num_collisions = num_collisions;
        self.slots_used = slots_used;
        self.avg_chain_length = if slots_used > 0 {
            #[allow(clippy::cast_precision_loss)]
            {
                num_collisions as f64 / slots_used as f64
            }
        } else {
            0.0
        };
    }
}

/// Configuration for maplet creation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MapletConfig {
    /// Initial capacity
    pub capacity: usize,

    /// Target false-positive rate
    pub false_positive_rate: f64,

    /// Maximum load factor before resizing
    pub max_load_factor: f64,

    /// Enable automatic resizing
    pub auto_resize: bool,

    /// Enable deletion support
    pub enable_deletion: bool,

    /// Enable merging support
    pub enable_merging: bool,
}

impl Default for MapletConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,
            false_positive_rate: 0.01,
            max_load_factor: 0.95,
            auto_resize: true,
            enable_deletion: true,
            enable_merging: true,
        }
    }
}

impl MapletConfig {
    /// Create a new configuration
    #[must_use]
    pub fn new(capacity: usize, false_positive_rate: f64) -> Self {
        Self {
            capacity,
            false_positive_rate,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> MapletResult<()> {
        if self.capacity == 0 {
            return Err(MapletError::InvalidCapacity(self.capacity));
        }

        if self.false_positive_rate <= 0.0 || self.false_positive_rate >= 1.0 {
            return Err(MapletError::InvalidErrorRate(self.false_positive_rate));
        }

        if self.max_load_factor <= 0.0 || self.max_load_factor > 1.0 {
            return Err(MapletError::InvalidErrorRate(self.max_load_factor));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maplet_stats() {
        let stats = MapletStats::new(1000, 500, 0.01);
        assert_eq!(stats.capacity, 1000);
        assert_eq!(stats.len, 500);
        assert_eq!(stats.load_factor, 0.5);
        assert_eq!(stats.false_positive_rate, 0.01);
    }

    #[test]
    fn test_maplet_config_validation() {
        let config = MapletConfig::new(1000, 0.01);
        assert!(config.validate().is_ok());

        let invalid_capacity = MapletConfig::new(0, 0.01);
        assert!(invalid_capacity.validate().is_err());

        let invalid_error_rate = MapletConfig::new(1000, 1.5);
        assert!(invalid_error_rate.validate().is_err());
    }
}
