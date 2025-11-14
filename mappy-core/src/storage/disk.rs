//! Disk-based storage backend
//! 
//! Uses Sled for persistent storage with ACID guarantees.

use super::{Storage, StorageStats, StorageConfig};
use crate::{MapletError, MapletResult};
use sled::{Db, Tree};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;
use std::path::Path;
use async_trait::async_trait;

/// Disk-based storage using Sled
pub struct DiskStorage {
    /// The Sled database
    db: Arc<Db>,
    /// Main data tree
    tree: Tree,
    /// Configuration
    #[allow(dead_code)]
    config: StorageConfig,
    /// Statistics
    stats: Arc<RwLock<StorageStats>>,
    /// Start time for latency calculation
    #[allow(dead_code)]
    start_time: Instant,
}

impl DiskStorage {
    /// Create a new disk storage
    pub fn new(config: StorageConfig) -> MapletResult<Self> {
        // Ensure data directory exists
        std::fs::create_dir_all(&config.data_dir)
            .map_err(|e| MapletError::Internal(format!("Failed to create data directory: {e}")))?;
        
        let db_path = Path::new(&config.data_dir).join("mappy.db");
        let db = sled::open(&db_path)
            .map_err(|e| MapletError::Internal(format!("Failed to open database: {e}")))?;
        
        let tree = db.open_tree("data")
            .map_err(|e| MapletError::Internal(format!("Failed to open tree: {e}")))?;
        
        Ok(Self {
            db: Arc::new(db),
            tree,
            config,
            stats: Arc::new(RwLock::new(StorageStats::default())),
            start_time: Instant::now(),
        })
    }
    
    /// Update statistics
    async fn update_stats<F>(&self, f: F) 
    where
        F: FnOnce(&mut StorageStats),
    {
        let mut stats = self.stats.write().await;
        f(&mut stats);
    }
    
    /// Calculate disk usage
    fn calculate_disk_usage(&self) -> u64 {
        self.db.size_on_disk()
            .map_err(|_| MapletError::Internal("Failed to get disk usage".to_string()))
            .unwrap_or(0)
    }
}

#[async_trait]
impl Storage for DiskStorage {
    async fn get(&self, key: &str) -> MapletResult<Option<Vec<u8>>> {
        let start = Instant::now();
        let result = self.tree.get(key)
            .map_err(|e| MapletError::Internal(format!("Failed to get key: {e}")))?
            .map(|ivec| ivec.to_vec());
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        }).await;
        
        Ok(result)
    }
    
    async fn set(&self, key: String, value: Vec<u8>) -> MapletResult<()> {
        let start = Instant::now();
        
        self.tree.insert(&key, value)
            .map_err(|e| MapletError::Internal(format!("Failed to set key: {e}")))?;
        
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = self.tree.len() as u64;
            stats.disk_usage = self.calculate_disk_usage();
        }).await;
        
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> MapletResult<bool> {
        let start = Instant::now();
        let existed = self.tree.remove(key)
            .map_err(|e| MapletError::Internal(format!("Failed to delete key: {e}")))?
            .is_some();
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = self.tree.len() as u64;
            stats.disk_usage = self.calculate_disk_usage();
        }).await;
        
        Ok(existed)
    }
    
    async fn exists(&self, key: &str) -> MapletResult<bool> {
        let start = Instant::now();
        let exists = self.tree.contains_key(key)
            .map_err(|e| MapletError::Internal(format!("Failed to check key existence: {e}")))?;
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        }).await;
        
        Ok(exists)
    }
    
    async fn keys(&self) -> MapletResult<Vec<String>> {
        let start = Instant::now();
        let keys: Result<Vec<String>, _> = self.tree.iter()
            .map(|result| {
                let (key, _) = result.map_err(|e| MapletError::Internal(format!("Failed to iterate: {e}")))?;
                String::from_utf8(key.to_vec())
                    .map_err(|e| MapletError::Internal(format!("Invalid UTF-8 key: {e}")))
            })
            .collect();
        let keys = keys?;
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        }).await;
        
        Ok(keys)
    }
    
    async fn clear_database(&self) -> MapletResult<()> {
        let start = Instant::now();
        self.tree.clear()
            .map_err(|e| MapletError::Internal(format!("Failed to clear database: {e}")))?;
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = 0;
            stats.disk_usage = self.calculate_disk_usage();
        }).await;
        
        Ok(())
    }
    
    async fn flush(&self) -> MapletResult<()> {
        let start = Instant::now();
        self.db.flush()
            .map_err(|e| MapletError::Internal(format!("Failed to flush: {e}")))?;
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        }).await;
        
        Ok(())
    }
    
    async fn close(&self) -> MapletResult<()> {
        self.db.flush()
            .map_err(|e| MapletError::Internal(format!("Failed to flush on close: {e}")))?;
        Ok(())
    }
    
    async fn stats(&self) -> MapletResult<StorageStats> {
        let mut stats = self.stats.read().await.clone();
        stats.total_keys = self.tree.len() as u64;
        stats.disk_usage = self.calculate_disk_usage();
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_disk_storage_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        let storage = DiskStorage::new(config).unwrap();
        
        // Test set and get
        storage.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        let value = storage.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
        
        // Test exists
        assert!(storage.exists("key1").await.unwrap());
        assert!(!storage.exists("key2").await.unwrap());
        
        // Test delete
        let deleted = storage.delete("key1").await.unwrap();
        assert!(deleted);
        assert!(!storage.exists("key1").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_disk_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        
        // Create storage and write data
        {
            let storage = DiskStorage::new(config.clone()).unwrap();
            storage.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
            storage.flush().await.unwrap();
            storage.close().await.unwrap();
        }
        
        // Reopen storage and verify data persists
        {
            let storage = DiskStorage::new(config).unwrap();
            let value = storage.get("key1").await.unwrap();
            assert_eq!(value, Some(b"value1".to_vec()));
            storage.close().await.unwrap();
        }
    }
    
    #[tokio::test]
    async fn test_disk_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        let storage = DiskStorage::new(config).unwrap();
        
        storage.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        
        // Flush to ensure data is written to disk
        storage.flush().await.unwrap();
        
        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.total_keys, 1);
        assert!(stats.disk_usage > 0);
        assert!(stats.operations_count > 0);
    }
}
