//! Storage layer for mappy
//!
//! Provides different storage backends for persistence and durability.

use crate::MapletResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod aof;
pub mod disk;
pub mod hybrid;
pub mod memory;

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageStats {
    /// Total number of keys stored
    pub total_keys: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Number of operations performed
    pub operations_count: u64,
    /// Average operation latency in microseconds
    pub avg_latency_us: u64,
}

/// Storage trait for different backends
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get a value by key
    async fn get(&self, key: &str) -> MapletResult<Option<Vec<u8>>>;

    /// Set a key-value pair
    async fn set(&self, key: String, value: Vec<u8>) -> MapletResult<()>;

    /// Delete a key
    async fn delete(&self, key: &str) -> MapletResult<bool>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> MapletResult<bool>;

    /// Get all keys
    async fn keys(&self) -> MapletResult<Vec<String>>;

    /// Clear all data
    async fn clear_database(&self) -> MapletResult<()>;

    /// Flush any pending writes
    async fn flush(&self) -> MapletResult<()>;

    /// Close the storage backend
    async fn close(&self) -> MapletResult<()>;

    /// Get storage statistics
    async fn stats(&self) -> MapletResult<StorageStats>;
}

/// Storage factory for creating different backends
pub struct StorageFactory;

impl StorageFactory {
    /// Create a storage backend based on persistence mode
    ///
    /// # Errors
    /// Returns an error if storage creation fails for the specified mode.
    pub fn create_storage(
        mode: PersistenceMode,
        config: StorageConfig,
    ) -> MapletResult<Box<dyn Storage>> {
        match mode {
            PersistenceMode::Memory => Ok(Box::new(memory::MemoryStorage::new(config)?)),
            PersistenceMode::Disk => Ok(Box::new(disk::DiskStorage::new(config)?)),
            PersistenceMode::AOF => Ok(Box::new(aof::AOFStorage::new(config)?)),
            PersistenceMode::Hybrid => Ok(Box::new(hybrid::HybridStorage::new(config)?)),
        }
    }
}

/// Persistence mode for storage
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PersistenceMode {
    /// In-memory only (no persistence)
    Memory,
    /// Append-only file
    AOF,
    /// Full durability (synchronous writes)
    Disk,
    /// Hybrid (memory + AOF)
    Hybrid,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Data directory for persistent storage
    pub data_dir: String,
    /// Maximum memory usage in bytes
    pub max_memory: Option<u64>,
    /// Enable compression
    pub enable_compression: bool,
    /// Sync interval for AOF mode (seconds)
    pub sync_interval: u64,
    /// Buffer size for writes
    pub write_buffer_size: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            max_memory: None,
            enable_compression: true,
            sync_interval: 1,
            write_buffer_size: 1024 * 1024, // 1MB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_stats_default() {
        let stats = StorageStats::default();
        assert_eq!(stats.total_keys, 0);
        assert_eq!(stats.memory_usage, 0);
        assert_eq!(stats.disk_usage, 0);
    }

    #[test]
    fn test_storage_stats_serialization() {
        let stats = StorageStats {
            total_keys: 100,
            memory_usage: 1024,
            disk_usage: 2048,
            operations_count: 500,
            avg_latency_us: 100,
        };

        let serialized = serde_json::to_string(&stats).expect("Failed to serialize");
        let deserialized: StorageStats =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.total_keys, 100);
        assert_eq!(deserialized.memory_usage, 1024);
    }

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.data_dir, "./data");
        assert!(config.enable_compression);
        assert_eq!(config.sync_interval, 1);
        assert_eq!(config.write_buffer_size, 1024 * 1024);
    }

    #[test]
    fn test_storage_config_clone() {
        let config = StorageConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.data_dir, config.data_dir);
        assert_eq!(cloned.enable_compression, config.enable_compression);
    }

    #[test]
    fn test_storage_config_serialization() {
        let config = StorageConfig::default();
        let serialized = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: StorageConfig =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.data_dir, config.data_dir);
    }

    #[test]
    fn test_persistence_mode_eq() {
        assert_eq!(PersistenceMode::Memory, PersistenceMode::Memory);
        assert_ne!(PersistenceMode::Memory, PersistenceMode::Disk);
    }

    #[test]
    fn test_persistence_mode_clone() {
        let mode = PersistenceMode::Memory;
        let cloned = mode;
        assert_eq!(cloned, mode);
    }

    #[test]
    fn test_persistence_mode_serialization() {
        let mode = PersistenceMode::Memory;
        let serialized = serde_json::to_string(&mode).expect("Failed to serialize");
        let deserialized: PersistenceMode =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized, mode);
    }

    #[test]
    fn test_storage_config_custom() {
        let config = StorageConfig {
            data_dir: "/tmp/data".to_string(),
            max_memory: Some(1024 * 1024),
            enable_compression: false,
            sync_interval: 5,
            write_buffer_size: 2048,
        };

        assert_eq!(config.data_dir, "/tmp/data");
        assert_eq!(config.max_memory, Some(1024 * 1024));
        assert!(!config.enable_compression);
        assert_eq!(config.sync_interval, 5);
        assert_eq!(config.write_buffer_size, 2048);
    }

    #[test]
    fn test_storage_stats_clone() {
        let stats = StorageStats {
            total_keys: 100,
            memory_usage: 1024,
            disk_usage: 2048,
            operations_count: 500,
            avg_latency_us: 100,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.total_keys, 100);
        assert_eq!(cloned.memory_usage, 1024);
    }
}
