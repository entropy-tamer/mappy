//! TTL (Time-To-Live) management for mappy
//!
//! Provides expiration tracking and automatic cleanup of expired keys.
//! Runtime-agnostic: uses `async_lock` for synchronization and feature-gated
//! spawn/timer (`smol` by default, `tokio` opt-in).

use crate::MapletResult;
use async_lock::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    #[must_use]
    pub fn new(key: String, db_id: u8, ttl_seconds: u64) -> Self {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + ttl_seconds;

        Self {
            key,
            expires_at,
            db_id,
        }
    }

    /// Check if this entry has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= self.expires_at
    }

    /// Get remaining TTL in seconds
    #[must_use]
    pub fn remaining_ttl(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        #[allow(clippy::cast_possible_wrap)]
        {
            self.expires_at as i64 - now as i64
        }
    }
}

/// Type-erased background task handle (works with both smol::Task and tokio::JoinHandle).
type CleanupHandle = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>>;

/// TTL manager for tracking and cleaning up expired keys
pub struct TTLManager {
    /// TTL configuration
    config: TTLConfig,
    /// Map of expiration time -> list of keys that expire at that time
    expiration_map: Arc<RwLock<BTreeMap<u64, Vec<TTLEntry>>>>,
    /// Map of key -> expiration time for fast lookups
    key_to_expiration: Arc<RwLock<std::collections::HashMap<String, u64>>>,
    /// Background cleanup task handle
    cleanup_handle: Arc<RwLock<Option<CleanupHandle>>>,
    /// Shutdown signal sender
    shutdown_tx: Arc<RwLock<Option<async_channel::Sender<()>>>>,
}

/// Perform one cleanup batch: collect expired entries, remove them from the maps,
/// and invoke the callback. Runtime-agnostic (uses async_lock::RwLock).
async fn run_cleanup_batch<F>(
    expiration_map: &Arc<RwLock<BTreeMap<u64, Vec<TTLEntry>>>>,
    key_to_expiration: &Arc<RwLock<std::collections::HashMap<String, u64>>>,
    max_batch: usize,
    cleanup_callback: &mut F,
) where
    F: FnMut(Vec<TTLEntry>) -> MapletResult<()>,
{
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut expired_entries = Vec::new();
    {
        let mut map = expiration_map.write().await;
        let expired_times: Vec<u64> = map
            .range(..=now)
            .take(max_batch)
            .map(|(&time, _)| time)
            .collect();
        for time in expired_times {
            if let Some(entries) = map.remove(&time) {
                expired_entries.extend(entries);
            }
        }
    }

    if !expired_entries.is_empty() {
        {
            let mut key_map = key_to_expiration.write().await;
            for entry in &expired_entries {
                key_map.remove(&entry.key);
            }
        }
        if let Err(e) = cleanup_callback(expired_entries) {
            eprintln!("TTL cleanup callback error: {e}");
        }
    }
}

impl TTLManager {
    /// Create a new TTL manager
    #[must_use]
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
    ///
    /// # Errors
    /// Returns an error if removing the old TTL or setting the new TTL fails.
    pub async fn set_ttl(self: Arc<Self>, key: String, db_id: u8, ttl_seconds: u64) -> MapletResult<()>
    where
        Self: Send + Sync,
    {
        let entry = TTLEntry::new(key.clone(), db_id, ttl_seconds);
        let expires_at = entry.expires_at;

        // Remove from old expiration time if it exists
        Arc::clone(&self).remove_ttl(key.clone()).await?;

        // Add to new expiration time
        {
            let mut expiration_map = self.expiration_map.write().await;
            expiration_map
                .entry(expires_at)
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
    ///
    /// # Errors
    /// Returns an error if retrieving TTL fails.
    pub async fn get_ttl(self: Arc<Self>, key: String) -> MapletResult<Option<i64>>
    where
        Self: Send + Sync,
    {
        let key_map = self.key_to_expiration.read().await;
        if let Some(&expires_at) = key_map.get(&key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            #[allow(clippy::cast_possible_wrap)]
            let remaining = expires_at as i64 - now as i64;
            Ok(Some(remaining.max(0)))
        } else {
            Ok(None)
        }
    }

    /// Remove TTL for a key
    ///
    /// # Errors
    /// Returns an error if removing TTL fails.
    pub async fn remove_ttl(self: Arc<Self>, key: String) -> MapletResult<()>
    where
        Self: Send + Sync,
    {
        let mut key_map = self.key_to_expiration.write().await;
        if let Some(expires_at) = key_map.remove(&key) {
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
    ///
    /// # Errors
    /// Returns an error if checking expiration fails.
    pub async fn is_expired(self: Arc<Self>, key: String) -> MapletResult<bool>
    where
        Self: Send + Sync,
    {
        let key_map = self.key_to_expiration.read().await;
        if let Some(&expires_at) = key_map.get(&key) {
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
    ///
    /// # Errors
    /// Returns an error if retrieving expired keys fails.
    pub async fn get_expired_keys(self: Arc<Self>) -> MapletResult<Vec<TTLEntry>>
    where
        Self: Send + Sync,
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut expired_entries = Vec::new();
        let expired_times: Vec<u64> = {
            let expiration_map = self.expiration_map.read().await;
            expiration_map
                .range(..=now)
                .map(|(&time, _)| time)
                .collect()
        };

        {
            let mut expiration_map = self.expiration_map.write().await;
            for time in expired_times {
                if let Some(entries) = expiration_map.remove(&time) {
                    expired_entries.extend(entries);
                }
            }
        }

        // Update key lookup map
        if !expired_entries.is_empty() {
            let mut key_map = self.key_to_expiration.write().await;
            for entry in &expired_entries {
                key_map.remove(&entry.key);
            }
        }

        Ok(expired_entries)
    }

    /// Start background cleanup task
    ///
    /// # Errors
    /// Returns an error if starting the cleanup task fails.
    pub async fn start_cleanup<F>(self: Arc<Self>, mut cleanup_callback: F) -> MapletResult<()>
    where
        Self: Send + Sync,
        F: FnMut(Vec<TTLEntry>) -> MapletResult<()> + Send + Sync + 'static,
    {
        if !self.config.enable_background_cleanup {
            return Ok(());
        }

        let (shutdown_tx, shutdown_rx) = async_channel::bounded(1);
        let expiration_map = Arc::clone(&self.expiration_map);
        let key_to_expiration = Arc::clone(&self.key_to_expiration);
        let config = self.config.clone();

        // --- smol: race a repeating timer against the shutdown channel ---
        // Tokio takes precedence when both features are enabled (workspace
        // feature unification can enable both simultaneously).
        #[cfg(all(feature = "smol", not(feature = "tokio")))]
        let handle: CleanupHandle = {
            use futures_lite::future::FutureExt as _;
            let interval_duration = Duration::from_secs(config.cleanup_interval_secs);
            Box::pin(smol::spawn(async move {
                loop {
                    let tick = async {
                        smol::Timer::after(interval_duration).await;
                        true
                    };
                    let shutdown = async {
                        let _ = shutdown_rx.recv().await;
                        false
                    };
                    // Returns true on tick, false on shutdown
                    if !tick.or(shutdown).await {
                        break;
                    }
                    run_cleanup_batch(
                        &expiration_map,
                        &key_to_expiration,
                        config.max_cleanup_batch_size,
                        &mut cleanup_callback,
                    )
                    .await;
                }
            }))
        };

        // --- tokio: select! between interval tick and shutdown ---
        #[cfg(feature = "tokio")]
        let handle: CleanupHandle = {
            Box::pin(async move {
                let _ = tokio::spawn(async move {
                    let mut interval =
                        tokio::time::interval(Duration::from_secs(config.cleanup_interval_secs));
                    loop {
                        tokio::select! {
                            _ = interval.tick() => {
                                run_cleanup_batch(
                                    &expiration_map,
                                    &key_to_expiration,
                                    config.max_cleanup_batch_size,
                                    &mut cleanup_callback,
                                ).await;
                            }
                            _ = shutdown_rx.recv() => {
                                break;
                            }
                        }
                    }
                })
                .await;
            })
        };

        // With neither runtime feature enabled there is no executor to spawn the
        // cleanup task onto. The handle is still produced so the unconditional
        // `cleanup_handle` assignment below keeps compiling; the future never runs.
        #[cfg(not(any(feature = "smol", feature = "tokio")))]
        let handle: CleanupHandle = Box::pin(async {});

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
    ///
    /// # Errors
    /// Returns an error if stopping the cleanup task fails.
    pub async fn stop_cleanup(self: Arc<Self>) -> MapletResult<()>
    where
        Self: Send + Sync,
    {
        // Send shutdown signal
        {
            let mut shutdown_tx = self.shutdown_tx.write().await;
            if let Some(tx) = shutdown_tx.take() {
                if let Err(e) = tx.send(()).await {
                    tracing::warn!("Failed to send shutdown signal: {:?}", e);
                }
            }
        }

        // Wait for cleanup task to finish
        {
            let mut cleanup_handle = self.cleanup_handle.write().await;
            if let Some(handle) = cleanup_handle.take() {
                handle.await;
            }
        }

        Ok(())
    }

    /// Get TTL statistics
    ///
    /// # Errors
    /// Returns an error if retrieving statistics fails.
    #[allow(clippy::significant_drop_tightening)]
    pub async fn get_stats(self: Arc<Self>) -> MapletResult<TTLStats>
    where
        Self: Send + Sync,
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let (total_keys, expired_count, next_expiration) = {
            let expiration_map = self.expiration_map.read().await;
            let key_map = self.key_to_expiration.read().await;

            let total_keys = key_map.len();
            let expired_count: usize = expiration_map
                .range(..=now)
                .map(|(_, entries)| entries.len())
                .sum();
            let next_expiration = expiration_map.range(now..).next().map(|(&time, _)| time);

            (total_keys, expired_count, next_expiration)
        };

        Ok(TTLStats {
            total_keys_with_ttl: total_keys as u64,
            expired_keys: expired_count as u64,
            next_expiration,
        })
    }

    /// Clear all TTL entries
    ///
    /// # Errors
    /// Returns an error if clearing fails.
    pub async fn clear_all(self: Arc<Self>) -> MapletResult<()>
    where
        Self: Send + Sync,
    {
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TTLStats {
    /// Total number of keys with TTL set
    pub total_keys_with_ttl: u64,
    /// Number of expired keys waiting for cleanup
    pub expired_keys: u64,
    /// Next expiration timestamp (if any)
    pub next_expiration: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ttl_manager_basic_operations() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        Arc::clone(&manager).set_ttl("key1".to_string(), 0, 60).await.unwrap();

        let ttl = Arc::clone(&manager).get_ttl("key1".to_string()).await.unwrap();
        assert!(ttl.is_some());
        assert!(ttl.unwrap() <= 60);

        assert!(!Arc::clone(&manager).is_expired("key1".to_string()).await.unwrap());

        Arc::clone(&manager).remove_ttl("key1".to_string()).await.unwrap();
        assert!(Arc::clone(&manager).get_ttl("key1".to_string()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        Arc::clone(&manager).set_ttl("key1".to_string(), 0, 1).await.unwrap();

        tokio::time::sleep(Duration::from_millis(1100)).await;

        assert!(Arc::clone(&manager).is_expired("key1".to_string()).await.unwrap());

        let expired = Arc::clone(&manager).get_expired_keys().await.unwrap();
        assert!(!expired.is_empty());
        assert_eq!(expired[0].key, "key1");
    }

    #[tokio::test]
    async fn test_ttl_stats() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        Arc::clone(&manager).set_ttl("key1".to_string(), 0, 60).await.unwrap();
        Arc::clone(&manager).set_ttl("key2".to_string(), 0, 120).await.unwrap();

        let stats = Arc::clone(&manager).get_stats().await.unwrap();
        assert_eq!(stats.total_keys_with_ttl, 2);
        assert_eq!(stats.expired_keys, 0);
        assert!(stats.next_expiration.is_some());
    }

    #[tokio::test]
    async fn test_ttl_clear_all() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        Arc::clone(&manager).set_ttl("key1".to_string(), 0, 60).await.unwrap();
        Arc::clone(&manager).set_ttl("key2".to_string(), 0, 120).await.unwrap();

        Arc::clone(&manager).clear_all().await.unwrap();

        let stats = Arc::clone(&manager).get_stats().await.unwrap();
        assert_eq!(stats.total_keys_with_ttl, 0);
    }

    #[tokio::test]
    async fn test_ttl_config_default() {
        let config = TTLConfig::default();
        assert_eq!(config.cleanup_interval_secs, 60);
        assert_eq!(config.max_cleanup_batch_size, 1000);
        assert!(config.enable_background_cleanup);
    }

    #[tokio::test]
    async fn test_ttl_config_clone() {
        let config = TTLConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.cleanup_interval_secs, 60);
    }

    #[tokio::test]
    async fn test_ttl_entry_new() {
        let entry = TTLEntry::new("key".to_string(), 0, 60);
        assert_eq!(entry.key, "key");
        assert_eq!(entry.db_id, 0);
        assert!(!entry.is_expired());
    }

    #[tokio::test]
    async fn test_ttl_entry_remaining_ttl() {
        let entry = TTLEntry::new("key".to_string(), 0, 60);
        let remaining = entry.remaining_ttl();
        assert!(remaining > 0);
        assert!(remaining <= 60);
    }

    #[tokio::test]
    async fn test_ttl_entry_expired() {
        let entry = TTLEntry::new("key".to_string(), 0, 0);
        assert!(entry.is_expired());
    }

    #[tokio::test]
    async fn test_ttl_entry_serialization() {
        let entry = TTLEntry::new("key".to_string(), 0, 60);
        let serialized = serde_json::to_string(&entry).expect("Failed to serialize");
        let deserialized: TTLEntry =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.key, "key");
    }

    #[tokio::test]
    async fn test_ttl_stats_default() {
        let stats = TTLStats::default();
        assert_eq!(stats.total_keys_with_ttl, 0);
        assert_eq!(stats.expired_keys, 0);
        assert!(stats.next_expiration.is_none());
    }

    #[tokio::test]
    async fn test_ttl_stats_serialization() {
        let stats = TTLStats {
            total_keys_with_ttl: 100,
            expired_keys: 10,
            next_expiration: Some(12345),
        };

        let serialized = serde_json::to_string(&stats).expect("Failed to serialize");
        let deserialized: TTLStats =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.total_keys_with_ttl, 100);
    }

    #[tokio::test]
    async fn test_ttl_manager_get_ttl_nonexistent() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        let ttl = Arc::clone(&manager).get_ttl("nonexistent".to_string()).await.unwrap();
        assert!(ttl.is_none());
    }

    #[tokio::test]
    async fn test_ttl_manager_is_expired_nonexistent() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        let expired = Arc::clone(&manager).is_expired("nonexistent".to_string()).await.unwrap();
        assert!(!expired);
    }

    #[tokio::test]
    async fn test_ttl_manager_remove_ttl_nonexistent() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        assert!(Arc::clone(&manager).remove_ttl("nonexistent".to_string()).await.is_ok());
    }

    #[tokio::test]
    async fn test_ttl_manager_update_ttl() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        Arc::clone(&manager).set_ttl("key1".to_string(), 0, 60).await.unwrap();
        Arc::clone(&manager).set_ttl("key1".to_string(), 0, 120).await.unwrap();

        let ttl = Arc::clone(&manager).get_ttl("key1".to_string()).await.unwrap();
        assert!(ttl.is_some());
        assert!(ttl.unwrap() <= 120);
    }

    #[tokio::test]
    async fn test_ttl_manager_multiple_keys() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        for i in 0..10 {
            Arc::clone(&manager).set_ttl(format!("key{}", i), 0, 60).await.unwrap();
        }

        let stats = Arc::clone(&manager).get_stats().await.unwrap();
        assert_eq!(stats.total_keys_with_ttl, 10);
    }

    #[tokio::test]
    async fn test_ttl_manager_get_expired_keys_empty() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        let expired = Arc::clone(&manager).get_expired_keys().await.unwrap();
        assert!(expired.is_empty());
    }

    #[tokio::test]
    async fn test_ttl_manager_futures_are_send() {
        let config = TTLConfig::default();
        let manager = Arc::new(TTLManager::new(config));

        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            Arc::clone(&manager_clone).set_ttl("key1".to_string(), 0, 60).await
        });
        assert!(handle.await.unwrap().is_ok());

        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            Arc::clone(&manager_clone).get_ttl("key1".to_string()).await
        });
        assert!(handle.await.unwrap().unwrap().is_some());

        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            Arc::clone(&manager_clone).is_expired("key1".to_string()).await
        });
        assert!(!handle.await.unwrap().unwrap());

        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            Arc::clone(&manager_clone).remove_ttl("key1".to_string()).await
        });
        assert!(handle.await.unwrap().is_ok());

        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            Arc::clone(&manager_clone).get_stats().await
        });
        assert!(handle.await.unwrap().is_ok());
    }

    #[tokio::test]
    async fn test_ttl_config_custom() {
        let config = TTLConfig {
            cleanup_interval_secs: 30,
            max_cleanup_batch_size: 500,
            enable_background_cleanup: false,
        };

        assert_eq!(config.cleanup_interval_secs, 30);
        assert_eq!(config.max_cleanup_batch_size, 500);
        assert!(!config.enable_background_cleanup);
    }
}
