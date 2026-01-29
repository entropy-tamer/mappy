//! Tensor export module (DLPack and libtorch)
//! 
//! Provides zero-copy tensor export from Rust to Python via DLPack and libtorch

use pyo3::prelude::*;

pub mod dlpack;
pub mod libtorch;

pub use dlpack::{to_dlpack_tensor, create_dlpack_view};
pub use libtorch::{to_libtorch_tensor, create_libtorch_view};

/// Tensor export preference
#[derive(Debug, Clone, Copy)]
pub enum TensorBackend {
    /// Use DLPack (framework-agnostic)
    DLPack,
    /// Use libtorch/PyTorch C API
    LibTorch,
    /// Automatic selection (try DLPack first, fallback to libtorch)
    Auto,
}

/// Convert Rust data to PyTorch tensor with automatic backend selection
pub fn to_tensor(
    py: Python,
    data: &[f32],
    shape: &[usize],
    backend: TensorBackend,
) -> PyResult<PyObject> {
    match backend {
        TensorBackend::DLPack => to_dlpack_tensor(py, data, shape),
        TensorBackend::LibTorch => to_libtorch_tensor(py, data, shape, false),
        TensorBackend::Auto => {
            // Try DLPack first, fallback to libtorch
            to_dlpack_tensor(py, data, shape)
                .or_else(|_| to_libtorch_tensor(py, data, shape, false))
        }
    }
}

/// Create a zero-copy tensor view from Rust-allocated memory
pub fn create_tensor_view(
    py: Python,
    data: &mut [f32],
    shape: &[usize],
    backend: TensorBackend,
) -> PyResult<PyObject> {
    match backend {
        TensorBackend::DLPack => create_dlpack_view(py, data, shape),
        TensorBackend::LibTorch => create_libtorch_view(py, data, shape),
        TensorBackend::Auto => {
            create_dlpack_view(py, data, shape)
                .or_else(|_| create_libtorch_view(py, data, shape))
        }
    }
}
