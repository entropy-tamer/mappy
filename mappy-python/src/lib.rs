//! # Mappy Python Bindings
//! 
//! Python bindings for mappy maplet data structures using PyO3.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use mappy_core::{Maplet, CounterOperator, Engine, EngineConfig, EngineStats, PersistenceMode};
use mappy_core::types::MapletConfig;
use mappy_core::storage::{StorageConfig, StorageStats};
use mappy_core::ttl::{TTLConfig, TTLStats};
use std::sync::Arc;
use tokio::runtime::Runtime;
use serde::{Serialize, Deserialize};

/// Python wrapper for Maplet (legacy support)
#[pyclass]
pub struct PyMaplet {
    inner: Maplet<String, u64, CounterOperator>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyMaplet {
    #[new]
    fn new(capacity: usize, false_positive_rate: f64) -> PyResult<Self> {
        let maplet = Maplet::new(capacity, false_positive_rate)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?);
        Ok(Self { inner: maplet, runtime })
    }
    
    fn insert(&mut self, key: String, value: u64) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.insert(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }
    
    fn query(&self, key: &str) -> Option<u64> {
        self.runtime.block_on(async {
            self.inner.query(&key.to_string()).await
        })
    }
    
    fn contains(&self, key: &str) -> bool {
        self.runtime.block_on(async {
            self.inner.contains(&key.to_string()).await
        })
    }
    
    fn len(&self) -> usize {
        self.runtime.block_on(async {
            self.inner.len().await
        })
    }
    
    fn is_empty(&self) -> bool {
        self.runtime.block_on(async {
            self.inner.is_empty().await
        })
    }
    
    fn error_rate(&self) -> f64 {
        self.inner.error_rate()
    }
    
    fn load_factor(&self) -> f64 {
        self.runtime.block_on(async {
            self.inner.load_factor().await
        })
    }
}

/// Python wrapper for Engine configuration
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyEngineConfig {
    #[pyo3(get, set)]
    pub capacity: usize,
    #[pyo3(get, set)]
    pub false_positive_rate: f64,
    #[pyo3(get, set)]
    pub persistence_mode: String,
    #[pyo3(get, set)]
    pub data_dir: Option<String>,
    #[pyo3(get, set)]
    pub memory_capacity: Option<usize>,
    #[pyo3(get, set)]
    pub aof_sync_interval_ms: Option<u64>,
    #[pyo3(get, set)]
    pub ttl_enabled: bool,
    #[pyo3(get, set)]
    pub ttl_cleanup_interval_ms: u64,
}

#[pymethods]
impl PyEngineConfig {
    #[new]
    fn new(
        capacity: Option<usize>,
        false_positive_rate: Option<f64>,
        persistence_mode: Option<String>,
        data_dir: Option<String>,
        memory_capacity: Option<usize>,
        aof_sync_interval_ms: Option<u64>,
        ttl_enabled: Option<bool>,
        ttl_cleanup_interval_ms: Option<u64>,
    ) -> Self {
        Self {
            capacity: capacity.unwrap_or(10000),
            false_positive_rate: false_positive_rate.unwrap_or(0.01),
            persistence_mode: persistence_mode.unwrap_or_else(|| "hybrid".to_string()),
            data_dir,
            memory_capacity,
            aof_sync_interval_ms,
            ttl_enabled: ttl_enabled.unwrap_or(true),
            ttl_cleanup_interval_ms: ttl_cleanup_interval_ms.unwrap_or(1000),
        }
    }

}

impl PyEngineConfig {
    fn to_rust_config(&self) -> Result<EngineConfig, PyErr> {
        let persistence_mode = match self.persistence_mode.as_str() {
            "memory" => PersistenceMode::Memory,
            "disk" => PersistenceMode::Disk,
            "aof" => PersistenceMode::AOF,
            "hybrid" => PersistenceMode::Hybrid,
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid persistence mode: {}", self.persistence_mode)
            )),
        };

        let maplet_config = MapletConfig {
            capacity: self.capacity,
            false_positive_rate: self.false_positive_rate,
            max_load_factor: 0.95,
            auto_resize: true,
            enable_deletion: true,
            enable_merging: true,
        };

        let storage_config = StorageConfig {
            data_dir: self.data_dir.clone().unwrap_or_else(|| "./data".to_string()),
            max_memory: self.memory_capacity.map(|v| v as u64),
            enable_compression: true,
            sync_interval: self.aof_sync_interval_ms.unwrap_or(1000) / 1000, // Convert ms to seconds
            write_buffer_size: 1024 * 1024, // 1MB
        };

        let ttl_config = TTLConfig {
            cleanup_interval_secs: self.ttl_cleanup_interval_ms / 1000, // Convert ms to seconds
            max_cleanup_batch_size: 1000,
            enable_background_cleanup: self.ttl_enabled,
        };

        Ok(EngineConfig {
            maplet: maplet_config,
            storage: storage_config,
            ttl: ttl_config,
            persistence_mode,
            data_dir: self.data_dir.clone(),
        })
    }
}

/// Python wrapper for Engine statistics
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyEngineStats {
    #[pyo3(get)]
    pub uptime_seconds: u64,
    #[pyo3(get)]
    pub total_operations: u64,
    #[pyo3(get)]
    pub maplet_capacity: usize,
    #[pyo3(get)]
    pub maplet_size: usize,
    #[pyo3(get)]
    pub maplet_load_factor: f64,
    #[pyo3(get)]
    pub maplet_error_rate: f64,
    #[pyo3(get)]
    pub maplet_memory_usage: usize,
    #[pyo3(get)]
    pub storage_operations: u64,
    #[pyo3(get)]
    pub storage_memory_usage: usize,
    #[pyo3(get)]
    pub ttl_entries: usize,
    #[pyo3(get)]
    pub ttl_cleanups: u64,
}

impl From<EngineStats> for PyEngineStats {
    fn from(stats: EngineStats) -> Self {
        Self {
            uptime_seconds: stats.uptime_seconds,
            total_operations: stats.total_operations,
            maplet_capacity: stats.maplet_stats.capacity,
            maplet_size: stats.maplet_stats.len,
            maplet_load_factor: stats.maplet_stats.load_factor,
            maplet_error_rate: stats.maplet_stats.false_positive_rate,
            maplet_memory_usage: stats.maplet_stats.memory_usage,
            storage_operations: stats.storage_stats.operations_count,
            storage_memory_usage: stats.storage_stats.memory_usage as usize,
            ttl_entries: stats.ttl_stats.total_keys_with_ttl as usize,
            ttl_cleanups: stats.ttl_stats.expired_keys,
        }
    }
}

/// Python wrapper for Engine
#[pyclass]
pub struct PyEngine {
    inner: Arc<Engine>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyEngine {
    #[new]
    fn new(config: Option<PyEngineConfig>) -> PyResult<Self> {
        let config = config.unwrap_or_else(|| PyEngineConfig::new(None, None, None, None, None, None, None, None));
        let rust_config = config.to_rust_config()?;
        
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?);
        
        let engine = runtime.block_on(async {
            Engine::new(rust_config).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        
        Ok(Self {
            inner: Arc::new(engine),
            runtime,
        })
    }

    /// Set a key-value pair
    fn set(&self, key: String, value: Vec<u8>) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.set(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }

    /// Get a value by key
    fn get(&self, key: String) -> PyResult<Option<Vec<u8>>> {
        Ok(self.runtime.block_on(async {
            self.inner.get(&key).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?)
    }

    /// Check if a key exists
    fn exists(&self, key: String) -> PyResult<bool> {
        Ok(self.runtime.block_on(async {
            self.inner.exists(&key).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?)
    }

    /// Delete a key
    fn delete(&self, key: String) -> PyResult<bool> {
        Ok(self.runtime.block_on(async {
            self.inner.delete(&key).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?)
    }

    /// Get all keys
    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.runtime.block_on(async {
            self.inner.keys().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?)
    }

    /// Clear all data
    fn clear(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.clear().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }

    /// Set TTL for a key
    fn expire(&self, key: String, seconds: u64) -> PyResult<bool> {
        Ok(self.runtime.block_on(async {
            self.inner.expire(&key, seconds).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?)
    }

    /// Get TTL for a key
    fn ttl(&self, key: String) -> PyResult<Option<u64>> {
        let result = self.runtime.block_on(async {
            self.inner.ttl(&key).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        
        Ok(result.map(|v| v as u64))
    }

    /// Remove TTL from a key (make it persistent)
    fn persist(&self, key: String) -> PyResult<bool> {
        Ok(self.runtime.block_on(async {
            self.inner.persist(&key).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?)
    }

    /// Set TTL for multiple keys (batch operation)
    fn expire_many(&self, keys: Vec<String>, seconds: u64) -> PyResult<usize> {
        let mut count = 0;
        for key in keys {
            if self.runtime.block_on(async {
                self.inner.expire(&key, seconds).await
            }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))? {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Get all keys with TTL
    fn keys_with_ttl(&self) -> PyResult<Vec<String>> {
        // For now, return empty list as the Engine doesn't expose this method
        // This would need to be implemented in the Engine if needed
        Ok(vec![])
    }

    /// Get engine statistics
    fn stats(&self) -> PyResult<PyEngineStats> {
        let stats = self.runtime.block_on(async {
            self.inner.stats().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        
        Ok(PyEngineStats::from(stats))
    }

    /// Get memory usage in bytes
    fn memory_usage(&self) -> PyResult<usize> {
        let usage = self.runtime.block_on(async {
            self.inner.memory_usage().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        
        Ok(usage as usize)
    }

    /// Flush pending writes to disk
    fn flush(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.flush().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }

    /// Close the engine and cleanup resources
    fn close(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.close().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }
}

/// Python module definition
#[pymodule]
fn mappy_python(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyMaplet>()?;
    m.add_class::<PyEngine>()?;
    m.add_class::<PyEngineConfig>()?;
    m.add_class::<PyEngineStats>()?;
    Ok(())
}
