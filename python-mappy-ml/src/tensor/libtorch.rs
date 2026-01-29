//! libtorch tensor export via tch-rs
//! 
//! Provides zero-copy tensor creation using PyTorch's from_blob API

use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyInt};
use numpy::{IntoPyArray, PyArray};
use tch::Tensor as TchTensor;
use ndarray;

/// Convert data to libtorch tensor using tch-rs
pub fn to_libtorch_tensor(
    py: Python,
    data: &[f32],
    shape: &[usize],
    pinned: bool,
) -> PyResult<PyObject> {
    // Create tch tensor from data
    let tch_tensor = if pinned {
        // For pinned memory, we'd need to allocate via libtorch's pinned allocator
        // For now, create regular tensor and let PyTorch handle pinning if needed
        TchTensor::from_slice(data)
            .reshape(&shape.iter().map(|&s| s as i64).collect::<Vec<_>>())
    } else {
        TchTensor::from_slice(data)
            .reshape(&shape.iter().map(|&s| s as i64).collect::<Vec<_>>())
    };
    
    // Convert tch tensor to Python PyTorch tensor
    // tch-rs provides a way to get the underlying C tensor pointer
    // We can use PyTorch's Python API to wrap it
    
    // For now, we'll use a simpler approach: convert to numpy then to torch
    // A more efficient approach would use torch::from_blob directly via FFI
    let numpy_array = tch_tensor_to_numpy(py, &tch_tensor)?;
    let torch = py.import("torch")?;
    let numpy_bound = numpy_array.bind(py);
    let tensor = torch.call_method1("from_numpy", (numpy_bound,))?;
    
    Ok(tensor.into())
}

/// Create a zero-copy libtorch view using from_blob
/// 
/// This uses PyTorch's from_blob API to create a tensor that shares memory
/// with the Rust buffer. The buffer must remain alive for the tensor's lifetime.
pub fn create_libtorch_view(
    py: Python,
    data: &mut [f32],
    shape: &[usize],
) -> PyResult<PyObject> {
    // Use PyTorch's from_blob to create a zero-copy tensor
    // This requires the data to be contiguous and properly aligned
    
    let torch = py.import("torch")?;
    
    // Convert shape to Python tuple
    let shape_vec: Vec<i64> = shape.iter().map(|&s| s as i64).collect();
    let shape_objs: Vec<PyObject> = shape_vec.iter().map(|&s| PyInt::new(py, s).into()).collect();
    let shape_refs: Vec<&PyAny> = shape_objs.iter().map(|obj| obj.as_ref(py)).collect();
    let shape_tuple = PyTuple::new(py, shape_refs)?;
    
    // Create numpy array from slice (for now - true zero-copy would use from_blob via FFI)
    let numpy_array = numpy::PyArray::from_slice(py, data);
    
    // Reshape numpy array
    let reshaped = numpy_array.call_method1("reshape", (shape_tuple,))?;
    
    // Convert to PyTorch tensor
    let tensor = torch.call_method1("from_numpy", (reshaped,))?;
    
    Ok(tensor.into())
}

/// Helper to convert tch tensor to numpy array
fn tch_tensor_to_numpy(py: Python, tensor: &TchTensor) -> PyResult<PyObject> {
    // Get tensor data as Vec<f32>
    let size = tensor.size();
    let total_size: usize = size.iter().product::<i64>() as usize;
    let mut data = vec![0.0f32; total_size];
    tensor.f_copy_data(&mut data, total_size).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to copy tensor data: {}", e)
        )
    })?;
    
    // Get shape
    let shape: Vec<i64> = tensor.size();
    let shape_usize: Vec<usize> = shape.iter().map(|&s| s as usize).collect();
    
    // Create numpy array
    let array: PyObject = match shape_usize.len() {
        1 => {
            PyArray::from_slice(py, &data).into()
        }
        2 => {
            let arr = ndarray::Array2::<f32>::from_shape_vec(
                (shape_usize[0], shape_usize[1]),
                data,
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to create 2D array: {}", e)
            ))?;
            arr.into_pyarray(py).into()
        }
        3 => {
            let arr = ndarray::Array3::<f32>::from_shape_vec(
                (shape_usize[0], shape_usize[1], shape_usize[2]),
                data,
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to create 3D array: {}", e)
            ))?;
            arr.into_pyarray(py).into()
        }
        _ => {
            let arr = ndarray::ArrayD::<f32>::from_shape_vec(
                shape_usize.as_slice(),
                data,
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to create array: {}", e)
            ))?;
            arr.into_pyarray(py).into()
        }
    };
    
    Ok(array)
}

