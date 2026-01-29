//! Image decoding and preprocessing module
//! 
//! Provides Rust-native image decoding (JPEG/PNG/raw bytes) with multi-threaded support
//! and preprocessing capabilities.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};
use numpy::{IntoPyArray, PyArray3, PyArrayDyn};
use image::{DynamicImage, ImageFormat, RgbImage, GrayImage};
use rayon::prelude::*;
use ndarray;

/// Decode a single image from bytes (JPEG, PNG, or raw bytes)
#[pyfunction]
pub fn decode_image(py: Python, bytes: &Bound<PyBytes>) -> PyResult<PyObject> {
    let data = bytes.as_bytes();
    
    // Try to detect format and decode
    let img = image::load_from_memory_with_format(data, ImageFormat::Png)
        .or_else(|_| image::load_from_memory_with_format(data, ImageFormat::Jpeg))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to decode image: {}", e)
        ))?;
    
    // Convert to RGB or grayscale based on image type
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    
    // Convert to numpy array [H, W, C] format
    let raw_data = rgb_img.into_raw();
    let ndarray = ndarray::Array3::<u8>::from_shape_vec(
        (height as usize, width as usize, 3),
        raw_data,
    ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
        format!("Failed to create array: {}", e)
    ))?;
    let array = ndarray.into_pyarray(py);
    Ok(array.into())
}

/// Decode multiple images in parallel using Rayon
#[pyfunction]
pub fn decode_image_batch(py: Python, bytes_list: &Bound<PyList>) -> PyResult<Vec<PyObject>> {
    let num_images = bytes_list.len();
    
    // Collect bytes into Vec<Vec<u8>>
    let bytes_vec: Result<Vec<Vec<u8>>, _> = (0..num_images)
        .map(|i| {
            let bytes = bytes_list.get_item(i)?;
            let py_bytes = bytes.downcast::<PyBytes>()?;
            Ok(py_bytes.as_bytes().to_vec())
        })
        .collect();
    
    let bytes_vec = bytes_vec?;
    
    // Decode images in parallel
    let decoded: Result<Vec<DynamicImage>, _> = bytes_vec
        .par_iter()
        .map(|data| {
            image::load_from_memory_with_format(data, ImageFormat::Png)
                .or_else(|_| image::load_from_memory_with_format(data, ImageFormat::Jpeg))
                .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))
        })
        .collect();
    
    let decoded = decoded.map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
        format!("Batch decode error: {}", e)
    ))?;
    
    // Convert to numpy arrays
    let arrays: Result<Vec<PyObject>, _> = decoded
        .iter()
        .map(|img| {
            let rgb_img = img.to_rgb8();
            let (width, height) = rgb_img.dimensions();
            let raw_data = rgb_img.into_raw();
            let ndarray = ndarray::Array3::<u8>::from_shape_vec(
                (height as usize, width as usize, 3),
                raw_data,
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to create array: {}", e)
            ))?;
            let array = ndarray.into_pyarray(py);
            Ok(array.into())
        })
        .collect();
    
    Ok(arrays.map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
        format!("Array conversion error: {}", e)
    ))?)
}

/// Preprocess an image: resize and optionally normalize
#[pyfunction]
#[pyo3(signature = (image, size, normalize = true))]
pub fn preprocess_image(
    py: Python,
    image: &Bound<PyAny>,
    size: (u32, u32),
    normalize: bool,
) -> PyResult<PyObject> {
    // Convert numpy array to image
    // Try to downcast to PyArray3 first (most common case)
    let img = if let Ok(array) = image.downcast::<PyArray3<u8>>() {
        // Get shape via Python API
        let shape_py = array.getattr("shape")?;
        let shape_tuple = shape_py.downcast::<PyTuple>()?;
        let height: usize = shape_tuple.get_item(0)?.extract()?;
        let width: usize = shape_tuple.get_item(1)?.extract()?;
        let channels: usize = shape_tuple.get_item(2)?.extract()?;
        
        // Get data via Python API
        let data_py = array.call_method0("flatten")?;
        let data_list = data_py.downcast::<PyList>()?;
        let data: Vec<u8> = data_list.iter().map(|item| item.extract::<u8>()).collect::<Result<_, _>>()?;
        
        if channels == 1 {
            // Grayscale
            let gray_data = data;
            DynamicImage::ImageLuma8(
                GrayImage::from_raw(width as u32, height as u32, gray_data)
                    .ok_or(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Failed to create grayscale image"
                    ))?
            )
        } else if channels == 3 {
            // RGB
            let rgb_data = data;
            DynamicImage::ImageRgb8(
                RgbImage::from_raw(width as u32, height as u32, rgb_data)
                    .ok_or(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Failed to create RGB image"
                    ))?
            )
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Unsupported number of channels (expected 1 or 3)"
            ));
        }
    } else {
        // Try dynamic array
        let array = image.downcast::<PyArrayDyn<u8>>()
            .map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected numpy array of uint8"
            ))?;
        
        // Get shape via Python API
        let shape_py = array.getattr("shape")?;
        let shape_tuple = shape_py.downcast::<pyo3::types::PyTuple>()?;
        if shape_tuple.len() != 3 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Expected 3D array [H, W, C]"
            ));
        }
        
        let height: usize = shape_tuple.get_item(0)?.extract()?;
        let width: usize = shape_tuple.get_item(1)?.extract()?;
        let channels: usize = shape_tuple.get_item(2)?.extract()?;
        
        // Get data via Python API
        let data_py = array.call_method0("flatten")?;
        let data_list = data_py.downcast::<pyo3::types::PyList>()?;
        let data: Vec<u8> = data_list.iter().map(|item| item.extract::<u8>()).collect::<Result<_, _>>()?;
        
        if channels == 1 {
            let gray_data = data;
            DynamicImage::ImageLuma8(
                GrayImage::from_raw(width as u32, height as u32, gray_data)
                    .ok_or(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Failed to create grayscale image"
                    ))?
            )
        } else if channels == 3 {
            let rgb_data = data;
            DynamicImage::ImageRgb8(
                RgbImage::from_raw(width as u32, height as u32, rgb_data)
                    .ok_or(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Failed to create RGB image"
                    ))?
            )
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Unsupported number of channels (expected 1 or 3)"
            ));
        }
    };
    
    // Resize if needed
    let (target_width, target_height) = size;
    let resized = if img.width() != target_width || img.height() != target_height {
        img.resize_exact(target_width, target_height, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    
    // Convert to RGB and normalize if requested
    let rgb = resized.to_rgb8();
    let (width, height) = rgb.dimensions();
    
    if normalize {
        // Normalize to [0, 1] and apply ImageNet stats
        let mean = [0.485, 0.456, 0.406];
        let std = [0.229, 0.224, 0.225];
        
        let normalized: Vec<f32> = rgb
            .pixels()
            .flat_map(|pixel| {
                let r = (pixel[0] as f32 / 255.0 - mean[0]) / std[0];
                let g = (pixel[1] as f32 / 255.0 - mean[1]) / std[1];
                let b = (pixel[2] as f32 / 255.0 - mean[2]) / std[2];
                vec![r, g, b]
            })
            .collect();
        
        let ndarray = ndarray::Array3::<f32>::from_shape_vec(
            (height as usize, width as usize, 3),
            normalized,
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to create normalized array: {}", e)
        ))?;
        let array = ndarray.into_pyarray(py);
        Ok(array.into())
    } else {
        // Return as uint8
        let raw_data = rgb.into_raw();
        let ndarray = ndarray::Array3::<u8>::from_shape_vec(
            (height as usize, width as usize, 3),
            raw_data,
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to create array: {}", e)
        ))?;
        let array = ndarray.into_pyarray(py);
        Ok(array.into())
    }
}
