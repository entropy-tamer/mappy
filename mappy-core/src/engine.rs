//! Engine layer for mappy
//! 
//! Integrates storage backends with maplet functionality to provide a complete key-value store.

use crate::{Maplet, MapletResult, MergeOperator};
use crate::types::MapletConfig;
use crate::storage::{Storage, StorageStats, StorageConfig, PersistenceMode};
use crate::storage::memory::MemoryStorage;
use crate::storage::disk::DiskStorage;
use crate::storage::aof::AOFStorage;
use crate::storage::hybrid::HybridStorage;
use crate::ttl::{TTLManager, TTLConfig, TTLStats};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

/// Simple merge operator for Vec<u8> that replaces values
#[derive(Debug, Clone, Default)]
pub struct ReplaceOperator;

impl MergeOperator<Vec<u8>> for ReplaceOperator {
    fn merge(&self, _existing: Vec<u8>, new: Vec<u8>) -> MapletResult<Vec<u8>> {
        Ok(new)
    }

    fn identity(&self) -> Vec<u8> {
        Vec::new()
    }
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Maplet configuration
    pub maplet: MapletConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// TTL configuration
    pub ttl: TTLConfig,
    /// Persistence mode
    pub persistence_mode: PersistenceMode,
    /// Data directory for persistent storage
    pub data_dir: Option<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            maplet: MapletConfig::default(),
            storage: StorageConfig::default(),
            ttl: TTLConfig::default(),
            persistence_mode: PersistenceMode::Memory,
            data_dir: None,
        }
    }
}

/// Engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    /// Maplet statistics
    pub maplet_stats: crate::MapletStats,
    /// Storage statistics
    pub storage_stats: StorageStats,
    /// TTL statistics
    pub ttl_stats: TTLStats,
    /// Engine uptime in seconds
    pub uptime_seconds: u64,
    /// Total operations performed
    pub total_operations: u64,
}

/// Main engine that combines maplet with storage
pub struct Engine {
    /// The maplet for approximate key-value operations
    maplet: Arc<RwLock<Maplet<String, Vec<u8>, ReplaceOperator>>>,
    /// The storage backend
    storage: Arc<dyn Storage>,
    /// TTL manager
    ttl_manager: Arc<TTLManager>,
    /// Engine configuration
    config: EngineConfig,
    /// Start time for uptime calculation
    start_time: SystemTime,
    /// Operation counter
    operation_count: Arc<RwLock<u64>>,
}

impl Engine {
    /// Create a new engine with the given configuration
    pub async fn new(config: EngineConfig) -> MapletResult<Self> {
        let maplet = Arc::new(RwLock::new(
            Maplet::<String, Vec<u8>, ReplaceOperator>::with_config(config.maplet.clone())?
        ));

        let storage: Arc<dyn Storage> = match config.persistence_mode {
            PersistenceMode::Memory => {
                Arc::new(MemoryStorage::new(config.storage.clone()).await?)
            },
            PersistenceMode::Disk => {
                Arc::new(DiskStorage::new(config.storage.clone()).await?)
            },
            PersistenceMode::AOF => {
                Arc::new(AOFStorage::new(config.storage.clone()).await?)
            },
            PersistenceMode::Hybrid => {
                Arc::new(HybridStorage::new(config.storage.clone()).await?)
            },
        };

        // Create TTL manager
        let ttl_manager = Arc::new(TTLManager::new(config.ttl.clone()));

        // Start TTL cleanup task
        let storage_clone = Arc::clone(&storage);
        ttl_manager.start_cleanup(move |expired_entries| {
            let storage = Arc::clone(&storage_clone);
            
            // Spawn async task for cleanup
            tokio::spawn(async move {
                for entry in expired_entries {
                    // Remove expired key from storage
                    let _ = storage.delete(&entry.key).await;
                }
            });
            
            Ok(())
        }).await?;

        Ok(Self {
            maplet,
            storage,
            ttl_manager,
            config,
            start_time: SystemTime::now(),
            operation_count: Arc::new(RwLock::new(0)),
        })
    }

    /// Get a value by key
    pub async fn get(&self, key: &str) -> MapletResult<Option<Vec<u8>>> {
        // Check if key has expired
        if self.ttl_manager.is_expired(key).await? {
            // Remove expired key
            self.ttl_manager.remove_ttl(key).await?;
            let _ = self.storage.delete(key).await;
            return Ok(None);
        }

        // First check the maplet for approximate membership
        let maplet_guard = self.maplet.read().await;
        if !maplet_guard.contains(&key.to_string()).await {
            drop(maplet_guard);
            return Ok(None);
        }
        drop(maplet_guard);

        // If the key exists in the maplet, get the actual value from storage
        let result = self.storage.get(key).await;
        
        // Increment operation counter
        {
            let mut count = self.operation_count.write().await;
            *count += 1;
        }

        result
    }

    /// Set a key-value pair
    pub async fn set(&self, key: String, value: Vec<u8>) -> MapletResult<()> {
        // Store in the maplet for approximate membership
        {
            let maplet_guard = self.maplet.write().await;
            maplet_guard.insert(key.clone(), value.clone()).await?;
        }

        // Store in the actual storage backend
        self.storage.set(key, value).await?;

        // Increment operation counter
        {
            let mut count = self.operation_count.write().await;
            *count += 1;
        }

        Ok(())
    }

    /// Delete a key
    pub async fn delete(&self, key: &str) -> MapletResult<bool> {
        // Remove from storage
        let result = self.storage.delete(key).await?;

        // If the key was successfully deleted from storage, we could remove it from maplet
        // but since maplet is just for approximate membership, we'll leave it as is
        // The false positives will be handled by the storage layer

        // Increment operation counter
        {
            let mut count = self.operation_count.write().await;
            *count += 1;
        }

        Ok(result)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> MapletResult<bool> {
        // Check maplet first for fast approximate membership
        let maplet_guard = self.maplet.read().await;
        if !maplet_guard.contains(&key.to_string()).await {
            drop(maplet_guard);
            return Ok(false);
        }
        drop(maplet_guard);

        // If it exists in maplet, check storage for definitive answer
        let result = self.storage.exists(key).await?;

        // Increment operation counter
        {
            let mut count = self.operation_count.write().await;
            *count += 1;
        }

        Ok(result)
    }

    /// Get all keys
    pub async fn keys(&self) -> MapletResult<Vec<String>> {
        let result = self.storage.keys().await?;

        // Increment operation counter
        {
            let mut count = self.operation_count.write().await;
            *count += 1;
        }

        Ok(result)
    }

    /// Clear all data
    pub async fn clear(&self) -> MapletResult<()> {
        // Clear maplet
        {
            let mut maplet_guard = self.maplet.write().await;
            // Note: Maplet doesn't have a clear method, so we create a new one
            *maplet_guard = Maplet::<String, Vec<u8>, ReplaceOperator>::with_config(self.config.maplet.clone())?;
        }

        // Clear storage
        self.storage.clear_database().await?;

        // Reset operation counter
        {
            let mut count = self.operation_count.write().await;
            *count = 0;
        }

        Ok(())
    }

    /// Flush data to persistent storage
    pub async fn flush(&self) -> MapletResult<()> {
        self.storage.flush().await?;
        Ok(())
    }

    /// Close the engine and cleanup resources
    pub async fn close(&self) -> MapletResult<()> {
        // Stop TTL cleanup task
        self.ttl_manager.stop_cleanup().await?;
        
        // Close storage
        self.storage.close().await?;
        Ok(())
    }

    /// Get engine statistics
    pub async fn stats(&self) -> MapletResult<EngineStats> {
        let maplet_guard = self.maplet.read().await;
        let maplet_stats = maplet_guard.stats().await;
        drop(maplet_guard);

        let storage_stats = self.storage.stats().await?;
        let ttl_stats = self.ttl_manager.get_stats().await?;
        let operation_count = *self.operation_count.read().await;
        
        let uptime = self.start_time.elapsed()
            .unwrap_or_default()
            .as_secs();

        Ok(EngineStats {
            maplet_stats,
            storage_stats,
            ttl_stats,
            uptime_seconds: uptime,
            total_operations: operation_count,
        })
    }

    /// Get memory usage in bytes
    pub async fn memory_usage(&self) -> MapletResult<u64> {
        let storage_stats = self.storage.stats().await?;
        Ok(storage_stats.memory_usage)
    }

    /// Get the persistence mode
    pub fn persistence_mode(&self) -> PersistenceMode {
        self.config.persistence_mode.clone()
    }

    /// Get the engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Set TTL for a key
    pub async fn expire(&self, key: &str, ttl_seconds: u64) -> MapletResult<bool> {
        // Check if key exists
        if !self.exists(key).await? {
            return Ok(false);
        }

        // Set TTL
        self.ttl_manager.set_ttl(key.to_string(), 0, ttl_seconds).await?;
        
        // Increment operation counter
        {
            let mut count = self.operation_count.write().await;
            *count += 1;
        }

        Ok(true)
    }

    /// Get TTL for a key in seconds
    pub async fn ttl(&self, key: &str) -> MapletResult<Option<i64>> {
        let result = self.ttl_manager.get_ttl(key).await?;
        
        // Increment operation counter
        {
            let mut count = self.operation_count.write().await;
            *count += 1;
        }

        Ok(result)
    }

    /// Remove TTL for a key
    pub async fn persist(&self, key: &str) -> MapletResult<bool> {
        let had_ttl = self.ttl_manager.get_ttl(key).await?.is_some();
        self.ttl_manager.remove_ttl(key).await?;
        
        // Increment operation counter
        {
            let mut count = self.operation_count.write().await;
            *count += 1;
        }

        Ok(had_ttl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_engine_creation() {
        let config = EngineConfig::default();
        let engine = Engine::new(config).await.unwrap();
        assert_eq!(engine.persistence_mode(), PersistenceMode::Memory);
    }

    #[tokio::test]
    async fn test_engine_basic_operations() {
        let config = EngineConfig::default();
        let engine = Engine::new(config).await.unwrap();

        // Test set and get
        engine.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        let value = engine.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Test exists
        assert!(engine.exists("key1").await.unwrap());
        assert!(!engine.exists("nonexistent").await.unwrap());

        // Test delete
        let deleted = engine.delete("key1").await.unwrap();
        assert!(deleted);
        assert!(!engine.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_engine_with_disk_storage() {
        let temp_dir = TempDir::new().unwrap();
        let config = EngineConfig {
            persistence_mode: PersistenceMode::Disk,
            data_dir: Some(temp_dir.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        
        let engine = Engine::new(config).await.unwrap();
        
        // Test operations
        engine.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        let value = engine.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_engine_stats() {
        let config = EngineConfig::default();
        let engine = Engine::new(config).await.unwrap();

        // Perform some operations
        engine.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        engine.get("key1").await.unwrap();

        let stats = engine.stats().await.unwrap();
        assert!(stats.total_operations > 0);
        assert!(stats.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn test_engine_ttl_operations() {
        let config = EngineConfig::default();
        let engine = Engine::new(config).await.unwrap();

        // Set a key
        engine.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        
        // Set TTL
        let result = engine.expire("key1", 60).await.unwrap();
        assert!(result);

        // Check TTL
        let ttl = engine.ttl("key1").await.unwrap();
        assert!(ttl.is_some());
        assert!(ttl.unwrap() <= 60);

        // Remove TTL
        let had_ttl = engine.persist("key1").await.unwrap();
        assert!(had_ttl);

        // Check TTL is gone
        let ttl = engine.ttl("key1").await.unwrap();
        assert!(ttl.is_none());
    }

    #[tokio::test]
    async fn test_engine_ttl_expiration() {
        let config = EngineConfig::default();
        let engine = Engine::new(config).await.unwrap();

        // Set a key with very short TTL
        engine.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        engine.expire("key1", 1).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;
        
        // Key should be expired
        let value = engine.get("key1").await.unwrap();
        assert!(value.is_none());
    }
}
