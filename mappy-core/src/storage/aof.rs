//! Append-Only File (AOF) storage backend
//! 
//! Provides durability through append-only logging with periodic sync.

use super::{Storage, StorageStats, StorageConfig};
use crate::{MapletError, MapletResult};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::{BufReader, Write, BufRead};
use serde_json;
use async_trait::async_trait;
use tokio::time::{interval, Duration};

/// AOF storage backend
pub struct AOFStorage {
    /// In-memory cache
    cache: Arc<DashMap<String, Vec<u8>>>,
    /// AOF file path
    aof_path: std::path::PathBuf,
    /// Configuration
    config: StorageConfig,
    /// Statistics
    stats: Arc<RwLock<StorageStats>>,
    /// Start time for latency calculation
    #[allow(dead_code)]
    start_time: Instant,
    /// Background sync task handle
    sync_handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum AOFEntry {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

impl AOFStorage {
    /// Create a new AOF storage
    pub fn new(config: StorageConfig) -> MapletResult<Self> {
        // Ensure data directory exists
        std::fs::create_dir_all(&config.data_dir)
            .map_err(|e| MapletError::Internal(format!("Failed to create data directory: {e}")))?;
        
        let aof_path = Path::new(&config.data_dir).join("mappy.aof");
        
        let mut storage = Self {
            cache: Arc::new(DashMap::new()),
            aof_path: aof_path,
            config,
            stats: Arc::new(RwLock::new(StorageStats::default())),
            start_time: Instant::now(),
            sync_handle: None,
        };
        
        // Load existing data from AOF file
        storage.load_from_aof()?;
        
        // Start background sync task
        storage.start_sync_task();
        
        Ok(storage)
    }
    
    /// Load data from AOF file
    fn load_from_aof(&self) -> MapletResult<()> {
        if !self.aof_path.exists() {
            return Ok(());
        }
        
        let file = std::fs::File::open(&self.aof_path)
            .map_err(|e| MapletError::Internal(format!("Failed to open AOF file: {e}")))?;
        
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line.map_err(|e| MapletError::Internal(format!("Failed to read AOF line: {e}")))?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: AOFEntry = serde_json::from_str(&line)
                .map_err(|e| MapletError::Internal(format!("Failed to parse AOF entry: {e}")))?;
            
            match entry {
                AOFEntry::Set { key, value } => {
                    self.cache.insert(key, value);
                }
                AOFEntry::Delete { key } => {
                    self.cache.remove(&key);
                }
            }
        }
        
        Ok(())
    }
    
    /// Append entry to AOF file
    fn append_to_aof(&self, entry: AOFEntry) -> MapletResult<()> {
        let line = serde_json::to_string(&entry)
            .map_err(|e| MapletError::Internal(format!("Failed to serialize AOF entry: {e}")))?;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.aof_path)
            .map_err(|e| MapletError::Internal(format!("Failed to open AOF file for append: {e}")))?;
        
        writeln!(file, "{line}")
            .map_err(|e| MapletError::Internal(format!("Failed to write to AOF file: {e}")))?;
        
        Ok(())
    }
    
    /// Start background sync task
    fn start_sync_task(&mut self) {
        let _cache = Arc::clone(&self.cache);
        let aof_path = self.aof_path.clone();
        let sync_interval = Duration::from_secs(self.config.sync_interval);
        
        let handle = tokio::spawn(async move {
            let mut interval = interval(sync_interval);
            
            loop {
                interval.tick().await;
                
                // Force sync to disk
                if let Err(e) = std::fs::File::open(&aof_path)
                    .and_then(|f| f.sync_all())
                {
                    eprintln!("Failed to sync AOF file: {e}");
                }
            }
        });
        
        self.sync_handle = Some(handle);
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
        for entry in self.cache.iter() {
            total += entry.key().len() + entry.value().len();
        }
        total as u64
    }
    
    /// Calculate disk usage
    fn calculate_disk_usage(&self) -> u64 {
        if self.aof_path.exists() {
            std::fs::metadata(&self.aof_path)
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        }
    }
}

#[async_trait]
impl Storage for AOFStorage {
    async fn get(&self, key: &str) -> MapletResult<Option<Vec<u8>>> {
        let start = Instant::now();
        let result = self.cache.get(key).map(|entry| entry.value().clone());
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        }).await;
        
        Ok(result)
    }
    
    async fn set(&self, key: String, value: Vec<u8>) -> MapletResult<()> {
        let start = Instant::now();
        
        // Update cache
        self.cache.insert(key.clone(), value.clone());
        
        // Append to AOF
        let entry = AOFEntry::Set { key, value };
        self.append_to_aof(entry)?;
        
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = self.cache.len() as u64;
            stats.memory_usage = self.calculate_memory_usage();
            stats.disk_usage = self.calculate_disk_usage();
        }).await;
        
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> MapletResult<bool> {
        let start = Instant::now();
        let existed = self.cache.remove(key).is_some();
        
        if existed {
            // Append delete to AOF
            let entry = AOFEntry::Delete { key: key.to_string() };
            self.append_to_aof(entry)?;
        }
        
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = self.cache.len() as u64;
            stats.memory_usage = self.calculate_memory_usage();
            stats.disk_usage = self.calculate_disk_usage();
        }).await;
        
        Ok(existed)
    }
    
    async fn exists(&self, key: &str) -> MapletResult<bool> {
        let start = Instant::now();
        let exists = self.cache.contains_key(key);
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        }).await;
        
        Ok(exists)
    }
    
    async fn keys(&self) -> MapletResult<Vec<String>> {
        let start = Instant::now();
        let keys: Vec<String> = self.cache.iter().map(|entry| entry.key().clone()).collect();
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        }).await;
        
        Ok(keys)
    }
    
    async fn clear_database(&self) -> MapletResult<()> {
        let start = Instant::now();
        self.cache.clear();
        
        // Clear AOF file
        std::fs::write(&self.aof_path, "")
            .map_err(|e| MapletError::Internal(format!("Failed to clear AOF file: {e}")))?;
        
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = 0;
            stats.memory_usage = 0;
            stats.disk_usage = 0;
        }).await;
        
        Ok(())
    }
    
    async fn flush(&self) -> MapletResult<()> {
        let start = Instant::now();
        
        // Force sync AOF file to disk
        std::fs::File::open(&self.aof_path)
            .and_then(|f| f.sync_all())
            .map_err(|e| MapletError::Internal(format!("Failed to sync AOF file: {e}")))?;
        
        let latency = start.elapsed().as_micros() as u64;
        
        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        }).await;
        
        Ok(())
    }
    
    async fn close(&self) -> MapletResult<()> {
        // Cancel background sync task
        if let Some(handle) = &self.sync_handle {
            handle.abort();
        }
        
        // Final sync
        self.flush().await?;
        
        Ok(())
    }
    
    async fn stats(&self) -> MapletResult<StorageStats> {
        let mut stats = self.stats.read().await.clone();
        stats.total_keys = self.cache.len() as u64;
        stats.memory_usage = self.calculate_memory_usage();
        stats.disk_usage = self.calculate_disk_usage();
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_aof_storage_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();
        
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
    async fn test_aof_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        
        // Create storage and write data
        {
            let storage = AOFStorage::new(config.clone()).unwrap();
            storage.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
            storage.flush().await.unwrap();
        }
        
        // Reopen storage and verify data persists
        {
            let storage = AOFStorage::new(config).unwrap();
            let value = storage.get("key1").await.unwrap();
            assert_eq!(value, Some(b"value1".to_vec()));
        }
    }
    
    #[tokio::test]
    async fn test_aof_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();
        
        storage.set("key1".to_string(), b"value1".to_vec()).await.unwrap();
        
        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.total_keys, 1);
        assert!(stats.memory_usage > 0);
        assert!(stats.disk_usage > 0);
        assert!(stats.operations_count > 0);
    }
}
