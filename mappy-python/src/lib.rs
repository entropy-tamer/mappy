//! # Mappy Python Bindings
//! 
//! Python bindings for mappy maplet data structures using PyO3.

use pyo3::prelude::*;
use mappy_core::{Maplet, CounterOperator};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Python wrapper for Maplet
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

/// Python module definition
#[pymodule]
fn mappy_python(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyMaplet>()?;
    Ok(())
}
