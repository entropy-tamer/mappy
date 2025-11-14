//! Benchmarking framework for compression algorithms

pub mod runner;
pub mod comparison;
pub mod metrics;

#[cfg(feature = "mappy-integration")]
pub mod mappy_comparison;

#[cfg(feature = "mappy-integration")]
pub mod ml_tasks;
#[cfg(feature = "mappy-integration")]
pub mod ml_benchmarks;
#[cfg(feature = "mappy-integration")]
#[cfg(test)]
mod ml_accuracy_tests;

pub use runner::BenchmarkRunner;
pub use comparison::ComparisonRunner;
pub use metrics::{BenchmarkMetrics, CompressionStats};

