//! Storage layer for mappy
//! 
//! Provides different storage backends for persistence and durability.

use async_trait::async_trait;
use crate::MapletResult;
use serde::{Serialize, Deserialize};

pub mod memory;
pub mod disk;
pub mod aof;
pub mod hybrid;

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
    pub async fn create_storage(
        mode: PersistenceMode,
        config: StorageConfig,
    ) -> MapletResult<Box<dyn Storage>> {
        match mode {
            PersistenceMode::Memory => {
                Ok(Box::new(memory::MemoryStorage::new(config)?))
            }
            PersistenceMode::Disk => {
                Ok(Box::new(disk::DiskStorage::new(config)?))
            }
            PersistenceMode::AOF => {
                Ok(Box::new(aof::AOFStorage::new(config)?))
            }
            PersistenceMode::Hybrid => {
                Ok(Box::new(hybrid::HybridStorage::new(config)?))
            }
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
