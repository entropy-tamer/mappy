#![allow(clippy::cast_precision_loss)] // Acceptable for plotting calculations
//! Report generation

use crate::benchmark::metrics::BenchmarkMetrics;
use crate::plotting::charts::ChartGenerator;
use anyhow::Result;
use serde_json;
use std::fs::File;
use std::io::Write;

#[cfg(feature = "mappy-integration")]
use crate::benchmark::mappy_comparison::StorageComparison;
#[cfg(feature = "mappy-integration")]
use crate::plotting::storage_charts::StorageChartGenerator;

/// Report generator
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate HTML report with charts
    pub fn generate_html_report(metrics: &[BenchmarkMetrics], output_path: &str) -> Result<String> {
        // Ensure reports directory exists
        let reports_dir = std::path::Path::new("reports");
        std::fs::create_dir_all(reports_dir)?;

        let chart_dir = std::path::Path::new(output_path)
            .parent()
            .unwrap_or(reports_dir);

        let ratio_chart = chart_dir.join("compression_ratio.png");
        let speed_chart = chart_dir.join("speed_comparison.png");
        let scatter_chart = chart_dir.join("ratio_vs_speed.png");

        // Generate charts
        ChartGenerator::compression_ratio_chart(metrics, ratio_chart.to_str().unwrap())?;
        ChartGenerator::speed_comparison_chart(metrics, speed_chart.to_str().unwrap())?;
        ChartGenerator::ratio_vs_speed_chart(metrics, scatter_chart.to_str().unwrap())?;

        // Generate HTML
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Stilts Compression Benchmark Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        table { border-collapse: collapse; width: 100%; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #4CAF50; color: white; }
        img { max-width: 100%; height: auto; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>Stilts Compression Benchmark Report</h1>
    <h2>Results Summary</h2>
    <table>
        <tr>
            <th>Algorithm</th>
            <th>Compression Ratio</th>
            <th>Compression Speed (MB/s)</th>
            <th>Decompression Speed (MB/s)</th>
            <th>Compressed Size (bytes)</th>
        </tr>
"#,
        );

        for metric in metrics {
            html.push_str(&format!(
                r#"        <tr>
            <td>{}</td>
            <td>{:.4}</td>
            <td>{:.2}</td>
            <td>{:.2}</td>
            <td>{}</td>
        </tr>
"#,
                metric.algorithm,
                metric.stats.compression_ratio,
                metric.stats.compression_speed_mbps,
                metric.stats.decompression_speed_mbps,
                metric.stats.compressed_size,
            ));
        }

        html.push_str(
            r#"    </table>
    <h2>Charts</h2>
    <h3>Compression Ratio Comparison</h3>
    <img src="compression_ratio.png" alt="Compression Ratio">
    <h3>Speed Comparison</h3>
    <img src="speed_comparison.png" alt="Speed Comparison">
    <h3>Compression Ratio vs Speed</h3>
    <img src="ratio_vs_speed.png" alt="Ratio vs Speed">
    <h2>JSON Data</h2>
    <pre>"#,
        );

        let json = serde_json::to_string_pretty(metrics)?;
        html.push_str(&json);
        html.push_str(
            r#"</pre>
</body>
</html>"#,
        );

        // Write HTML file
        let mut file = File::create(output_path)?;
        file.write_all(html.as_bytes())?;

        Ok(output_path.to_string())
    }

    /// Generate comprehensive storage comparison report
    #[cfg(feature = "mappy-integration")]
    pub fn generate_storage_report(
        comparisons: &[StorageComparison],
        output_path: &str,
    ) -> Result<String> {
        // Ensure reports directory exists
        let reports_dir = std::path::Path::new("reports");
        std::fs::create_dir_all(reports_dir)?;

        let chart_dir = std::path::Path::new(output_path)
            .parent()
            .unwrap_or(reports_dir);

        let size_chart = chart_dir.join("storage_size.png");
        let ratio_chart = chart_dir.join("storage_compression_ratio.png");
        let memory_chart = chart_dir.join("storage_memory.png");
        let scatter_chart = chart_dir.join("storage_ratio_vs_memory.png");

        // Generate charts
        StorageChartGenerator::storage_size_chart(comparisons, size_chart.to_str().unwrap())?;
        StorageChartGenerator::compression_ratio_chart(comparisons, ratio_chart.to_str().unwrap())?;
        StorageChartGenerator::memory_usage_chart(comparisons, memory_chart.to_str().unwrap())?;
        StorageChartGenerator::ratio_vs_memory_chart(comparisons, scatter_chart.to_str().unwrap())?;

        // Generate HTML
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Stilts Storage Comparison Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        table { border-collapse: collapse; width: 100%; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #4CAF50; color: white; }
        img { max-width: 100%; height: auto; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>Stilts Storage Comparison Report</h1>
    <h2>Results Summary</h2>
    <table>
        <tr>
            <th>Method</th>
            <th>Original Size (bytes)</th>
            <th>Storage Size (bytes)</th>
            <th>Compression Ratio</th>
            <th>Memory Usage (bytes)</th>
            <th>Insert Time (ms)</th>
        </tr>
"#,
        );

        for comp in comparisons {
            html.push_str(&format!(
                r#"        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{:.4}</td>
            <td>{}</td>
            <td>{:.4}</td>
        </tr>
"#,
                comp.method,
                comp.original_size,
                comp.storage_size,
                comp.compression_ratio,
                comp.memory_usage_bytes,
                comp.insert_time_ms,
            ));
        }

        html.push_str(
            r#"    </table>
    <h2>Charts</h2>
    <h3>Storage Size Comparison</h3>
    <img src="storage_size.png" alt="Storage Size">
    <h3>Compression Ratio Comparison</h3>
    <img src="storage_compression_ratio.png" alt="Compression Ratio">
    <h3>Memory Usage Comparison</h3>
    <img src="storage_memory.png" alt="Memory Usage">
    <h3>Compression Ratio vs Memory Usage</h3>
    <img src="storage_ratio_vs_memory.png" alt="Ratio vs Memory">
</body>
</html>"#,
        );

        // Write HTML file
        let mut file = std::fs::File::create(output_path)?;
        file.write_all(html.as_bytes())?;

        Ok(output_path.to_string())
    }
}
