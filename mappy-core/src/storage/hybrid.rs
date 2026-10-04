//! Hybrid storage backend
//!
//! Combines in-memory cache with AOF logging for optimal performance and durability.
use super::{Storage, StorageConfig, StorageStats};
use crate::{MapletError, MapletResult};
use async_lock::RwLock;
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

/// Hybrid storage backend combining memory and AOF
pub struct HybridStorage {
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
    sync_handle: Option<std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>>>,
    /// Write buffer for batching AOF writes
    write_buffer: Arc<RwLock<Vec<AOFEntry>>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
enum AOFEntry {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

impl HybridStorage {
    /// Create a new hybrid storage
    ///
    /// # Errors
    /// Returns an error if the data directory cannot be created or if loading from AOF fails.
    pub fn new(config: StorageConfig) -> MapletResult<Self> {
        // Ensure data directory exists
        std::fs::create_dir_all(&config.data_dir)
            .map_err(|e| MapletError::Internal(format!("Failed to create data directory: {e}")))?;

        let aof_path = Path::new(&config.data_dir).join("mappy.aof");

        let mut storage = Self {
            cache: Arc::new(DashMap::new()),
            aof_path,
            config,
            stats: Arc::new(RwLock::new(StorageStats::default())),
            start_time: Instant::now(),
            sync_handle: None,
            write_buffer: Arc::new(RwLock::new(Vec::new())),
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
            let line =
                line.map_err(|e| MapletError::Internal(format!("Failed to read AOF line: {e}")))?;
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

    /// Add entry to write buffer
    async fn add_to_buffer(&self, entry: AOFEntry) -> MapletResult<()> {
        {
            let mut buffer = self.write_buffer.write().await;
            buffer.push(entry);

            // Flush buffer if it's full
            if buffer.len() >= self.config.write_buffer_size {
                self.flush_buffer_internal(&mut buffer)?;
            }
        }

        Ok(())
    }

    /// Flush write buffer to AOF file
    fn flush_buffer_internal(&self, buffer: &mut Vec<AOFEntry>) -> MapletResult<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.aof_path)
            .map_err(|e| {
                MapletError::Internal(format!("Failed to open AOF file for append: {e}"))
            })?;

        for entry in buffer.iter() {
            let line = serde_json::to_string(entry).map_err(|e| {
                MapletError::Internal(format!("Failed to serialize AOF entry: {e}"))
            })?;
            writeln!(file, "{line}")
                .map_err(|e| MapletError::Internal(format!("Failed to write to AOF file: {e}")))?;
        }

        buffer.clear();
        Ok(())
    }

    /// Start background sync task
    fn start_sync_task(&mut self) {
        let write_buffer = Arc::clone(&self.write_buffer);
        let aof_path = self.aof_path.clone();
        let sync_interval = Duration::from_secs(self.config.sync_interval);

        // Tokio takes precedence when both features are enabled (workspace
        // feature unification can enable both simultaneously).
        #[cfg(all(feature = "smol", not(feature = "tokio")))]
        let handle: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>> =
            Box::pin(smol::spawn(async move {
                loop {
                    smol::Timer::after(sync_interval).await;
                    {
                        let mut buffer = write_buffer.write().await;
                        if !buffer.is_empty()
                            && let Err(e) =
                                Self::flush_buffer_internal_static(&mut buffer, &aof_path)
                        {
                            eprintln!("Failed to flush AOF buffer: {e}");
                        }
                    }
                    if aof_path.exists() {
                        if let Err(e) = std::fs::File::open(&aof_path).and_then(|f| f.sync_all()) {
                            eprintln!("Failed to sync AOF file: {e}");
                        }
                    }
                }
            }));

        #[cfg(feature = "tokio")]
        let handle: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>> =
            Box::pin(async move {
                let _ = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(sync_interval);
                    loop {
                        interval.tick().await;
                        {
                            let mut buffer = write_buffer.write().await;
                            if !buffer.is_empty()
                                && let Err(e) =
                                    Self::flush_buffer_internal_static(&mut buffer, &aof_path)
                            {
                                eprintln!("Failed to flush AOF buffer: {e}");
                            }
                        }
                        if aof_path.exists() {
                            if let Err(e) =
                                std::fs::File::open(&aof_path).and_then(|f| f.sync_all())
                            {
                                eprintln!("Failed to sync AOF file: {e}");
                            }
                        }
                    }
                })
                .await;
            });

        // Neither runtime feature enabled: no executor to spawn the sync task onto.
        // The handle is still produced so the assignment below stays unconditional;
        // the future never runs.
        #[cfg(not(any(feature = "smol", feature = "tokio")))]
        let handle: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>> =
            Box::pin(async {});

        self.sync_handle = Some(handle);
    }

    /// Static version of `flush_buffer_internal` for use in background task
    fn flush_buffer_internal_static(
        buffer: &mut Vec<AOFEntry>,
        aof_path: &std::path::Path,
    ) -> MapletResult<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(aof_path)
            .map_err(|e| {
                MapletError::Internal(format!("Failed to open AOF file for append: {e}"))
            })?;

        for entry in buffer.iter() {
            let line = serde_json::to_string(entry).map_err(|e| {
                MapletError::Internal(format!("Failed to serialize AOF entry: {e}"))
            })?;
            writeln!(file, "{line}")
                .map_err(|e| MapletError::Internal(format!("Failed to write to AOF file: {e}")))?;
        }

        buffer.clear();
        Ok(())
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
impl Storage for HybridStorage {
    async fn get(&self, key: &str) -> MapletResult<Option<Vec<u8>>> {
        let start = Instant::now();
        let result = self.cache.get(key).map(|entry| entry.value().clone());
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        })
        .await;

        Ok(result)
    }

    async fn set(&self, key: String, value: Vec<u8>) -> MapletResult<()> {
        let start = Instant::now();

        // Update cache immediately
        self.cache.insert(key.clone(), value.clone());

        // Add to write buffer for AOF
        let entry = AOFEntry::Set { key, value };
        self.add_to_buffer(entry).await?;

        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = self.cache.len() as u64;
            stats.memory_usage = self.calculate_memory_usage();
            stats.disk_usage = self.calculate_disk_usage();
        })
        .await;

        Ok(())
    }

    async fn delete(&self, key: &str) -> MapletResult<bool> {
        let start = Instant::now();
        let existed = self.cache.remove(key).is_some();

        if existed {
            // Add delete to write buffer
            let entry = AOFEntry::Delete {
                key: key.to_string(),
            };
            self.add_to_buffer(entry).await?;
        }

        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = self.cache.len() as u64;
            stats.memory_usage = self.calculate_memory_usage();
            stats.disk_usage = self.calculate_disk_usage();
        })
        .await;

        Ok(existed)
    }

    async fn exists(&self, key: &str) -> MapletResult<bool> {
        let start = Instant::now();
        let exists = self.cache.contains_key(key);
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        })
        .await;

        Ok(exists)
    }

    async fn keys(&self) -> MapletResult<Vec<String>> {
        let start = Instant::now();
        let keys: Vec<String> = self.cache.iter().map(|entry| entry.key().clone()).collect();
        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        })
        .await;

        Ok(keys)
    }

    async fn clear_database(&self) -> MapletResult<()> {
        let start = Instant::now();
        self.cache.clear();

        // Clear write buffer
        {
            let mut buffer = self.write_buffer.write().await;
            buffer.clear();
        }

        // Clear AOF file
        std::fs::write(&self.aof_path, "")
            .map_err(|e| MapletError::Internal(format!("Failed to clear AOF file: {e}")))?;

        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
            stats.total_keys = 0;
            stats.memory_usage = 0;
            stats.disk_usage = 0;
        })
        .await;

        Ok(())
    }

    async fn flush(&self) -> MapletResult<()> {
        let start = Instant::now();

        // Flush write buffer
        {
            let mut buffer = self.write_buffer.write().await;
            self.flush_buffer_internal(&mut buffer)?;
        }

        // Force sync AOF file to disk
        std::fs::File::open(&self.aof_path)
            .and_then(|f| f.sync_all())
            .map_err(|e| MapletError::Internal(format!("Failed to sync AOF file: {e}")))?;

        let latency = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        })
        .await;

        Ok(())
    }

    async fn close(&self) -> MapletResult<()> {
        // Cancel background sync task
        if let Some(_handle) = &self.sync_handle {
            // Task is cancelled when the storage is dropped.
        }

        // Final flush
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
    async fn test_hybrid_storage_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        // Test set and get
        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();
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
    async fn test_hybrid_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };

        // Create storage and write data
        {
            let storage = HybridStorage::new(config.clone()).unwrap();
            storage
                .set("key1".to_string(), b"value1".to_vec())
                .await
                .unwrap();
            storage.flush().await.unwrap();
        }

        // Reopen storage and verify data persists
        {
            let storage = HybridStorage::new(config).unwrap();
            let value = storage.get("key1").await.unwrap();
            assert_eq!(value, Some(b"value1".to_vec()));
        }
    }

    #[tokio::test]
    async fn test_hybrid_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        // Flush to ensure data is written to disk
        storage.flush().await.unwrap();

        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.total_keys, 1);
        assert!(stats.memory_usage > 0);
        assert!(stats.disk_usage > 0);
        assert!(stats.operations_count > 0);
    }

    #[tokio::test]
    async fn test_hybrid_storage_clear() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        storage.clear_database().await.unwrap();

        assert_eq!(storage.get("key1").await.unwrap(), None);
        assert_eq!(storage.keys().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_hybrid_storage_keys() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();
        storage
            .set("key2".to_string(), b"value2".to_vec())
            .await
            .unwrap();

        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_hybrid_storage_flush() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        assert!(storage.flush().await.is_ok());
    }

    #[tokio::test]
    async fn test_hybrid_storage_close() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        assert!(storage.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_hybrid_storage_delete_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        let deleted = storage.delete("nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_hybrid_storage_empty_keys() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[test]
    fn test_aof_entry_serialization() {
        let entry = AOFEntry::Set {
            key: "test".to_string(),
            value: vec![1, 2, 3],
        };

        let serialized = serde_json::to_string(&entry).expect("Failed to serialize");
        let deserialized: AOFEntry =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        match deserialized {
            AOFEntry::Set { key, value } => {
                assert_eq!(key, "test");
                assert_eq!(value, vec![1, 2, 3]);
            }
            _ => panic!("Expected Set entry"),
        }
    }

    #[test]
    fn test_aof_entry_delete_serialization() {
        let entry = AOFEntry::Delete {
            key: "test".to_string(),
        };

        let serialized = serde_json::to_string(&entry).expect("Failed to serialize");
        let deserialized: AOFEntry =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        match deserialized {
            AOFEntry::Delete { key } => {
                assert_eq!(key, "test");
            }
            _ => panic!("Expected Delete entry"),
        }
    }

    #[tokio::test]
    async fn test_hybrid_storage_buffer_flush() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 2,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();
        storage
            .set("key2".to_string(), b"value2".to_vec())
            .await
            .unwrap();

        storage.flush().await.unwrap();

        let value = storage.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_hybrid_storage_large_value() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        let large_value = vec![0u8; 10000];
        storage
            .set("key1".to_string(), large_value.clone())
            .await
            .unwrap();

        let value = storage.get("key1").await.unwrap();
        assert_eq!(value, Some(large_value));
    }

    #[tokio::test]
    async fn test_hybrid_storage_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();
        storage
            .set("key1".to_string(), b"value2".to_vec())
            .await
            .unwrap();

        let value = storage.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value2".to_vec()));
    }

    #[tokio::test]
    async fn test_hybrid_storage_multiple_deletes() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            write_buffer_size: 10,
            ..Default::default()
        };
        let storage = HybridStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();
        storage
            .set("key2".to_string(), b"value2".to_vec())
            .await
            .unwrap();

        storage.delete("key1").await.unwrap();
        storage.delete("key2").await.unwrap();

        assert_eq!(storage.keys().await.unwrap().len(), 0);
    }
}
