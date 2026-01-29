//! # Python Mappy ML
//!
//! High-performance Rust-native ML data pipeline for mappy with:
//! - Image decoding (JPEG/PNG/raw bytes)
//! - Zero-copy tensor export (DLPack/libtorch)
//! - Async GPU transfer
//! - ML-optimized operators

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)] // Acceptable for Python type conversions

use pyo3::prelude::*;

mod operators;
mod image;
mod tensor;
mod pinned_memory;
mod gpu;
mod dataset;

// Re-export public APIs
pub use operators::*;

/// Python module initialization
#[pymodule]
fn python_mappy_ml(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // Operators
    m.add_class::<operators::PyVectorConcatOperator>()?;
    m.add_class::<operators::PyByteArrayOperator>()?;
    
    // Image processing
    m.add_function(wrap_pyfunction!(image::decode_image, m)?)?;
    m.add_function(wrap_pyfunction!(image::decode_image_batch, m)?)?;
    m.add_function(wrap_pyfunction!(image::preprocess_image, m)?)?;
    
    // Dataset
    m.add_class::<dataset::MappyMLDataset>()?;
    
    Ok(())
}

