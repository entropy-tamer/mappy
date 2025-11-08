//! Python bindings for Stilts

use pyo3::prelude::*;
use stilts::compression::{HuffmanCompressor, ArithmeticCompressor, DictionaryCompressor, Compressor};
use stilts::formats::{SpaceSeparatedParser, CommaSeparatedParser, JsonParser, TagParser};
use stilts::benchmark::{BenchmarkRunner, ComparisonRunner};
use stilts::plotting::ReportGenerator;
use serde_json;

#[pyclass]
pub struct StiltsCompressor {
    huffman: HuffmanCompressor,
    arithmetic: ArithmeticCompressor,
    dictionary: DictionaryCompressor,
}

#[pymethods]
impl StiltsCompressor {
    #[new]
    fn new() -> Self {
        Self {
            huffman: HuffmanCompressor::new(),
            arithmetic: ArithmeticCompressor::new(),
            dictionary: DictionaryCompressor::new(),
        }
    }
    
    fn compress(&mut self, tags: Vec<String>, algorithm: String) -> PyResult<Vec<u8>> {
        let compressor: &dyn Compressor = match algorithm.as_str() {
            "huffman" => {
                self.huffman.build_from_corpus(&tags).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
                &self.huffman
            },
            "arithmetic" => {
                self.arithmetic.build_from_corpus(&tags).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
                &self.arithmetic
            },
            "dictionary" => {
                self.dictionary.build_from_corpus(&tags).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
                &self.dictionary
            },
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Unknown algorithm")),
        };
        
        compressor.compress(&tags).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
    
    fn decompress(&self, data: Vec<u8>, algorithm: String) -> PyResult<Vec<String>> {
        let compressor: &dyn Compressor = match algorithm.as_str() {
            "huffman" => &self.huffman,
            "arithmetic" => &self.arithmetic,
            "dictionary" => &self.dictionary,
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Unknown algorithm")),
        };
        
        compressor.decompress(&data).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
    
    fn compare_algorithms(&mut self, tags: Vec<String>) -> PyResult<String> {
        let results = BenchmarkRunner::benchmark_all(&tags, 10)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        
        serde_json::to_string(&results)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }
    
    fn benchmark(&mut self, tags: Vec<String>, algorithms: Vec<String>) -> PyResult<String> {
        let mut all_results = Vec::new();
        
        for algorithm in algorithms {
            let compressor: &dyn Compressor = match algorithm.as_str() {
                "huffman" => {
                    self.huffman.build_from_corpus(&tags).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
                    &self.huffman
                },
                "arithmetic" => {
                    self.arithmetic.build_from_corpus(&tags).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
                    &self.arithmetic
                },
                "dictionary" => {
                    self.dictionary.build_from_corpus(&tags).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
                    &self.dictionary
                },
                _ => continue,
            };
            
            let result = BenchmarkRunner::benchmark(compressor, &tags, 10)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
            all_results.push(result);
        }
        
        serde_json::to_string(&all_results)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }
    
    fn plot_comparison(&mut self, tags: Vec<String>, mut output_path: String) -> PyResult<String> {
        // Ensure output is in reports directory
        if !output_path.starts_with("reports/") && !output_path.contains("/") {
            output_path = format!("reports/{}", output_path);
        }
        let results = BenchmarkRunner::benchmark_all(&tags, 10)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        
        // Also compare with external libraries
        let data: Vec<u8> = tags.iter().flat_map(|t| t.as_bytes()).collect();
        let mut external_results = ComparisonRunner::compare_all(&data, 10)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        
        let mut all_results = results;
        all_results.append(&mut external_results);
        
        ReportGenerator::generate_html_report(&all_results, &output_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }
}

#[pymodule]
fn stilts_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<StiltsCompressor>()?;
    Ok(())
}

