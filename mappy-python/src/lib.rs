//! # Mappy Python Bindings
//! 
//! Python bindings for mappy maplet data structures using PyO3.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
// IntoPy trait should be in prelude
use mappy_core::{
    Maplet, CounterOperator, MaxOperator, MinOperator, VectorOperator,
    Engine, EngineConfig, EngineStats, PersistenceMode
};
use mappy_core::types::MapletConfig;
use mappy_core::storage::StorageConfig;
use mappy_core::ttl::TTLConfig;
// use mappy_core::operators::MergeOperator;  // Not needed yet
use std::sync::Arc;
use std::ffi::CString;
use tokio::runtime::Runtime;
use serde::{Serialize, Deserialize};

// ============================================================================
// Operator Classes
// ============================================================================

/// Python wrapper for CounterOperator
#[pyclass]
#[derive(Clone, Copy)]
pub struct PyCounterOperator {
    inner: CounterOperator,
}

#[pymethods]
impl PyCounterOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: CounterOperator,
        }
    }
}

/// Python wrapper for MaxOperator
#[pyclass]
#[derive(Clone, Copy)]
pub struct PyMaxOperator {
    inner: MaxOperator,
}

#[pymethods]
impl PyMaxOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: MaxOperator,
        }
    }
}

/// Python wrapper for MinOperator
#[pyclass]
#[derive(Clone, Copy)]
pub struct PyMinOperator {
    inner: MinOperator,
}

#[pymethods]
impl PyMinOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: MinOperator,
        }
    }
}

/// Python wrapper for VectorOperator
#[pyclass]
#[derive(Clone, Copy)]
pub struct PyVectorOperator {
    inner: VectorOperator,
}

#[pymethods]
impl PyVectorOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: VectorOperator,
        }
    }
}

// ============================================================================
// Generic Maplet with Operator Support
// ============================================================================

/// Helper function to convert Rust values to Python objects  
fn to_py_any_u64(py: Python<'_>, value: u64) -> Bound<PyAny> {
    // Convert u64 to Python int - PyO3 0.27 requires explicit conversion
    // Use PyObject which implements From<u64> via IntoPy trait (in prelude)
    // Since into_py is not available, use a workaround: convert to string then eval
    let s = CString::new(value.to_string()).unwrap();
    py.eval(s.as_c_str(), None, None).unwrap()
}

fn to_py_any_f64(py: Python<'_>, value: f64) -> Bound<PyAny> {
    // Convert f64 to Python float - use eval as workaround
    let s = CString::new(value.to_string()).unwrap();
    py.eval(s.as_c_str(), None, None).unwrap()
}

/// Enum for different maplet types based on operator
pub enum PyMapletGenericInner {
    Counter(Arc<Maplet<String, u64, CounterOperator>>),
    MaxU64(Arc<Maplet<String, u64, MaxOperator>>),
    MaxF64(Arc<Maplet<String, f64, MaxOperator>>),
    MinU64(Arc<Maplet<String, u64, MinOperator>>),
    MinF64(Arc<Maplet<String, f64, MinOperator>>),
    Vector(Arc<Maplet<String, Vec<f64>, VectorOperator>>),
}

/// Python wrapper for Maplet with operator support
#[pyclass]
pub struct PyMapletGeneric {
    inner: PyMapletGenericInner,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyMapletGeneric {
    #[new]
    #[pyo3(signature = (capacity, false_positive_rate, operator = None))]
    fn new(
        capacity: usize,
        false_positive_rate: f64,
        operator: Option<Bound<PyAny>>,
    ) -> PyResult<Self> {
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to create runtime: {e}")
            ))?);

        let inner = if let Some(op) = operator {
            Python::with_gil(|py| -> PyResult<PyMapletGenericInner> {
                // Check operator type
                if op.is_instance_of::<PyCounterOperator>() {
                    let maplet = Maplet::<String, u64, CounterOperator>::new(capacity, false_positive_rate)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                    Ok(PyMapletGenericInner::Counter(Arc::new(maplet)))
                } else if op.is_instance_of::<PyMaxOperator>() {
                    // Default to u64 for MaxOperator
                    let maplet = Maplet::<String, u64, MaxOperator>::new(capacity, false_positive_rate)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                    Ok(PyMapletGenericInner::MaxU64(Arc::new(maplet)))
                } else if op.is_instance_of::<PyMinOperator>() {
                    // Default to u64 for MinOperator
                    let maplet = Maplet::<String, u64, MinOperator>::new(capacity, false_positive_rate)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                    Ok(PyMapletGenericInner::MinU64(Arc::new(maplet)))
                } else if op.is_instance_of::<PyVectorOperator>() {
                    let maplet = Maplet::<String, Vec<f64>, VectorOperator>::new(capacity, false_positive_rate)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                    Ok(PyMapletGenericInner::Vector(Arc::new(maplet)))
                } else {
                    Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Unknown operator type"
                    ))
                }
            })?
        } else {
            // Default to CounterOperator
            let maplet = Maplet::<String, u64, CounterOperator>::new(capacity, false_positive_rate)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
            PyMapletGenericInner::Counter(Arc::new(maplet))
        };

        Ok(Self { inner, runtime })
    }

    fn insert(&mut self, key: String, value: Bound<PyAny>) -> PyResult<()> {
        Python::with_gil(|py| {
            match &self.inner {
                PyMapletGenericInner::Counter(maplet) => {
                    let val: u64 = value.extract()?;
                    self.runtime.block_on(async {
                        maplet.insert(key, val).await
                    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                }
                PyMapletGenericInner::MaxU64(maplet) => {
                    let val: u64 = value.extract()?;
                    self.runtime.block_on(async {
                        maplet.insert(key, val).await
                    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                }
                PyMapletGenericInner::MaxF64(maplet) => {
                    let val: f64 = value.extract()?;
                    self.runtime.block_on(async {
                        maplet.insert(key, val).await
                    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                }
                PyMapletGenericInner::MinU64(maplet) => {
                    let val: u64 = value.extract()?;
                    self.runtime.block_on(async {
                        maplet.insert(key, val).await
                    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                }
                PyMapletGenericInner::MinF64(maplet) => {
                    let val: f64 = value.extract()?;
                    self.runtime.block_on(async {
                        maplet.insert(key, val).await
                    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                }
                PyMapletGenericInner::Vector(maplet) => {
                    // Handle NumPy arrays and lists
                    let vec: Vec<f64> = if let Ok(list) = value.cast::<PyList>() {
                        // Convert Python list to Vec<f64>
                        list.iter().map(|item| {
                            item.extract::<f64>()
                                .or_else(|_| item.extract::<f32>().map(|x| x as f64))
                                .or_else(|_| item.extract::<i64>().map(|x| x as f64))
                                .or_else(|_| item.extract::<i32>().map(|x| x as f64))
                        }).collect::<Result<Vec<f64>, _>>()?
                    } else {
                        // Try to get numpy array attributes
                        if value.getattr("shape").is_ok() {
                            // It's a numpy array - try to convert to list first, then extract
                            if let Ok(tolist) = value.call_method0("tolist") {
                                if let Ok(list) = tolist.cast::<PyList>() {
                                    list.iter().map(|item| {
                                        item.extract::<f64>()
                                            .or_else(|_| item.extract::<f32>().map(|x| x as f64))
                                            .or_else(|_| item.extract::<i64>().map(|x| x as f64))
                                            .or_else(|_| item.extract::<i32>().map(|x| x as f64))
                                    }).collect::<Result<Vec<f64>, _>>()?
                                } else {
                                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                                        "Could not convert numpy array tolist() result to list"
                                    ));
                                }
                            } else {
                                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                                    "Could not call tolist() on numpy array"
                                ));
                            }
                        } else {
                            // Try direct extraction
                            value.extract()?
                        }
                    };
                    self.runtime.block_on(async {
                        maplet.insert(key, vec).await
                    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
                }
            }
            Ok(())
        })
    }

    fn query(&self, key: &str) -> PyResult<Option<PyObject>> {
        Python::with_gil(|py| -> PyResult<Option<PyObject>> {
            match &self.inner {
                PyMapletGenericInner::Counter(maplet) => {
                    let result = self.runtime.block_on(async {
                        maplet.query(&key.to_string()).await
                    });
                    Ok(result.map(|v| to_py_any_u64(py, v).into()))
                }
                PyMapletGenericInner::MaxU64(maplet) => {
                    let result = self.runtime.block_on(async {
                        maplet.query(&key.to_string()).await
                    });
                    Ok(result.map(|v| to_py_any_u64(py, v).into()))
                }
                PyMapletGenericInner::MaxF64(maplet) => {
                    let result = self.runtime.block_on(async {
                        maplet.query(&key.to_string()).await
                    });
                    Ok(result.map(|v| to_py_any_f64(py, v).into()))
                }
                PyMapletGenericInner::MinU64(maplet) => {
                    let result = self.runtime.block_on(async {
                        maplet.query(&key.to_string()).await
                    });
                    Ok(result.map(|v| to_py_any_u64(py, v).into()))
                }
                PyMapletGenericInner::MinF64(maplet) => {
                    let result = self.runtime.block_on(async {
                        maplet.query(&key.to_string()).await
                    });
                    Ok(result.map(|v| to_py_any_f64(py, v).into()))
                }
                PyMapletGenericInner::Vector(maplet) => {
                    let result = self.runtime.block_on(async {
                        maplet.query(&key.to_string()).await
                    });
                    if let Some(vec) = result {
                        // Convert Vec<f64> to Python list (can be converted to numpy array in Python)
                        let list = PyList::new(py, vec)?;
                        Ok(Some(list.into()))
                    } else {
                        Ok(None)
                    }
                }
            }
        })
    }

    fn contains(&self, key: String) -> bool {
        match &self.inner {
            PyMapletGenericInner::Counter(maplet) => {
                self.runtime.block_on(async {
                    maplet.contains(&key).await
                })
            }
            PyMapletGenericInner::MaxU64(maplet) => {
                self.runtime.block_on(async {
                    maplet.contains(&key).await
                })
            }
            PyMapletGenericInner::MaxF64(maplet) => {
                self.runtime.block_on(async {
                    maplet.contains(&key).await
                })
            }
            PyMapletGenericInner::MinU64(maplet) => {
                self.runtime.block_on(async {
                    maplet.contains(&key).await
                })
            }
            PyMapletGenericInner::MinF64(maplet) => {
                self.runtime.block_on(async {
                    maplet.contains(&key).await
                })
            }
            PyMapletGenericInner::Vector(maplet) => {
                self.runtime.block_on(async {
                    maplet.contains(&key).await
                })
            }
        }
    }

    fn len(&self) -> usize {
        match &self.inner {
            PyMapletGenericInner::Counter(maplet) => {
                self.runtime.block_on(async {
                    maplet.len().await
                })
            }
            PyMapletGenericInner::MaxU64(maplet) => {
                self.runtime.block_on(async {
                    maplet.len().await
                })
            }
            PyMapletGenericInner::MaxF64(maplet) => {
                self.runtime.block_on(async {
                    maplet.len().await
                })
            }
            PyMapletGenericInner::MinU64(maplet) => {
                self.runtime.block_on(async {
                    maplet.len().await
                })
            }
            PyMapletGenericInner::MinF64(maplet) => {
                self.runtime.block_on(async {
                    maplet.len().await
                })
            }
            PyMapletGenericInner::Vector(maplet) => {
                self.runtime.block_on(async {
                    maplet.len().await
                })
            }
        }
    }

    fn is_empty(&self) -> bool {
        match &self.inner {
            PyMapletGenericInner::Counter(maplet) => {
                self.runtime.block_on(async {
                    maplet.is_empty().await
                })
            }
            PyMapletGenericInner::MaxU64(maplet) => {
                self.runtime.block_on(async {
                    maplet.is_empty().await
                })
            }
            PyMapletGenericInner::MaxF64(maplet) => {
                self.runtime.block_on(async {
                    maplet.is_empty().await
                })
            }
            PyMapletGenericInner::MinU64(maplet) => {
                self.runtime.block_on(async {
                    maplet.is_empty().await
                })
            }
            PyMapletGenericInner::MinF64(maplet) => {
                self.runtime.block_on(async {
                    maplet.is_empty().await
                })
            }
            PyMapletGenericInner::Vector(maplet) => {
                self.runtime.block_on(async {
                    maplet.is_empty().await
                })
            }
        }
    }

    fn error_rate(&self) -> f64 {
        match &self.inner {
            PyMapletGenericInner::Counter(maplet) => maplet.error_rate(),
            PyMapletGenericInner::MaxU64(maplet) => maplet.error_rate(),
            PyMapletGenericInner::MaxF64(maplet) => maplet.error_rate(),
            PyMapletGenericInner::MinU64(maplet) => maplet.error_rate(),
            PyMapletGenericInner::MinF64(maplet) => maplet.error_rate(),
            PyMapletGenericInner::Vector(maplet) => maplet.error_rate(),
        }
    }

    fn load_factor(&self) -> f64 {
        match &self.inner {
            PyMapletGenericInner::Counter(maplet) => {
                self.runtime.block_on(async {
                    maplet.load_factor().await
                })
            }
            PyMapletGenericInner::MaxU64(maplet) => {
                self.runtime.block_on(async {
                    maplet.load_factor().await
                })
            }
            PyMapletGenericInner::MaxF64(maplet) => {
                self.runtime.block_on(async {
                    maplet.load_factor().await
                })
            }
            PyMapletGenericInner::MinU64(maplet) => {
                self.runtime.block_on(async {
                    maplet.load_factor().await
                })
            }
            PyMapletGenericInner::MinF64(maplet) => {
                self.runtime.block_on(async {
                    maplet.load_factor().await
                })
            }
            PyMapletGenericInner::Vector(maplet) => {
                self.runtime.block_on(async {
                    maplet.load_factor().await
                })
            }
        }
    }

    fn find_slot_for_key(&self, key: &str) -> PyResult<Option<usize>> {
        match &self.inner {
            PyMapletGenericInner::Counter(maplet) => {
                Ok(self.runtime.block_on(async {
                    maplet.find_slot_for_key(&key.to_string()).await
                }))
            }
            PyMapletGenericInner::MaxU64(maplet) => {
                Ok(self.runtime.block_on(async {
                    maplet.find_slot_for_key(&key.to_string()).await
                }))
            }
            PyMapletGenericInner::MaxF64(maplet) => {
                Ok(self.runtime.block_on(async {
                    maplet.find_slot_for_key(&key.to_string()).await
                }))
            }
            PyMapletGenericInner::MinU64(maplet) => {
                Ok(self.runtime.block_on(async {
                    maplet.find_slot_for_key(&key.to_string()).await
                }))
            }
            PyMapletGenericInner::MinF64(maplet) => {
                Ok(self.runtime.block_on(async {
                    maplet.find_slot_for_key(&key.to_string()).await
                }))
            }
            PyMapletGenericInner::Vector(maplet) => {
                Ok(self.runtime.block_on(async {
                    maplet.find_slot_for_key(&key.to_string()).await
                }))
            }
        }
    }

    fn delete(&self, key: &str) -> PyResult<bool> {
        // Query the value first, then delete with that value
        let result = self.runtime.block_on(async {
            match &self.inner {
                PyMapletGenericInner::Counter(maplet) => {
                    if let Some(value) = maplet.query(&key.to_string()).await {
                        maplet.delete(&key.to_string(), &value).await
                    } else {
                        Ok(false)
                    }
                }
                PyMapletGenericInner::MaxU64(maplet) => {
                    if let Some(value) = maplet.query(&key.to_string()).await {
                        maplet.delete(&key.to_string(), &value).await
                    } else {
                        Ok(false)
                    }
                }
                PyMapletGenericInner::MaxF64(maplet) => {
                    if let Some(value) = maplet.query(&key.to_string()).await {
                        maplet.delete(&key.to_string(), &value).await
                    } else {
                        Ok(false)
                    }
                }
                PyMapletGenericInner::MinU64(maplet) => {
                    if let Some(value) = maplet.query(&key.to_string()).await {
                        maplet.delete(&key.to_string(), &value).await
                    } else {
                        Ok(false)
                    }
                }
                PyMapletGenericInner::MinF64(maplet) => {
                    if let Some(value) = maplet.query(&key.to_string()).await {
                        maplet.delete(&key.to_string(), &value).await
                    } else {
                        Ok(false)
                    }
                }
                PyMapletGenericInner::Vector(maplet) => {
                    if let Some(value) = maplet.query(&key.to_string()).await {
                        maplet.delete(&key.to_string(), &value).await
                    } else {
                        Ok(false)
                    }
                }
            }
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(result)
    }

    fn clear(&self) -> PyResult<()> {
        // Maplet doesn't have a native clear() method.
        // For clear functionality, use Engine instead of Maplet.
        // As a workaround, we'll recreate the maplet by clearing the internal state.
        // This is a limitation of the probabilistic data structure design.
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "clear() is not supported for Maplet. Use Engine for clear functionality, or recreate the Maplet."
        ))
    }

    fn stats(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| -> PyResult<PyObject> {
            let stats = PyDict::new(py);
            stats.set_item("item_count", self.len())?;
            stats.set_item("error_rate", self.error_rate())?;
            stats.set_item("false_positive_rate", self.error_rate())?;
            stats.set_item("load_factor", self.load_factor())?;
            // Estimate memory usage
            let memory_usage = self.len() * 12; // ~12 bytes per item
            stats.set_item("memory_usage", memory_usage)?;
            Ok(stats.into())
        })
    }
}

// ============================================================================
// Legacy PyMaplet (backward compatibility)
// ============================================================================

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
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {e}")))?);
        Ok(Self { inner: maplet, runtime })
    }
    
    fn insert(&mut self, key: String, value: u64) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.insert(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        Ok(())
    }
    
    fn query(&self, key: &str) -> PyResult<Option<u64>> {
        Ok(self.runtime.block_on(async {
            self.inner.query(&key.to_string()).await
        }))
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
    
    /// Find the slot for a key (quotient filter feature)
    fn find_slot_for_key(&self, key: &str) -> PyResult<Option<usize>> {
        Ok(self.runtime.block_on(async {
            self.inner.find_slot_for_key(&key.to_string()).await
        }))
    }
}

// ============================================================================
// Engine Classes (unchanged)
// ============================================================================

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
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {e}")))?);
        
        let engine = runtime.block_on(async {
            Engine::new(rust_config).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        
        Ok(Self {
            inner: Arc::new(engine),
            runtime,
        })
    }

    /// Set a key-value pair
    fn set(&self, key: String, value: Vec<u8>) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.set(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
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
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
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
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        
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
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        
        Ok(PyEngineStats::from(stats))
    }

    /// Get memory usage in bytes
    fn memory_usage(&self) -> PyResult<usize> {
        let usage = self.runtime.block_on(async {
            self.inner.memory_usage().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        
        Ok(usage as usize)
    }

    /// Flush pending writes to disk
    fn flush(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.flush().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        Ok(())
    }

    /// Close the engine and cleanup resources
    fn close(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.close().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        Ok(())
    }
    
    /// Find the slot for a key (quotient filter feature)
    fn find_slot_for_key(&self, key: String) -> PyResult<Option<usize>> {
        let result = self.runtime.block_on(async {
            self.inner.find_slot_for_key(&key).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;
        Ok(result)
    }
}

// ============================================================================
// Module Definition
// ============================================================================

/// Python module definition
#[pymodule]
fn mappy_python(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // Operator classes
    m.add_class::<PyCounterOperator>()?;
    m.add_class::<PyMaxOperator>()?;
    m.add_class::<PyMinOperator>()?;
    m.add_class::<PyVectorOperator>()?;
    
    // Maplet classes
    m.add_class::<PyMaplet>()?; // Legacy
    m.add_class::<PyMapletGeneric>()?; // New operator-based
    
    // Engine classes
    m.add_class::<PyEngine>()?;
    m.add_class::<PyEngineConfig>()?;
    m.add_class::<PyEngineStats>()?;
    
    // Add type aliases for backward compatibility
    // Use Python::with_gil to get the classes after they're added
    let py_maplet_generic = m.getattr("PyMapletGeneric")?;
    m.add("Maplet", py_maplet_generic)?;
    let py_counter_op = m.getattr("PyCounterOperator")?;
    m.add("CounterOperator", py_counter_op)?;
    let py_max_op = m.getattr("PyMaxOperator")?;
    m.add("MaxOperator", py_max_op)?;
    let py_min_op = m.getattr("PyMinOperator")?;
    m.add("MinOperator", py_min_op)?;
    let py_vector_op = m.getattr("PyVectorOperator")?;
    m.add("VectorOperator", py_vector_op)?;
    
    Ok(())
}
