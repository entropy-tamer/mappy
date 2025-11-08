//! Chart generation using plotters

use anyhow::Result;
use plotters::prelude::*;
use crate::benchmark::metrics::BenchmarkMetrics;

/// Chart generator
pub struct ChartGenerator;

impl ChartGenerator {
    /// Generate compression ratio comparison chart
    pub fn compression_ratio_chart(
        metrics: &[BenchmarkMetrics],
        output_path: &str,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let mut chart = ChartBuilder::on(&root)
            .caption("Compression Ratio Comparison", ("sans-serif", 40))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..metrics.len(), 0.0..1.0)?;
        
        chart.configure_mesh().draw()?;
        
        let bars: Vec<(usize, f64)> = metrics
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.stats.compression_ratio))
            .collect();
        
        chart.draw_series(
            bars.iter().map(|(x, y)| {
                Rectangle::new([(*x, 0.0), (*x + 1, *y)], BLUE.filled())
            })
        )?;
        
        root.present()?;
        Ok(())
    }
    
    /// Generate speed comparison chart
    pub fn speed_comparison_chart(
        metrics: &[BenchmarkMetrics],
        output_path: &str,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let max_speed = metrics
            .iter()
            .map(|m| m.stats.compression_speed_mbps.max(m.stats.decompression_speed_mbps))
            .fold(0.0, f64::max);
        
        let mut chart = ChartBuilder::on(&root)
            .caption("Speed Comparison (MB/s)", ("sans-serif", 40))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..metrics.len(), 0.0..max_speed * 1.1)?;
        
        chart.configure_mesh().draw()?;
        
        // Compression speed
        let comp_bars: Vec<(usize, f64)> = metrics
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.stats.compression_speed_mbps))
            .collect();
        
        chart.draw_series(
            comp_bars.iter().map(|(x, y)| {
                Rectangle::new([(*x, 0.0), (*x + 1, *y)], BLUE.filled())
            })
        )?;
        
        // Decompression speed
        let decomp_bars: Vec<(usize, f64)> = metrics
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.stats.decompression_speed_mbps))
            .collect();
        
        chart.draw_series(
            decomp_bars.iter().map(|(x, y)| {
                Rectangle::new([(*x, 0.0), (*x + 1, *y)], RED.filled())
            })
        )?;
        
        root.present()?;
        Ok(())
    }
    
    /// Generate scatter plot: compression ratio vs speed
    pub fn ratio_vs_speed_chart(
        metrics: &[BenchmarkMetrics],
        output_path: &str,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let max_ratio = metrics
            .iter()
            .map(|m| m.stats.compression_ratio)
            .fold(0.0, f64::max);
        let max_speed = metrics
            .iter()
            .map(|m| m.stats.compression_speed_mbps)
            .fold(0.0, f64::max);
        
        let mut chart = ChartBuilder::on(&root)
            .caption("Compression Ratio vs Speed", ("sans-serif", 40))
            .margin(5)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..max_ratio * 1.1, 0.0..max_speed * 1.1)?;
        
        chart.configure_mesh().draw()?;
        
        let points: Vec<(f64, f64)> = metrics
            .iter()
            .map(|m| (m.stats.compression_ratio, m.stats.compression_speed_mbps))
            .collect();
        
        chart.draw_series(
            points.iter().map(|(x, y)| {
                Circle::new((*x, *y), 5, BLUE.filled())
            })
        )?;
        
        root.present()?;
        Ok(())
    }
}

