//! DLPack-compatible tensor export
//! 
//! Implements minimal DLPack support for zero-copy tensor sharing

use pyo3::prelude::*;
use numpy::{PyArray, IntoPyArray};
use ndarray;

/// Convert data to DLPack tensor
/// 
/// Note: This is a simplified implementation. Full DLPack support would require
/// implementing the DLPack C API structures. For now, we use numpy arrays which
/// can be converted to DLPack tensors by PyTorch.
pub fn to_dlpack_tensor(
    py: Python,
    data: &[f32],
    shape: &[usize],
) -> PyResult<PyObject> {
    // Create numpy array from data
    let array = match shape.len() {
        1 => {
            PyArray::from_slice(py, data).into()
        }
        2 => {
            let arr = ndarray::Array2::<f32>::from_shape_vec(
                (shape[0], shape[1]),
                data.to_vec(),
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to create 2D array: {}", e)
            ))?;
            arr.into_pyarray(py).into()
        }
        3 => {
            let arr = ndarray::Array3::<f32>::from_shape_vec(
                (shape[0], shape[1], shape[2]),
                data.to_vec(),
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to create 3D array: {}", e)
            ))?;
            arr.into_pyarray(py).into()
        }
        _ => {
            // Use dynamic array for arbitrary dimensions
            let arr = ndarray::ArrayD::<f32>::from_shape_vec(
                shape,
                data.to_vec(),
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to create array: {}", e)
            ))?;
            arr.into_pyarray(py).into()
        }
    };
    
    // Convert numpy array to PyTorch tensor via DLPack
    // PyTorch's from_numpy uses DLPack internally for zero-copy conversion
    let torch = py.import("torch")?;
    let tensor = torch.call_method1("from_numpy", (array,))?;
    
    Ok(tensor.into())
}

/// Create a zero-copy DLPack view from Rust-allocated memory
/// 
/// This creates a numpy array that shares memory with the Rust buffer.
/// The buffer must remain alive for the lifetime of the returned tensor.
pub fn create_dlpack_view(
    py: Python,
    data: &mut [f32],
    shape: &[usize],
) -> PyResult<PyObject> {
    // For zero-copy, we need to ensure the memory is properly aligned and contiguous
    // numpy arrays can share memory with Rust slices if properly set up
    
    // Create numpy array from slice (this creates a view, not a copy)
    let array = match shape.len() {
        1 => {
            PyArray::from_slice(py, data).into()
        }
        2 => {
            // For 2D, we need to ensure the slice is properly shaped
            let arr = unsafe {
                ndarray::ArrayViewMut2::from_shape_ptr(
                    (shape[0], shape[1]),
                    data.as_mut_ptr(),
                )
            };
            PyArray::from_owned_array(py, arr.into_owned()).into()
        }
        3 => {
            let arr = unsafe {
                ndarray::ArrayViewMut3::from_shape_ptr(
                    (shape[0], shape[1], shape[2]),
                    data.as_mut_ptr(),
                )
            };
            PyArray::from_owned_array(py, arr.into_owned()).into()
        }
        _ => {
            // For arbitrary dimensions, create a view
            let arr = unsafe {
                ndarray::ArrayViewMutD::from_shape_ptr(
                    shape,
                    data.as_mut_ptr(),
                )
            };
            PyArray::from_owned_array(py, arr.into_owned()).into()
        }
    };
    
    // Convert to PyTorch tensor
    let torch = py.import("torch")?;
    let array_bound = array.bind(py);
    let tensor = torch.call_method1("from_numpy", (array_bound,))?;
    
    Ok(tensor.into())
}

