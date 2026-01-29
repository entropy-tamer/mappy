#![allow(clippy::cast_precision_loss)] // Acceptable for plotting calculations
//! Storage comparison charts

use crate::benchmark::mappy_comparison::StorageComparison;
use anyhow::Result;
use plotters::prelude::*;

/// Storage comparison chart generator
pub struct StorageChartGenerator;

impl StorageChartGenerator {
    /// Generate storage size comparison chart
    pub fn storage_size_chart(comparisons: &[StorageComparison], output_path: &str) -> Result<()> {
        let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_size = comparisons
            .iter()
            .map(|c| c.storage_size.max(c.original_size))
            .fold(0, usize::max) as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption("Storage Size Comparison", ("sans-serif", 50))
            .margin(10)
            .x_label_area_size(80)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0..comparisons.len() as f64, 0.0..max_size * 1.1)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < comparisons.len() {
                    comparisons[idx].method.clone()
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        // Original size bars
        let original_bars: Vec<(usize, f64)> = comparisons
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.original_size as f64))
            .collect();

        // Draw bars centered at x + 0.5, offset for grouped bars
        chart
            .draw_series(original_bars.iter().map(|(x, y)| {
                let center = *x as f64 + 0.5;
                let width = 0.35; // Bar width for grouped bars
                Rectangle::new([(center - width, 0.0), (center, *y)], BLUE.filled())
            }))?
            .label("Original Size");

        // Storage size bars
        let storage_bars: Vec<(usize, f64)> = comparisons
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.storage_size as f64))
            .collect();

        chart
            .draw_series(storage_bars.iter().map(|(x, y)| {
                let center = *x as f64 + 0.5;
                let width = 0.35; // Bar width for grouped bars
                Rectangle::new([(center, 0.0), (center + width, *y)], RED.filled())
            }))?
            .label("Storage Size");

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;

        root.present()?;
        Ok(())
    }

    /// Generate compression ratio comparison chart
    pub fn compression_ratio_chart(
        comparisons: &[StorageComparison],
        output_path: &str,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_ratio = comparisons
            .iter()
            .map(|c| c.compression_ratio)
            .fold(0.0, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption("Compression Ratio Comparison", ("sans-serif", 50))
            .margin(10)
            .x_label_area_size(80)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0..comparisons.len() as f64, 0.0..max_ratio * 1.1)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < comparisons.len() {
                    comparisons[idx].method.clone()
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        let bars: Vec<(usize, f64)> = comparisons
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.compression_ratio))
            .collect();

        // Draw bars centered at x + 0.5
        chart.draw_series(bars.iter().map(|(x, y)| {
            let center = *x as f64 + 0.5;
            let width = 0.8; // Bar width (80% of available space)
            Rectangle::new(
                [(center - width / 2.0, 0.0), (center + width / 2.0, *y)],
                GREEN.filled(),
            )
        }))?;

        root.present()?;
        Ok(())
    }

    /// Generate memory usage comparison chart
    pub fn memory_usage_chart(comparisons: &[StorageComparison], output_path: &str) -> Result<()> {
        let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_memory = comparisons
            .iter()
            .map(|c| c.memory_usage_bytes)
            .fold(0, usize::max) as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption("Memory Usage Comparison (bytes)", ("sans-serif", 50))
            .margin(10)
            .x_label_area_size(80)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0..comparisons.len() as f64, 0.0..max_memory * 1.1)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < comparisons.len() {
                    comparisons[idx].method.clone()
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        let bars: Vec<(usize, f64)> = comparisons
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.memory_usage_bytes as f64))
            .collect();

        // Draw bars centered at x + 0.5
        chart.draw_series(bars.iter().map(|(x, y)| {
            let center = *x as f64 + 0.5;
            let width = 0.8; // Bar width (80% of available space)
            Rectangle::new(
                [(center - width / 2.0, 0.0), (center + width / 2.0, *y)],
                MAGENTA.filled(),
            )
        }))?;

        root.present()?;
        Ok(())
    }

    /// Generate comprehensive comparison chart (ratio vs memory)
    pub fn ratio_vs_memory_chart(
        comparisons: &[StorageComparison],
        output_path: &str,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_ratio = comparisons
            .iter()
            .map(|c| c.compression_ratio)
            .fold(0.0, f64::max);
        let max_memory = comparisons
            .iter()
            .map(|c| c.memory_usage_bytes)
            .fold(0, usize::max) as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption("Compression Ratio vs Memory Usage", ("sans-serif", 50))
            .margin(10)
            .x_label_area_size(80)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0..max_ratio * 1.1, 0.0..max_memory * 1.1)?;

        chart.configure_mesh().draw()?;

        let points: Vec<(f64, f64)> = comparisons
            .iter()
            .map(|c| (c.compression_ratio, c.memory_usage_bytes as f64))
            .collect();

        chart.draw_series(
            points
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 8, BLUE.filled())),
        )?;

        // Note: Labels would be added here if needed, but plotters Text API may vary

        root.present()?;
        Ok(())
    }
}
