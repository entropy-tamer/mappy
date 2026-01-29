//! Python bindings for mappy operators (VectorConcatOperator, ByteArrayOperator)

use pyo3::prelude::*;
use mappy_core::{ByteArrayOperator, VectorConcatOperator};

/// Python wrapper for VectorConcatOperator
#[pyclass]
#[derive(Clone, Copy)]
pub struct PyVectorConcatOperator {
    #[allow(dead_code)] // Used by PyO3 for type conversion
    inner: VectorConcatOperator,
}

#[pymethods]
impl PyVectorConcatOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: VectorConcatOperator,
        }
    }
}

/// Python wrapper for ByteArrayOperator
#[pyclass]
#[derive(Clone, Copy)]
pub struct PyByteArrayOperator {
    #[allow(dead_code)] // Used by PyO3 for type conversion
    inner: ByteArrayOperator,
}

#[pymethods]
impl PyByteArrayOperator {
    #[new]
    fn new() -> Self {
        Self {
            inner: ByteArrayOperator,
        }
    }
}

