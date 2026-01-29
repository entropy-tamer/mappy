#![allow(clippy::cast_precision_loss)] // Acceptable for plotting calculations
//! Plotting and visualization utilities

pub mod charts;
pub mod reports;

#[cfg(feature = "mappy-integration")]
pub mod storage_charts;

#[cfg(feature = "mappy-integration")]
pub mod ml_reports;

pub use charts::ChartGenerator;
pub use reports::ReportGenerator;
