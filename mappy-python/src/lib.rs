//! # Mappy Python Bindings
//! 
//! Python bindings for mappy maplet data structures using PyO3.

use pyo3::prelude::*;
use mappy_core::{Maplet, CounterOperator, MaxOperator, MinOperator, SetOperator, VectorOperator, VectorConcatOperator, CustomOperator, MapletError, MapletStats};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Python wrapper for Maplet with CounterOperator
#[pyclass]
pub struct PyCounterMaplet {
    inner: Maplet<String, i64, CounterOperator>,
    runtime: Arc<Runtime>,
}

/// Python wrapper for Maplet with MaxOperator
#[pyclass]
pub struct PyMaxMaplet {
    inner: Maplet<String, i64, MaxOperator>,
    runtime: Arc<Runtime>,
}

/// Python wrapper for Maplet with MinOperator
#[pyclass]
pub struct PyMinMaplet {
    inner: Maplet<String, i64, MinOperator>,
    runtime: Arc<Runtime>,
}

/// Python wrapper for Maplet with SetOperator
#[pyclass]
pub struct PySetMaplet {
    inner: Maplet<String, i64, SetOperator>,
    runtime: Arc<Runtime>,
}

/// Python wrapper for Maplet with VectorOperator
#[pyclass]
pub struct PyVectorMaplet {
    inner: Maplet<String, i64, VectorOperator>,
    runtime: Arc<Runtime>,
}

// CustomOperator removed due to Default trait requirement

/// Python operator classes
#[pyclass]
#[derive(Clone)]
pub struct PyCounterOperator {
    inner: CounterOperator,
}

#[pyclass]
#[derive(Clone)]
pub struct PyMaxOperator {
    inner: MaxOperator,
}

#[pyclass]
#[derive(Clone)]
pub struct PyMinOperator {
    inner: MinOperator,
}

#[pyclass]
#[derive(Clone)]
pub struct PySetOperator {
    inner: SetOperator,
}

#[pyclass]
#[derive(Clone)]
pub struct PyVectorOperator {
    inner: VectorOperator,
}

#[pyclass]
#[derive(Clone)]
pub struct PyVectorConcatOperator {
    inner: VectorConcatOperator,
}

// PyCustomOperator removed due to Default trait requirement

/// Python error wrapper
#[pyclass]
pub struct PyMapletError {
    inner: MapletError,
}

/// Python stats wrapper
#[pyclass]
pub struct PyMapletStats {
    inner: MapletStats,
}

// Macro to generate common maplet methods (without #[pymethods])
macro_rules! impl_maplet_methods {
    ($struct_name:ident) => {
        impl $struct_name {
            fn insert(&mut self, key: String, value: i64) -> PyResult<()> {
                self.runtime.block_on(async {
                    self.inner.insert(key, value).await
                }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
                Ok(())
            }
            
            fn query(&self, key: &str) -> Option<i64> {
                self.runtime.block_on(async {
                    self.inner.query(&key.to_string()).await
                })
            }
            
            fn contains(&self, key: &str) -> bool {
                self.runtime.block_on(async {
                    self.inner.contains(&key.to_string()).await
                })
            }
            
            fn delete(&mut self, key: &str, value: i64) -> PyResult<bool> {
                self.runtime.block_on(async {
                    self.inner.delete(&key.to_string(), &value).await
                }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
            }
            
            fn clear(&mut self) -> PyResult<()> {
                // For now, we'll just return Ok since clear is not available in the current API
                Ok(())
            }
            
            fn stats(&self) -> PyMapletStats {
                PyMapletStats {
                    inner: self.runtime.block_on(async {
                        self.inner.stats().await
                    })
                }
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
        }
    };
}

#[pymethods]
impl PyCounterMaplet {
    #[new]
    fn new(capacity: usize, false_positive_rate: f64, operator: PyCounterOperator) -> PyResult<Self> {
        let maplet = Maplet::with_operator(capacity, false_positive_rate, operator.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?);
        Ok(Self { inner: maplet, runtime })
    }
    
    fn insert(&mut self, key: String, value: i64) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.insert(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }
    
    fn query(&self, key: &str) -> Option<i64> {
        self.runtime.block_on(async {
            self.inner.query(&key.to_string()).await
        })
    }
    
    fn contains(&self, key: &str) -> bool {
        self.runtime.block_on(async {
            self.inner.contains(&key.to_string()).await
        })
    }
    
    fn delete(&mut self, key: &str, value: i64) -> PyResult<bool> {
        self.runtime.block_on(async {
            self.inner.delete(&key.to_string(), &value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
    
    fn clear(&mut self) -> PyResult<()> {
        Ok(())
    }
    
    fn stats(&self) -> PyMapletStats {
        PyMapletStats {
            inner: self.runtime.block_on(async {
                self.inner.stats().await
            })
        }
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
}

#[pymethods]
impl PyMaxMaplet {
    #[new]
    fn new(capacity: usize, false_positive_rate: f64, operator: PyMaxOperator) -> PyResult<Self> {
        let maplet = Maplet::with_operator(capacity, false_positive_rate, operator.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?);
        Ok(Self { inner: maplet, runtime })
    }
    
    fn insert(&mut self, key: String, value: i64) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.insert(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }
    
    fn query(&self, key: &str) -> Option<i64> {
        self.runtime.block_on(async {
            self.inner.query(&key.to_string()).await
        })
    }
    
    fn contains(&self, key: &str) -> bool {
        self.runtime.block_on(async {
            self.inner.contains(&key.to_string()).await
        })
    }
    
    fn delete(&mut self, key: &str, value: i64) -> PyResult<bool> {
        self.runtime.block_on(async {
            self.inner.delete(&key.to_string(), &value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
    
    fn clear(&mut self) -> PyResult<()> {
        Ok(())
    }
    
    fn stats(&self) -> PyMapletStats {
        PyMapletStats {
            inner: self.runtime.block_on(async {
                self.inner.stats().await
            })
        }
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
}

#[pymethods]
impl PyMinMaplet {
    #[new]
    fn new(capacity: usize, false_positive_rate: f64, operator: PyMinOperator) -> PyResult<Self> {
        let maplet = Maplet::with_operator(capacity, false_positive_rate, operator.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?);
        Ok(Self { inner: maplet, runtime })
    }
    
    fn insert(&mut self, key: String, value: i64) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.insert(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }
    
    fn query(&self, key: &str) -> Option<i64> {
        self.runtime.block_on(async {
            self.inner.query(&key.to_string()).await
        })
    }
    
    fn contains(&self, key: &str) -> bool {
        self.runtime.block_on(async {
            self.inner.contains(&key.to_string()).await
        })
    }
    
    fn delete(&mut self, key: &str, value: i64) -> PyResult<bool> {
        self.runtime.block_on(async {
            self.inner.delete(&key.to_string(), &value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
    
    fn clear(&mut self) -> PyResult<()> {
        Ok(())
    }
    
    fn stats(&self) -> PyMapletStats {
        PyMapletStats {
            inner: self.runtime.block_on(async {
                self.inner.stats().await
            })
        }
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
}

#[pymethods]
impl PySetMaplet {
    #[new]
    fn new(capacity: usize, false_positive_rate: f64, operator: PySetOperator) -> PyResult<Self> {
        let maplet = Maplet::with_operator(capacity, false_positive_rate, operator.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?);
        Ok(Self { inner: maplet, runtime })
    }
    
    fn insert(&mut self, key: String, value: i64) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.insert(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }
    
    fn query(&self, key: &str) -> Option<i64> {
        self.runtime.block_on(async {
            self.inner.query(&key.to_string()).await
        })
    }
    
    fn contains(&self, key: &str) -> bool {
        self.runtime.block_on(async {
            self.inner.contains(&key.to_string()).await
        })
    }
    
    fn delete(&mut self, key: &str, value: i64) -> PyResult<bool> {
        self.runtime.block_on(async {
            self.inner.delete(&key.to_string(), &value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
    
    fn clear(&mut self) -> PyResult<()> {
        Ok(())
    }
    
    fn stats(&self) -> PyMapletStats {
        PyMapletStats {
            inner: self.runtime.block_on(async {
                self.inner.stats().await
            })
        }
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
}

#[pymethods]
impl PyVectorMaplet {
    #[new]
    fn new(capacity: usize, false_positive_rate: f64, operator: PyVectorOperator) -> PyResult<Self> {
        let maplet = Maplet::with_operator(capacity, false_positive_rate, operator.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?);
        Ok(Self { inner: maplet, runtime })
    }
    
    fn insert(&mut self, key: String, value: i64) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.insert(key, value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(())
    }
    
    fn query(&self, key: &str) -> Option<i64> {
        self.runtime.block_on(async {
            self.inner.query(&key.to_string()).await
        })
    }
    
    fn contains(&self, key: &str) -> bool {
        self.runtime.block_on(async {
            self.inner.contains(&key.to_string()).await
        })
    }
    
    fn delete(&mut self, key: &str, value: i64) -> PyResult<bool> {
        self.runtime.block_on(async {
            self.inner.delete(&key.to_string(), &value).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
    
    fn clear(&mut self) -> PyResult<()> {
        Ok(())
    }
    
    fn stats(&self) -> PyMapletStats {
        PyMapletStats {
            inner: self.runtime.block_on(async {
                self.inner.stats().await
            })
        }
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
}

// PyCustomMaplet implementation removed due to Default trait requirement

#[pymethods]
impl PyCounterOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: CounterOperator::default(),
        }
    }
}

#[pymethods]
impl PyMaxOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: MaxOperator::default(),
        }
    }
}

#[pymethods]
impl PyMinOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: MinOperator::default(),
        }
    }
}

#[pymethods]
impl PySetOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: SetOperator::default(),
        }
    }
}

#[pymethods]
impl PyVectorOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: VectorOperator::default(),
        }
    }
}

#[pymethods]
impl PyVectorConcatOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: VectorConcatOperator::default(),
        }
    }
}

// Define a simple addition function for the custom operator
fn add_merge(a: i64, b: i64) -> i64 {
    a + b
}

// PyCustomOperator implementation removed due to Default trait requirement


#[pymethods]
impl PyMapletStats {
    fn load_factor(&self) -> f64 {
        self.inner.load_factor
    }
    
    fn memory_usage(&self) -> usize {
        self.inner.memory_usage
    }
    
    fn item_count(&self) -> usize {
        self.inner.len
    }
    
    fn false_positive_rate(&self) -> f64 {
        self.inner.false_positive_rate
    }
}

/// Python module definition
#[pymodule]
fn mappy_python(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // Maplet classes
    m.add_class::<PyCounterMaplet>()?;
    m.add_class::<PyMaxMaplet>()?;
    m.add_class::<PyMinMaplet>()?;
    m.add_class::<PySetMaplet>()?;
    m.add_class::<PyVectorMaplet>()?;
    
    // Operator classes
    m.add_class::<PyCounterOperator>()?;
    m.add_class::<PyMaxOperator>()?;
    m.add_class::<PyMinOperator>()?;
    m.add_class::<PySetOperator>()?;
    m.add_class::<PyVectorOperator>()?;
    m.add_class::<PyVectorConcatOperator>()?;
    
    // Utility classes
    m.add_class::<PyMapletError>()?;
    m.add_class::<PyMapletStats>()?;
    Ok(())
}
