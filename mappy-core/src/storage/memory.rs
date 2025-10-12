//! In-memory storage backend
//! 
//! Uses DashMap for high-performance concurrent access.

use super::{Storage, StorageStats, StorageConfig};
use crate::{MapletError, MapletResult};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;
use async_trait::async_trait;

/// In-memory storage using DashMap
pub struct MemoryStorage {
    /// The actual storage
    data: Arc<DashMap<String, Vec<u8>>>,
    /// Configuration
    config: StorageConfig,
    /// Statistics
    stats: Arc<RwLock<StorageStats>>,
    /// Start time for latency calculation
    start_time: Instant,
}

impl MemoryStorage {
    /// Create a new memory storage
    pub async fn new(config: StorageConfig) -> MapletResult<Self> {
        Ok(Self {
            data: Arc::new(DashMap::new()),
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
    
    /// Calculate memory usage
    fn calculate_memory_usage(&self) -> u64 {
        let mut total = 0;
        for entry in self.data.iter() {
            total += entry.key().len() + entry.value().len();
        }
        total as u64
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> MapletResult<Option<Vec<u8>>> {
        let start = Instant::now();
        let result = self.data.get(key).map(|entry| entry.value().clone());
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = (stats.avg_latency_us + latency) / 2;
        }).await;
        
        Ok(result)
    }
    
    async fn set(&self, key: String, value: Vec<u8>) -> MapletResult<()> {
        let start = Instant::now();
        
        // Check memory limits
        if let Some(max_memory) = self.config.max_memory {
            let current_usage = self.calculate_memory_usage();
            if current_usage + key.len() as u64 + value.len() as u64 > max_memory {
                return Err(MapletError::Internal("Memory limit exceeded".to_string()));
            }
        }
        
        self.data.insert(key.clone(), value);
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = (stats.avg_latency_us + latency) / 2;
            stats.total_keys = self.data.len() as u64;
            stats.memory_usage = self.calculate_memory_usage();
        }).await;
        
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> MapletResult<bool> {
        let start = Instant::now();
        let existed = self.data.remove(key).is_some();
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = (stats.avg_latency_us + latency) / 2;
            stats.total_keys = self.data.len() as u64;
            stats.memory_usage = self.calculate_memory_usage();
        }).await;
        
        Ok(existed)
    }
    
    async fn exists(&self, key: &str) -> MapletResult<bool> {
        let start = Instant::now();
        let exists = self.data.contains_key(key);
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = (stats.avg_latency_us + latency) / 2;
        }).await;
        
        Ok(exists)
    }
    
    async fn keys(&self) -> MapletResult<Vec<String>> {
        let start = Instant::now();
        let keys: Vec<String> = self.data.iter().map(|entry| entry.key().clone()).collect();
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = (stats.avg_latency_us + latency) / 2;
        }).await;
        
        Ok(keys)
    }
    
    async fn clear_database(&self) -> MapletResult<()> {
        let start = Instant::now();
        self.data.clear();
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = (stats.avg_latency_us + latency) / 2;
            stats.total_keys = 0;
            stats.memory_usage = 0;
        }).await;
        
        Ok(())
    }
    
    async fn flush(&self) -> MapletResult<()> {
        // Memory storage doesn't need flushing
        Ok(())
    }
    
    async fn close(&self) -> MapletResult<()> {
        // Memory storage doesn't need explicit closing
        Ok(())
    }
    
    async fn stats(&self) -> MapletResult<StorageStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_basic_operations() {
        let config = StorageConfig::default();
        let storage = MemoryStorage::new(config).await.unwrap();
        
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
    async fn test_memory_storage_keys() {
        let config = StorageConfig::default();
        let storage = MemoryStorage::new(config).await.unwrap();
        
        storage.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        storage.set("key2".to_string(), b"value2".to_vec()).await.unwrap();
        
        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }
    
    #[tokio::test]
    async fn test_memory_storage_clear() {
        let config = StorageConfig::default();
        let storage = MemoryStorage::new(config).await.unwrap();
        
        storage.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        storage.set("key2".to_string(), b"value2".to_vec()).await.unwrap();
        
        storage.clear_database().await.unwrap();
        
        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), 0);
    }
    
    #[tokio::test]
    async fn test_memory_storage_stats() {
        let config = StorageConfig::default();
        let storage = MemoryStorage::new(config).await.unwrap();
        
        storage.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        
        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.total_keys, 1);
        assert!(stats.memory_usage > 0);
        assert!(stats.operations_count > 0);
    }
}
