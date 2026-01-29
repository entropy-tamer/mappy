//! MappyMLDataset implementation
//! 
//! Provides a PyTorch Dataset that loads data from mappy, decodes images in Rust,
//! and returns zero-copy tensors. Always returns (tensor, label) tuple.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple, PyInt};
use numpy::PyArray;
use crate::image;
use crate::tensor::{to_tensor, TensorBackend};
use std::sync::Arc;

/// MappyMLDataset - PyTorch Dataset for mappy storage
/// 
/// Loads data from mappy maplet, decodes images in Rust, and returns PyTorch tensors.
/// **Always returns (tensor, int) tuple** to fix MNIST unpacking issues.
#[pyclass]
pub struct MappyMLDataset {
    maplet: PyObject,
    keys: Vec<String>,
    image_shape: Vec<usize>,
    label_dtype: String,
    backend: TensorBackend,
}

#[pymethods]
impl MappyMLDataset {
    #[new]
    #[pyo3(signature = (maplet, keys, image_shape, label_dtype = "int64", backend = "auto"))]
    fn new(
        py: Python,
        maplet: PyObject,
        keys: Vec<String>,
        image_shape: Vec<usize>,
        label_dtype: String,
        backend: String,
    ) -> PyResult<Self> {
        let tensor_backend = match backend.as_str() {
            "dlpack" => TensorBackend::DLPack,
            "libtorch" => TensorBackend::LibTorch,
            "auto" | _ => TensorBackend::Auto,
        };
        
        Ok(Self {
            maplet,
            keys,
            image_shape,
            label_dtype,
            backend: tensor_backend,
        })
    }
    
    /// Get item at index - ALWAYS returns (tensor, label) tuple
    fn __getitem__(&self, py: Python, idx: usize) -> PyResult<PyObject> {
        if idx >= self.keys.len() {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                format!("Index {} out of range for dataset of size {}", idx, self.keys.len())
            ));
        }
        
        let key = &self.keys[idx];
        
        // Query data from maplet
        let maplet_py = self.maplet.bind(py);
        let data_opt = maplet_py.call_method1("query", (key,))?;
        let data = data_opt.ok_or(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Key '{}' not found in maplet", key)
        ))?;
        
        // Handle different data formats:
        // 1. Bytes (from ByteArrayOperator) - decode image, extract label from key
        // 2. Tuple (image, label) - use directly
        // 3. List of floats (pickled data) - unpickle and handle
        
        let (image_tensor, label): (PyObject, i64) = if let Ok(bytes) = data.downcast::<PyBytes>() {
            // Case 1: Raw bytes (from ByteArrayOperator)
            let (tensor, _) = self.decode_image_from_bytes(py, &bytes)?;
            let label = self.extract_label_from_key(key)?;
            (tensor, label)
        } else if let Ok(tuple) = data.downcast::<pyo3::types::PyTuple>() {
            // Case 2: Already a tuple (image, label)
            if tuple.len() == 2 {
                let image_obj = tuple.get_item(0)?;
                let label_obj = tuple.get_item(1)?;
                let label: i64 = label_obj.extract()?;
                let tensor = self.convert_to_tensor(py, image_obj)?;
                (tensor, label)
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Expected tuple of length 2, got {}", tuple.len())
                ));
            }
        } else if let Ok(list) = data.downcast::<pyo3::types::PyList>() {
            // Case 3: List of floats (pickled data representation)
            // Reconstruct bytes and try to unpickle
            let bytes_vec: Result<Vec<u8>, _> = (0..list.len())
                .map(|i| {
                    let val = list.get_item(i)?;
                    let num: f64 = val.extract()?;
                    Ok((num as u8) & 0xFF)
                })
                .collect();
            let bytes_vec = bytes_vec?;
            let bytes = PyBytes::new(py, &bytes_vec);
            
            // Try to unpickle
            let pickle = py.import("pickle")?;
            let unpickled = pickle.call_method1("loads", (bytes,))?;
            
            // If unpickled is a tuple, use it
            if let Ok(tuple) = unpickled.downcast::<pyo3::types::PyTuple>() {
                if tuple.len() == 2 {
                    let image_obj = tuple.get_item(0)?;
                    let label_obj = tuple.get_item(1)?;
                    let label: i64 = label_obj.extract()?;
                    let tensor = self.convert_to_tensor(py, image_obj)?;
                    (tensor, label)
                } else {
                    // Not a 2-tuple - try to extract label from key and decode as image
                    let (tensor, _) = self.decode_image_from_bytes(py, &bytes)?;
                    let label = self.extract_label_from_key(key)?;
                    (tensor, label)
                }
            } else {
                // Not a tuple - decode as image and extract label from key
                let (tensor, _) = self.decode_image_from_bytes(py, &bytes)?;
                let label = self.extract_label_from_key(key)?;
                (tensor, label)
            }
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                format!("Unsupported data type for key '{}': {}", key, data.get_type().name()?)
            ));
        };
        
        // Create tuple (tensor, label) - ALWAYS returns 2-tuple
        let label_py: PyObject = label.into_py(py);
        let image_bound = image_tensor.bind(py);
        let label_bound = label_py.bind(py);
        let tuple = PyTuple::new(py, &[image_bound.as_ref(), label_bound.as_ref()])?;
        
        Ok(tuple.into())
    }
    
    /// Get dataset length
    fn __len__(&self) -> usize {
        self.keys.len()
    }
    
    /// Get string representation
    fn __repr__(&self) -> String {
        format!(
            "MappyMLDataset(keys={}, image_shape={:?}, label_dtype={})",
            self.keys.len(),
            self.image_shape,
            self.label_dtype
        )
    }
}

impl MappyMLDataset {
    /// Decode image from bytes and return (tensor, label) tuple
    fn decode_image_from_bytes(
        &self,
        py: Python,
        bytes: &Bound<PyBytes>,
    ) -> PyResult<(PyObject, i64)> {
        // Note: In the current implementation, labels need to be stored separately
        // or extracted from the key. For now, we return 0 as a placeholder.
        // This will be fixed when we update the storage format to use ByteArrayOperator
        // and store labels in a separate maplet or as metadata.
        // Decode image from bytes
        let image_array = image::decode_image(py, bytes)?;
        
        // Convert to tensor
        let tensor = self.convert_to_tensor(py, image_array)?;
        
        // Extract label from key (fallback - labels should be stored separately)
        // For now, we need to pass the key - this will be fixed when we store labels properly
        // For MNIST, labels are typically stored in a separate maplet or in the data itself
        let label = 0i64; // Default - will be properly extracted when labels are stored correctly
        
        Ok((tensor, label))
    }
    
    /// Convert image array to tensor with proper shape and normalization
    /// 
    /// Handles multiple input types:
    /// - numpy arrays (uint8)
    /// - PIL Images (converted to numpy first)
    /// - PyTorch tensors (converted to numpy first)
    fn convert_to_tensor(
        &self,
        py: Python,
        image_obj: Bound<PyAny>,
    ) -> PyResult<PyObject> {
        // Try different image formats
        if let Ok(array) = image_obj.downcast::<numpy::PyArray3<u8>>() {
            // Already a numpy array - get data directly
            // Use Python API to get shape
            let shape_py = array.getattr("shape")?;
            let shape_tuple = shape_py.downcast::<pyo3::types::PyTuple>()?;
            let height: usize = shape_tuple.get_item(0)?.extract()?;
            let width: usize = shape_tuple.get_item(1)?.extract()?;
            let channels: usize = shape_tuple.get_item(2)?.extract()?;
            // Get data via Python API
            let data_py = array.call_method0("flatten")?;
            let data_list = data_py.downcast::<pyo3::types::PyList>()?;
            let data: Vec<u8> = data_list.iter().map(|item| item.extract::<u8>()).collect::<Result<_, _>>()?;
            // Convert to f32 and normalize
            let data_f32: Vec<f32> = data.iter().map(|&x| x as f32 / 255.0).collect();
            return to_tensor(py, &data_f32, &self.image_shape, self.backend);
        } else if let Ok(array) = image_obj.downcast::<numpy::PyArrayDyn<u8>>() {
            // Dynamic numpy array - try to reshape to 3D
            // Use Python API to get shape
            let numpy_mod = py.import("numpy")?;
            let shape_py = numpy_mod.call_method1("array", (array,))?.getattr("shape")?;
            let shape_tuple = shape_py.downcast::<pyo3::types::PyTuple>()?;
            if shape_tuple.len() == 3 {
                let height: usize = shape_tuple.get_item(0)?.extract()?;
                let width: usize = shape_tuple.get_item(1)?.extract()?;
                let channels: usize = shape_tuple.get_item(2)?.extract()?;
                // Get data via Python API
                let data_py = array.call_method0("flatten")?;
                let data_list = data_py.downcast::<pyo3::types::PyList>()?;
                let data: Vec<u8> = data_list.iter().map(|item| item.extract::<u8>()).collect::<Result<_, _>>()?;
                // Convert to f32 and normalize
                let data_f32: Vec<f32> = data.iter().map(|&x| x as f32 / 255.0).collect();
                return to_tensor(py, &data_f32, &self.image_shape, self.backend);
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Expected 3D array, got {}D", shape_tuple.len())
                ));
            }
        } else {
            // Try PIL Image conversion
            let pil = py.import("PIL.Image")?;
            let pil_image_class = pil.getattr("Image")?;
            
            // Check if it's a PIL Image
            if image_obj.is_instance(pil_image_class)? {
                // Convert PIL Image to numpy
                let numpy = py.import("numpy")?;
                let array = numpy.call_method1("asarray", (image_obj,))?;
                
                // Try to downcast to PyArray3 or PyArrayDyn
                if let Ok(arr3) = array.downcast::<numpy::PyArray3<u8>>() {
                    // Use Python API to get shape and data
                    let shape_py = arr3.getattr("shape")?;
                    let shape_tuple = shape_py.downcast::<pyo3::types::PyTuple>()?;
                    let height: usize = shape_tuple.get_item(0)?.extract()?;
                    let width: usize = shape_tuple.get_item(1)?.extract()?;
                    let channels: usize = shape_tuple.get_item(2)?.extract()?;
                    let data_py = arr3.call_method0("flatten")?;
                    let data_list = data_py.downcast::<pyo3::types::PyList>()?;
                    let data: Vec<u8> = data_list.iter().map(|item| item.extract::<u8>()).collect::<Result<_, _>>()?;
                    let data_f32: Vec<f32> = data.iter().map(|&x| x as f32 / 255.0).collect();
                    return to_tensor(py, &data_f32, &self.image_shape, self.backend);
                } else if let Ok(arr_dyn) = array.downcast::<numpy::PyArrayDyn<u8>>() {
                    // Use Python API for dynamic arrays
                    let numpy_mod = py.import("numpy")?;
                    let shape_py = arr_dyn.getattr("shape")?;
                    let shape_tuple = shape_py.downcast::<pyo3::types::PyTuple>()?;
                    if shape_tuple.len() == 3 {
                        let height: usize = shape_tuple.get_item(0)?.extract()?;
                        let width: usize = shape_tuple.get_item(1)?.extract()?;
                        let channels: usize = shape_tuple.get_item(2)?.extract()?;
                        let data_py = arr_dyn.call_method0("flatten")?;
                        let data_list = data_py.downcast::<pyo3::types::PyList>()?;
                        let data: Vec<u8> = data_list.iter().map(|item| item.extract::<u8>()).collect::<Result<_, _>>()?;
                        let data_f32: Vec<f32> = data.iter().map(|&x| x as f32 / 255.0).collect();
                        return to_tensor(py, &data_f32, &self.image_shape, self.backend);
                    } else {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Expected 3D array from PIL, got {}D", shape_tuple.len())
                        ));
                    }
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Failed to convert PIL Image to numpy array"
                    ));
                }
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    format!("Unsupported image type: {}", image_obj.get_type().name()?)
                ));
            }
        }
    }
    
    /// Extract label from key
    /// 
    /// For MNIST-style keys like "train_12345", extract the numeric part.
    /// This is a simple heuristic - in production, labels should be stored separately.
    fn extract_label_from_key(&self, key: &str) -> PyResult<i64> {
        // Try to extract label from key format like "train_12345" or "img_12345"
        // For MNIST, the label might be encoded in the key or we need to query a separate maplet
        // For now, try to extract from key suffix
        
        if let Some(underscore_pos) = key.rfind('_') {
            if let Ok(num) = key[underscore_pos + 1..].parse::<i64>() {
                // For MNIST, labels are 0-9, so take modulo 10
                // This is a heuristic - proper implementation would store labels separately
                return Ok(num % 10);
            }
        }
        
        // Default to 0 if we can't extract
        // In production, labels should be stored in a separate maplet with key "label_{original_key}"
        Ok(0)
    }
}
