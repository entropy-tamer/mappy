#![allow(clippy::cast_precision_loss)] // Acceptable for plotting calculations
//! Chart generation using plotters

use crate::benchmark::metrics::BenchmarkMetrics;
use anyhow::Result;
use plotters::prelude::*;

/// Chart generator
pub struct ChartGenerator;

impl ChartGenerator {
    /// Generate compression ratio comparison chart
    pub fn compression_ratio_chart(metrics: &[BenchmarkMetrics], output_path: &str) -> Result<()> {
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Compression Ratio Comparison", ("sans-serif", 40))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..metrics.len() as f64, 0.0..1.0)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < metrics.len() {
                    metrics[idx].algorithm.clone()
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        let bars: Vec<(usize, f64)> = metrics
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.stats.compression_ratio))
            .collect();

        // Draw bars centered at x + 0.5
        chart.draw_series(bars.iter().map(|(x, y)| {
            let center = *x as f64 + 0.5;
            let width = 0.8; // Bar width (80% of available space)
            Rectangle::new(
                [(center - width / 2.0, 0.0), (center + width / 2.0, *y)],
                BLUE.filled(),
            )
        }))?;

        root.present()?;
        Ok(())
    }

    /// Generate speed comparison chart
    pub fn speed_comparison_chart(metrics: &[BenchmarkMetrics], output_path: &str) -> Result<()> {
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_speed = metrics
            .iter()
            .map(|m| {
                m.stats
                    .compression_speed_mbps
                    .max(m.stats.decompression_speed_mbps)
            })
            .fold(0.0, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption("Speed Comparison (MB/s)", ("sans-serif", 40))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..metrics.len() as f64, 0.0..max_speed * 1.1)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < metrics.len() {
                    metrics[idx].algorithm.clone()
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        // Compression speed
        let comp_bars: Vec<(usize, f64)> = metrics
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.stats.compression_speed_mbps))
            .collect();

        // Draw bars centered at x + 0.5, offset for grouped bars
        chart
            .draw_series(comp_bars.iter().map(|(x, y)| {
                let center = *x as f64 + 0.5;
                let width = 0.35; // Bar width for grouped bars
                Rectangle::new([(center - width, 0.0), (center, *y)], BLUE.filled())
            }))?
            .label("Compression");

        // Decompression speed
        let decomp_bars: Vec<(usize, f64)> = metrics
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.stats.decompression_speed_mbps))
            .collect();

        chart
            .draw_series(decomp_bars.iter().map(|(x, y)| {
                let center = *x as f64 + 0.5;
                let width = 0.35; // Bar width for grouped bars
                Rectangle::new([(center, 0.0), (center + width, *y)], RED.filled())
            }))?
            .label("Decompression");

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;

        root.present()?;
        Ok(())
    }

    /// Generate scatter plot: compression ratio vs speed
    pub fn ratio_vs_speed_chart(metrics: &[BenchmarkMetrics], output_path: &str) -> Result<()> {
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
            points
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 5, BLUE.filled())),
        )?;

        root.present()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::metrics::CompressionStats;

    #[test]
    fn test_compression_ratio_chart() {
        let metrics = vec![BenchmarkMetrics {
            algorithm: "test".to_string(),
            stats: CompressionStats::new(1000, 500, 100.0, 50.0, 128),
            memory_usage_bytes: 500,
        }];

        let result = ChartGenerator::compression_ratio_chart(&metrics, "/tmp/test_ratio.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_speed_comparison_chart() {
        let metrics = vec![BenchmarkMetrics {
            algorithm: "test".to_string(),
            stats: CompressionStats::new(1000, 500, 100.0, 50.0, 128),
            memory_usage_bytes: 500,
        }];

        let result = ChartGenerator::speed_comparison_chart(&metrics, "/tmp/test_speed.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ratio_vs_speed_chart() {
        let metrics = vec![BenchmarkMetrics {
            algorithm: "test".to_string(),
            stats: CompressionStats::new(1000, 500, 100.0, 50.0, 128),
            memory_usage_bytes: 500,
        }];

        let result = ChartGenerator::ratio_vs_speed_chart(&metrics, "/tmp/test_ratio_speed.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compression_ratio_chart_empty_metrics() {
        let metrics: Vec<BenchmarkMetrics> = vec![];
        let result = ChartGenerator::compression_ratio_chart(&metrics, "/tmp/test_empty.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_speed_comparison_chart_empty_metrics() {
        let metrics: Vec<BenchmarkMetrics> = vec![];
        let result = ChartGenerator::speed_comparison_chart(&metrics, "/tmp/test_empty_speed.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ratio_vs_speed_chart_empty_metrics() {
        let metrics: Vec<BenchmarkMetrics> = vec![];
        let result = ChartGenerator::ratio_vs_speed_chart(&metrics, "/tmp/test_empty_ratio_speed.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compression_ratio_chart_multiple_metrics() {
        let metrics = vec![
            BenchmarkMetrics {
                algorithm: "algo1".to_string(),
                stats: CompressionStats::new(1000, 500, 100.0, 50.0, 128),
                memory_usage_bytes: 500,
            },
            BenchmarkMetrics {
                algorithm: "algo2".to_string(),
                stats: CompressionStats::new(1000, 600, 120.0, 60.0, 128),
                memory_usage_bytes: 600,
            },
        ];

        let result = ChartGenerator::compression_ratio_chart(&metrics, "/tmp/test_multiple.png");
        assert!(result.is_ok());
    }
}
