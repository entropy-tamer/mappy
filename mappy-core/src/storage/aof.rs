//! Append-Only File (AOF) storage backend
//!
//! Provides durability through append-only logging with periodic sync.
use super::{Storage, StorageConfig, StorageStats};
use crate::{MapletError, MapletResult};
use async_lock::RwLock;
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

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
    sync_handle: Option<std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum AOFEntry {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

impl AOFStorage {
    /// Create a new AOF storage
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

    /// Append entry to AOF file
    fn append_to_aof(&self, entry: &AOFEntry) -> MapletResult<()> {
        let line = serde_json::to_string(&entry)
            .map_err(|e| MapletError::Internal(format!("Failed to serialize AOF entry: {e}")))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.aof_path)
            .map_err(|e| {
                MapletError::Internal(format!("Failed to open AOF file for append: {e}"))
            })?;

        writeln!(file, "{line}")
            .map_err(|e| MapletError::Internal(format!("Failed to write to AOF file: {e}")))?;

        Ok(())
    }

    /// Start background sync task
    fn start_sync_task(&mut self) {
        let aof_path = self.aof_path.clone();
        let sync_interval = Duration::from_secs(self.config.sync_interval);

        // Tokio takes precedence when both features are enabled (workspace
        // feature unification can enable both simultaneously).
        #[cfg(all(feature = "smol", not(feature = "tokio")))]
        let handle: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>> =
            Box::pin(smol::spawn(async move {
                loop {
                    smol::Timer::after(sync_interval).await;
                    if aof_path.exists()
                        && let Err(e) = std::fs::File::open(&aof_path).and_then(|f| f.sync_all())
                    {
                        eprintln!("Failed to sync AOF file: {e}");
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
                        if aof_path.exists()
                            && let Err(e) =
                                std::fs::File::open(&aof_path).and_then(|f| f.sync_all())
                        {
                            eprintln!("Failed to sync AOF file: {e}");
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
            std::fs::metadata(&self.aof_path).map_or(0, |m| m.len())
        } else {
            0
        }
    }
}

/// Controls how offline compaction treats the original AOF.
#[derive(Debug, Clone, Default)]
pub struct CompactOptions {
    /// Hard-link destination for the pre-compaction file. Defaults to
    /// `<stem>.aof.pre-compact` next to the AOF.
    pub backup_path: Option<PathBuf>,
    /// Replace an existing backup instead of refusing to run.
    pub overwrite_backup: bool,
    /// Skip the backup entirely. Only useful when disk is critically short.
    pub skip_backup: bool,
}

/// What an offline compaction actually kept.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AofCompactionReport {
    /// Bytes in the original AOF before compaction.
    pub input_bytes: u64,
    /// Bytes in the compacted AOF.
    pub output_bytes: u64,
    /// Non-empty JSON records read in the first pass.
    pub records_read: u64,
    /// Records copied into the compacted AOF.
    pub records_written: u64,
    /// Distinct live keys retained.
    pub keys_retained: u64,
    /// Delete records encountered and dropped.
    pub tombstones_dropped: u64,
}

#[derive(serde::Deserialize)]
enum CompactionEntry {
    Set {
        key: String,
        #[allow(dead_code)]
        value: serde::de::IgnoredAny,
    },
    Delete {
        key: String,
    },
}

struct LiveOffsets {
    by_key: HashMap<String, u64>,
    records_read: u64,
    tombstones_dropped: u64,
}

#[allow(clippy::cast_possible_truncation)] // AOF line lengths fit u64 on every supported target
fn scan_live_offsets(path: &Path) -> MapletResult<LiveOffsets> {
    let file = File::open(path)
        .map_err(|e| MapletError::Internal(format!("Failed to open AOF for scanning: {e}")))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut offset = 0_u64;
    let mut by_key = HashMap::new();
    let mut records_read = 0_u64;
    let mut tombstones_dropped = 0_u64;

    loop {
        line.clear();
        let read = reader.read_line(&mut line).map_err(|e| {
            MapletError::Internal(format!("Failed to read AOF line at byte {offset}: {e}"))
        })?;
        if read == 0 {
            break;
        }
        if line.trim().is_empty() {
            offset += read as u64;
            continue;
        }

        let entry: CompactionEntry = serde_json::from_str(&line).map_err(|e| {
            MapletError::Internal(format!("Failed to parse AOF entry at byte {offset}: {e}"))
        })?;
        records_read += 1;
        match entry {
            CompactionEntry::Set { key, value: _ } => {
                by_key.insert(key, offset);
            }
            CompactionEntry::Delete { key } => {
                by_key.remove(&key);
                tombstones_dropped += 1;
            }
        }
        offset += read as u64;
    }

    Ok(LiveOffsets {
        by_key,
        records_read,
        tombstones_dropped,
    })
}

/// Report what a compaction would keep without changing the file.
///
/// # Errors
/// Returns an error when the file cannot be read or a non-empty line is not a valid AOF entry.
pub fn analyze_aof(path: &Path) -> MapletResult<AofCompactionReport> {
    let input_bytes = std::fs::metadata(path)
        .map_err(|e| MapletError::Internal(format!("Failed to stat AOF: {e}")))?
        .len();
    let scan = scan_live_offsets(path)?;
    let keys_retained = scan.by_key.len() as u64;
    let mut offsets = scan.by_key.into_values().collect::<Vec<_>>();
    offsets.sort_unstable();

    let file = File::open(path)
        .map_err(|e| MapletError::Internal(format!("Failed to reopen AOF for analysis: {e}")))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut byte_offset = 0_u64;
    let mut output_bytes = 0_u64;
    let mut records_written = 0_u64;
    loop {
        line.clear();
        let read = reader.read_line(&mut line).map_err(|e| {
            MapletError::Internal(format!(
                "Failed to reread AOF line at byte {byte_offset}: {e}"
            ))
        })?;
        if read == 0 {
            break;
        }
        if offsets.binary_search(&byte_offset).is_ok() {
            output_bytes += read as u64;
            if !line.ends_with('\n') {
                output_bytes += 1;
            }
            records_written += 1;
        }
        byte_offset += read as u64;
    }

    Ok(AofCompactionReport {
        input_bytes,
        output_bytes,
        records_read: scan.records_read,
        records_written,
        keys_retained,
        tombstones_dropped: scan.tombstones_dropped,
    })
}

#[allow(clippy::cast_possible_truncation)] // AOF line lengths fit u64 on every supported target
fn copy_live_entries(path: &Path, temp_path: &Path, offsets: &[u64]) -> MapletResult<(u64, u64)> {
    let input = File::open(path)
        .map_err(|e| MapletError::Internal(format!("Failed to reopen AOF for copying: {e}")))?;
    let output = File::create(temp_path)
        .map_err(|e| MapletError::Internal(format!("Failed to create compacted AOF: {e}")))?;
    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);
    let mut line = String::new();
    let mut offset = 0_u64;
    let mut records_written = 0_u64;

    loop {
        line.clear();
        let read = reader.read_line(&mut line).map_err(|e| {
            MapletError::Internal(format!("Failed to reread AOF line at byte {offset}: {e}"))
        })?;
        if read == 0 {
            break;
        }
        if offsets.binary_search(&offset).is_ok() {
            writer.write_all(line.as_bytes()).map_err(|e| {
                MapletError::Internal(format!("Failed to write compacted AOF entry: {e}"))
            })?;
            if !line.ends_with('\n') {
                writer.write_all(b"\n").map_err(|e| {
                    MapletError::Internal(format!("Failed to terminate compacted AOF entry: {e}"))
                })?;
            }
            records_written += 1;
        }
        offset += read as u64;
    }

    writer
        .flush()
        .map_err(|e| MapletError::Internal(format!("Failed to flush compacted AOF: {e}")))?;
    writer
        .get_ref()
        .sync_all()
        .map_err(|e| MapletError::Internal(format!("Failed to sync compacted AOF: {e}")))?;

    Ok((records_written, offset))
}

fn sync_parent_dir(path: &Path) -> MapletResult<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|e| {
            MapletError::Internal(format!(
                "Failed to sync AOF directory {}: {e}",
                parent.display()
            ))
        })
}

/// Compact an AOF in place with bounded memory.
///
/// The first pass keeps only the byte offset of the last `Set` for each key;
/// values are parsed as `IgnoredAny`, so a 4 KiB vector does not become a
/// resident `Vec<u8>`. The second pass copies only surviving lines into a temp
/// file in the same directory, syncs it, hard-links a backup of the original,
/// and atomically renames the temp file over the AOF. A concurrent append is
/// detected by comparing the byte count of the second pass with the original
/// size; in that case the original is left untouched.
///
/// This is offline-only. Stop every writer before calling it.
///
/// # Errors
/// Returns an error if the AOF is malformed, changes while being copied, or
/// the backup/rename cannot be completed.
pub fn compact_aof(
    path: impl AsRef<Path>,
    options: CompactOptions,
) -> MapletResult<AofCompactionReport> {
    let path = path.as_ref();
    let metadata = std::fs::metadata(path)
        .map_err(|e| MapletError::Internal(format!("Failed to stat AOF: {e}")))?;
    let input_bytes = metadata.len();

    let live = scan_live_offsets(path)?;
    let mut offsets: Vec<u64> = live.by_key.into_values().collect();
    offsets.sort_unstable();

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| MapletError::Internal("AOF path has no UTF-8 file name".to_string()))?;
    let nonce = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let temp_path = parent.join(format!(".{file_name}.compact-{nonce}-{nanos}.tmp"));

    let (records_written, rescanned_bytes) = match copy_live_entries(path, &temp_path, &offsets) {
        Ok(result) => result,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }
    };
    if rescanned_bytes != input_bytes {
        let _ = std::fs::remove_file(&temp_path);
        return Err(MapletError::Internal(format!(
            "AOF changed during compaction: {input_bytes} bytes before, {rescanned_bytes} bytes after"
        )));
    }

    let output_bytes = std::fs::metadata(&temp_path)
        .map_err(|e| MapletError::Internal(format!("Failed to stat compacted AOF: {e}")))?
        .len();

    let backup_path = options
        .backup_path
        .unwrap_or_else(|| path.with_extension("aof.pre-compact"));
    if !options.skip_backup {
        if backup_path == path {
            let _ = std::fs::remove_file(&temp_path);
            return Err(MapletError::Internal(
                "AOF backup path must differ from the AOF itself".to_string(),
            ));
        }
        if backup_path.exists() {
            if !options.overwrite_backup {
                let _ = std::fs::remove_file(&temp_path);
                return Err(MapletError::Internal(format!(
                    "Backup already exists: {}",
                    backup_path.display()
                )));
            }
            std::fs::remove_file(&backup_path).map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                MapletError::Internal(format!("Failed to remove old backup: {e}"))
            })?;
        }
        std::fs::hard_link(path, &backup_path).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            MapletError::Internal(format!(
                "Failed to hard-link backup {}: {e}",
                backup_path.display()
            ))
        })?;
    }

    std::fs::rename(&temp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        MapletError::Internal(format!("Failed to install compacted AOF: {e}"))
    })?;
    sync_parent_dir(path)?;

    Ok(AofCompactionReport {
        input_bytes,
        output_bytes,
        records_read: live.records_read,
        records_written,
        keys_retained: offsets.len() as u64,
        tombstones_dropped: live.tombstones_dropped,
    })
}

#[async_trait]
impl Storage for AOFStorage {
    async fn get(&self, key: &str) -> MapletResult<Option<Vec<u8>>> {
        let start = Instant::now();
        let result = self.cache.get(key).map(|entry| entry.value().clone());
        #[allow(clippy::cast_possible_truncation)] // Microseconds won't exceed u64
        let latency = start.elapsed().as_micros() as u64;

        self.update_stats(|stats| {
            stats.operations_count += 1;
            stats.avg_latency_us = u64::midpoint(stats.avg_latency_us, latency);
        })
        .await;

        Ok(result)
    }

    async fn set(&self, key: String, value: Vec<u8>) -> MapletResult<()> {
        let start = Instant::now();

        // Update cache
        self.cache.insert(key.clone(), value.clone());

        // Append to AOF
        let entry = AOFEntry::Set { key, value };
        self.append_to_aof(&entry)?;

        #[allow(clippy::cast_possible_truncation)] // Microseconds won't exceed u64
        let latency = start.elapsed().as_micros() as u64;

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
            // Append delete to AOF
            let entry = AOFEntry::Delete {
                key: key.to_string(),
            };
            self.append_to_aof(&entry)?;
        }

        #[allow(clippy::cast_possible_truncation)] // Microseconds won't exceed u64
        let latency = start.elapsed().as_micros() as u64;

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
        #[allow(clippy::cast_possible_truncation)] // Microseconds won't exceed u64
        let latency = start.elapsed().as_micros() as u64;

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
        #[allow(clippy::cast_possible_truncation)] // Microseconds won't exceed u64
        let latency = start.elapsed().as_micros() as u64;

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

        // Clear AOF file
        std::fs::write(&self.aof_path, "")
            .map_err(|e| MapletError::Internal(format!("Failed to clear AOF file: {e}")))?;

        #[allow(clippy::cast_possible_truncation)] // Microseconds won't exceed u64
        let latency = start.elapsed().as_micros() as u64;

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

        // Force sync AOF file to disk
        std::fs::File::open(&self.aof_path)
            .and_then(|f| f.sync_all())
            .map_err(|e| MapletError::Internal(format!("Failed to sync AOF file: {e}")))?;

        #[allow(clippy::cast_possible_truncation)] // Microseconds won't exceed u64
        let latency = start.elapsed().as_micros() as u64;

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
    async fn test_compact_aof_keeps_last_set_and_honors_delete() {
        let temp_dir = TempDir::new().unwrap();
        let aof_path = temp_dir.path().join("mappy.aof");
        let entries = [
            AOFEntry::Set {
                key: "live".to_string(),
                value: b"first".to_vec(),
            },
            AOFEntry::Set {
                key: "dead".to_string(),
                value: b"gone".to_vec(),
            },
            AOFEntry::Delete {
                key: "dead".to_string(),
            },
            AOFEntry::Set {
                key: "live".to_string(),
                value: b"second".to_vec(),
            },
        ];
        let body = entries
            .iter()
            .map(|entry| serde_json::to_string(entry).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&aof_path, format!("{body}\n")).unwrap();

        let report = compact_aof(&aof_path, CompactOptions::default()).unwrap();
        assert_eq!(report.records_read, 4);
        assert_eq!(report.records_written, 1);
        assert_eq!(report.keys_retained, 1);
        assert_eq!(report.tombstones_dropped, 1);
        assert!(report.output_bytes < report.input_bytes);
        assert!(aof_path.with_extension("aof.pre-compact").exists());

        let storage = AOFStorage::new(StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(storage.get("live").await.unwrap(), Some(b"second".to_vec()));
        assert_eq!(storage.get("dead").await.unwrap(), None);
    }

    #[test]
    fn test_compact_aof_rejects_malformed_line_without_touching_original() {
        let temp_dir = TempDir::new().unwrap();
        let aof_path = temp_dir.path().join("mappy.aof");
        let body = "not json\n";
        std::fs::write(&aof_path, body).unwrap();

        let error = compact_aof(&aof_path, CompactOptions::default()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Failed to parse AOF entry at byte 0")
        );
        assert_eq!(std::fs::read_to_string(&aof_path).unwrap(), body);
        assert!(!aof_path.with_extension("aof.pre-compact").exists());
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
            storage
                .set("key1".to_string(), b"value1".to_vec())
                .await
                .unwrap();
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

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.total_keys, 1);
        assert!(stats.memory_usage > 0);
        assert!(stats.disk_usage > 0);
        assert!(stats.operations_count > 0);
    }

    #[tokio::test]
    async fn test_aof_storage_clear() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        storage.clear_database().await.unwrap();

        assert_eq!(storage.get("key1").await.unwrap(), None);
        assert_eq!(storage.keys().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_aof_storage_keys() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

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
    async fn test_aof_storage_flush() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        assert!(storage.flush().await.is_ok());
    }

    #[tokio::test]
    async fn test_aof_storage_close() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

        storage
            .set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        assert!(storage.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_aof_storage_delete_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

        let deleted = storage.delete("nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_aof_storage_get_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

        let value = storage.get("nonexistent").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_aof_storage_empty_keys() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn test_aof_storage_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

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
    async fn test_aof_storage_large_value() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

        let large_value = vec![0u8; 10000];
        storage
            .set("key1".to_string(), large_value.clone())
            .await
            .unwrap();

        let value = storage.get("key1").await.unwrap();
        assert_eq!(value, Some(large_value));
    }

    #[tokio::test]
    async fn test_aof_storage_multiple_deletes() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            sync_interval: 1,
            ..Default::default()
        };
        let storage = AOFStorage::new(config).unwrap();

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
            AOFEntry::Delete { .. } => panic!("Expected Set entry"),
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
            AOFEntry::Set { .. } => panic!("Expected Delete entry"),
        }
    }
}
