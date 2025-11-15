#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Example: ML task benchmarks demonstrating that mappy's approximate nature
//! doesn't hurt ML performance with Huffman-compressed tags

#[cfg(feature = "mappy-integration")]
use stilts::benchmark::ml_benchmarks::MLBenchmarkRunner;
#[cfg(feature = "mappy-integration")]
use stilts::plotting::ml_reports::MLReportGenerator;

#[cfg(feature = "mappy-integration")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== ML Task Benchmarks: Exact vs Approximate ===\n");

    println!("Running comprehensive ML benchmarks...");
    println!("This demonstrates that mappy's approximate nature doesn't");
    println!("significantly impact ML task performance.\n");

    let results = MLBenchmarkRunner::run_comprehensive_benchmarks().await?;

    println!("\n=== Results Summary ===\n");

    for result in &results {
        println!("Task: {}", result.task_name);
        println!(
            "  Exact Accuracy:     {:.2}%",
            result.exact_accuracy * 100.0
        );
        println!(
            "  Approximate Accuracy: {:.2}%",
            result.approximate_accuracy * 100.0
        );
        println!(
            "  Accuracy Difference:  {:.4}%",
            result.accuracy_difference * 100.0
        );
        println!("  Exact Time:          {:.2} ms", result.exact_time_ms);
        println!("  Approximate Time:   {:.2} ms", result.approximate_time_ms);
        println!("  Speed Ratio:        {:.2}x", result.speed_ratio);

        let is_acceptable = result.is_acceptable(0.1, 3.0);
        println!(
            "  Acceptable:         {}",
            if is_acceptable { "✓ YES" } else { "✗ NO" }
        );
        println!();
    }

    // Summary statistics
    let avg_accuracy_diff: f64 =
        results.iter().map(|r| r.accuracy_difference).sum::<f64>() / results.len() as f64;

    let avg_speed_ratio: f64 =
        results.iter().map(|r| r.speed_ratio).sum::<f64>() / results.len() as f64;

    let all_acceptable = results.iter().all(|r| r.is_acceptable(0.1, 3.0));

    println!("=== Overall Summary ===");
    println!(
        "Average Accuracy Difference: {:.4}%",
        avg_accuracy_diff * 100.0
    );
    println!("Average Speed Ratio:        {:.2}x", avg_speed_ratio);
    println!(
        "All Tasks Acceptable:       {}",
        if all_acceptable { "✓ YES" } else { "✗ NO" }
    );
    println!();

    if all_acceptable {
        println!("✓ SUCCESS: All ML tasks perform acceptably with approximate mappy storage!");
        println!("  - Accuracy degradation is minimal (<10%)");
        println!("  - Speed degradation is acceptable (<3x)");
        println!("  - Huffman compression + mappy is viable for ML workloads");
    } else {
        println!("⚠ WARNING: Some tasks may need optimization");
    }

    // Generate HTML report
    println!("\n=== Generating HTML Report ===");
    let report_path =
        MLReportGenerator::generate_ml_report(&results, "reports/ml_benchmark_report.html")?;
    println!("Report generated: {}", report_path);

    Ok(())
}

#[cfg(not(feature = "mappy-integration"))]
fn main() {
    println!("Please enable the 'mappy-integration' feature to run this example");
    println!("Run: cargo run --example ml_benchmark_demo --features mappy-integration");
}
