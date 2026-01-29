#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark calculations
//! Benchmarking framework for compression algorithms

pub mod comparison;
pub mod metrics;
pub mod runner;

#[cfg(feature = "mappy-integration")]
pub mod mappy_comparison;

#[cfg(feature = "mappy-integration")]
#[cfg(test)]
mod ml_accuracy_tests;
#[cfg(feature = "mappy-integration")]
pub mod ml_benchmarks;
#[cfg(feature = "mappy-integration")]
pub mod ml_tasks;

pub use comparison::ComparisonRunner;
pub use metrics::{BenchmarkMetrics, CompressionStats};
pub use runner::BenchmarkRunner;
