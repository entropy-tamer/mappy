//! TTL (Time-To-Live) management for mappy
//! 
//! Provides expiration tracking and automatic cleanup of expired keys.

use crate::MapletResult;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// TTL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTLConfig {
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Maximum number of keys to clean up per batch
    pub max_cleanup_batch_size: usize,
    /// Enable background cleanup task
    pub enable_background_cleanup: bool,
}

impl Default for TTLConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_secs: 60, // 1 minute
            max_cleanup_batch_size: 1000,
            enable_background_cleanup: true,
        }
    }
}

/// TTL entry tracking expiration time for a key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTLEntry {
    /// The key
    pub key: String,
    /// Expiration timestamp (Unix timestamp in seconds)
    pub expires_at: u64,
    /// Database ID
    pub db_id: u8,
}

impl TTLEntry {
    /// Create a new TTL entry
    pub fn new(key: String, db_id: u8, ttl_seconds: u64) -> Self {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() + ttl_seconds;
        
        Self {
            key,
            db_id,
            expires_at,
        }
    }

    /// Check if this entry has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= self.expires_at
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expires_at as i64 - now as i64
    }
}

/// TTL manager for tracking and cleaning up expired keys
pub struct TTLManager {
    /// TTL configuration
    config: TTLConfig,
    /// Map of expiration time -> list of keys that expire at that time
    expiration_map: Arc<RwLock<BTreeMap<u64, Vec<TTLEntry>>>>,
    /// Map of key -> expiration time for fast lookups
    key_to_expiration: Arc<RwLock<std::collections::HashMap<String, u64>>>,
    /// Background cleanup task handle
    cleanup_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Shutdown signal
    shutdown_tx: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl TTLManager {
    /// Create a new TTL manager
    pub fn new(config: TTLConfig) -> Self {
        Self {
            config,
            expiration_map: Arc::new(RwLock::new(BTreeMap::new())),
            key_to_expiration: Arc::new(RwLock::new(std::collections::HashMap::new())),
            cleanup_handle: Arc::new(RwLock::new(None)),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Set TTL for a key
    pub async fn set_ttl(&self, key: String, db_id: u8, ttl_seconds: u64) -> MapletResult<()> {
        let entry = TTLEntry::new(key.clone(), db_id, ttl_seconds);
        let expires_at = entry.expires_at;

        // Remove from old expiration time if it exists
        self.remove_ttl(&key).await?;

        // Add to new expiration time
        {
            let mut expiration_map = self.expiration_map.write().await;
            expiration_map.entry(expires_at)
                .or_insert_with(Vec::new)
                .push(entry);
        }

        // Update key lookup map
        {
            let mut key_map = self.key_to_expiration.write().await;
            key_map.insert(key, expires_at);
        }

        Ok(())
    }

    /// Get TTL for a key in seconds
    pub async fn get_ttl(&self, key: &str) -> MapletResult<Option<i64>> {
        let key_map = self.key_to_expiration.read().await;
        if let Some(&expires_at) = key_map.get(key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let remaining = expires_at as i64 - now as i64;
            Ok(Some(remaining.max(0)))
        } else {
            Ok(None)
        }
    }

    /// Remove TTL for a key
    pub async fn remove_ttl(&self, key: &str) -> MapletResult<()> {
        let mut key_map = self.key_to_expiration.write().await;
        if let Some(expires_at) = key_map.remove(key) {
            drop(key_map);

            let mut expiration_map = self.expiration_map.write().await;
            if let Some(entries) = expiration_map.get_mut(&expires_at) {
                entries.retain(|entry| entry.key != key);
                if entries.is_empty() {
                    expiration_map.remove(&expires_at);
                }
            }
        }

        Ok(())
    }

    /// Check if a key has expired
    pub async fn is_expired(&self, key: &str) -> MapletResult<bool> {
        let key_map = self.key_to_expiration.read().await;
        if let Some(&expires_at) = key_map.get(key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(now >= expires_at)
        } else {
            Ok(false)
        }
    }

    /// Get all expired keys
    pub async fn get_expired_keys(&self) -> MapletResult<Vec<TTLEntry>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut expired_entries = Vec::new();
        let mut expiration_map = self.expiration_map.write().await;
        
        // Get all entries that have expired
        let expired_times: Vec<u64> = expiration_map
            .range(..=now)
            .map(|(&time, _)| time)
            .collect();

        for time in expired_times {
            if let Some(entries) = expiration_map.remove(&time) {
                expired_entries.extend(entries);
            }
        }

        // Update key lookup map
        let mut key_map = self.key_to_expiration.write().await;
        for entry in &expired_entries {
            key_map.remove(&entry.key);
        }

        Ok(expired_entries)
    }

    /// Start background cleanup task
    pub async fn start_cleanup<F>(&self, mut cleanup_callback: F) -> MapletResult<()>
    where
        F: FnMut(Vec<TTLEntry>) -> MapletResult<()> + Send + Sync + 'static,
    {
        if !self.config.enable_background_cleanup {
            return Ok(());
        }

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        let expiration_map = Arc::clone(&self.expiration_map);
        let key_to_expiration = Arc::clone(&self.key_to_expiration);
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.cleanup_interval_secs));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Perform cleanup
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        let mut expired_entries = Vec::new();
                        {
                            let mut expiration_map = expiration_map.write().await;
                            let expired_times: Vec<u64> = expiration_map
                                .range(..=now)
                                .take(config.max_cleanup_batch_size)
                                .map(|(&time, _)| time)
                                .collect();

                            for time in expired_times {
                                if let Some(entries) = expiration_map.remove(&time) {
                                    expired_entries.extend(entries);
                                }
                            }
                        }

                        if !expired_entries.is_empty() {
                            // Update key lookup map
                            {
                                let mut key_map = key_to_expiration.write().await;
                                for entry in &expired_entries {
                                    key_map.remove(&entry.key);
                                }
                            }

                            // Call cleanup callback
                            if let Err(e) = cleanup_callback(expired_entries) {
                                eprintln!("TTL cleanup callback error: {}", e);
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        // Store cleanup handle and shutdown sender
        {
            let mut cleanup_handle = self.cleanup_handle.write().await;
            *cleanup_handle = Some(handle);
        }
        {
            let mut shutdown_tx_guard = self.shutdown_tx.write().await;
            *shutdown_tx_guard = Some(shutdown_tx);
        }

        Ok(())
    }

    /// Stop background cleanup task
    pub async fn stop_cleanup(&self) -> MapletResult<()> {
        // Send shutdown signal
        {
            let mut shutdown_tx = self.shutdown_tx.write().await;
            if let Some(tx) = shutdown_tx.take() {
                let _ = tx.send(());
            }
        }

        // Wait for cleanup task to finish
        {
            let mut cleanup_handle = self.cleanup_handle.write().await;
            if let Some(handle) = cleanup_handle.take() {
                let _ = handle.await;
            }
        }

        Ok(())
    }

    /// Get TTL statistics
    pub async fn get_stats(&self) -> MapletResult<TTLStats> {
        let expiration_map = self.expiration_map.read().await;
        let key_map = self.key_to_expiration.read().await;

        let total_keys = key_map.len();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let expired_count: usize = expiration_map
            .range(..=now)
            .map(|(_, entries)| entries.len())
            .sum();

        Ok(TTLStats {
            total_keys_with_ttl: total_keys as u64,
            expired_keys: expired_count as u64,
            next_expiration: expiration_map
                .range(now..)
                .next()
                .map(|(&time, _)| time),
        })
    }

    /// Clear all TTL entries
    pub async fn clear_all(&self) -> MapletResult<()> {
        {
            let mut expiration_map = self.expiration_map.write().await;
            expiration_map.clear();
        }
        {
            let mut key_map = self.key_to_expiration.write().await;
            key_map.clear();
        }
        Ok(())
    }
}

/// TTL statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTLStats {
    /// Total number of keys with TTL set
    pub total_keys_with_ttl: u64,
    /// Number of expired keys waiting for cleanup
    pub expired_keys: u64,
    /// Next expiration timestamp (if any)
    pub next_expiration: Option<u64>,
}

impl Default for TTLStats {
    fn default() -> Self {
        Self {
            total_keys_with_ttl: 0,
            expired_keys: 0,
            next_expiration: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ttl_manager_basic_operations() {
        let config = TTLConfig::default();
        let manager = TTLManager::new(config);

        // Set TTL for a key
        manager.set_ttl("key1".to_string(), 0, 60).await.unwrap();
        
        // Check TTL
        let ttl = manager.get_ttl("key1").await.unwrap();
        assert!(ttl.is_some());
        assert!(ttl.unwrap() <= 60);

        // Check if not expired
        assert!(!manager.is_expired("key1").await.unwrap());

        // Remove TTL
        manager.remove_ttl("key1").await.unwrap();
        assert!(manager.get_ttl("key1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let config = TTLConfig::default();
        let manager = TTLManager::new(config);

        // Set very short TTL
        manager.set_ttl("key1".to_string(), 0, 1).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(1100)).await;
        
        // Check if expired
        assert!(manager.is_expired("key1").await.unwrap());
        
        // Get expired keys
        let expired = manager.get_expired_keys().await.unwrap();
        assert!(!expired.is_empty());
        assert_eq!(expired[0].key, "key1");
    }

    #[tokio::test]
    async fn test_ttl_stats() {
        let config = TTLConfig::default();
        let manager = TTLManager::new(config);

        // Set some TTLs
        manager.set_ttl("key1".to_string(), 0, 60).await.unwrap();
        manager.set_ttl("key2".to_string(), 0, 120).await.unwrap();

        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_keys_with_ttl, 2);
        assert_eq!(stats.expired_keys, 0);
        assert!(stats.next_expiration.is_some());
    }

    #[tokio::test]
    async fn test_ttl_clear_all() {
        let config = TTLConfig::default();
        let manager = TTLManager::new(config);

        // Set some TTLs
        manager.set_ttl("key1".to_string(), 0, 60).await.unwrap();
        manager.set_ttl("key2".to_string(), 0, 120).await.unwrap();

        // Clear all
        manager.clear_all().await.unwrap();

        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_keys_with_ttl, 0);
    }
}
