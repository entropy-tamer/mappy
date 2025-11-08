//! Benchmarking framework for compression algorithms

pub mod runner;
pub mod comparison;
pub mod metrics;

#[cfg(feature = "mappy-integration")]
pub mod mappy_comparison;

pub use runner::BenchmarkRunner;
pub use comparison::ComparisonRunner;
pub use metrics::{BenchmarkMetrics, CompressionStats};

