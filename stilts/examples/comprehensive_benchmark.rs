#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Comprehensive benchmark with multiple tag sets and formats

use stilts::benchmark::mappy_comparison::MappyComparisonRunner;
use stilts::compression::{
    ArithmeticCompressor, Compressor, DictionaryCompressor, HuffmanCompressor,
};
#[cfg(feature = "mappy-integration")]
use stilts::formats::{CommaSeparatedParser, JsonParser, SpaceSeparatedParser, TagParser};
use stilts::plotting::ReportGenerator;

#[cfg(feature = "mappy-integration")]
fn main() -> anyhow::Result<()> {
    println!("=== Stilts Comprehensive Benchmark ===\n");

    // Test case 1: Original example (long tag string)
    println!("Test Case 1: Long tag string");
    let parser1 = SpaceSeparatedParser::new();
    let input1 = "2007 3_toes 4_fingers anthro biped black_and_white chest canid canine claws collar dialogue domestic_dog english_text eyewear fangs feet female fingers accessories glasses humanoid_features humanoid_form inks legs_together lineart mammal monochrome clothing dressed pen(artwork) pince-nez raine_(rainedog) rainedog razor sharp_teeth shaving signature simple_background small_frame solo standing stubble tail teeth text toe_claws toes tools traditionalmedia(artwork) visible what white_background";
    let tags1 = parser1.parse(input1)?;
    println!(
        "  Parsed {} tags, {} bytes original",
        tags1.len(),
        input1.len()
    );

    // Test case 2: Compressed format (short codes)
    println!("\nTest Case 2: Compressed codes format");
    let input2 = "gj 3x 4S 1 04 7M A G I e t 1R IAZ v 0d Y 12 1N o 3 w 6 1t 01 Bb 539 WY Ix 0 0R H K px 0ED JI0 JDO 3P4 47 3Ui 1Z J 7C 5 0C 07e D c L 2l 0F Ig 7D W RF 0r";
    let tags2 = parser1.parse(input2)?;
    println!(
        "  Parsed {} tags, {} bytes original",
        tags2.len(),
        input2.len()
    );

    // Test case 3: Large dataset (generated)
    println!("\nTest Case 3: Large generated dataset");
    let tags3 = MappyComparisonRunner::generate_test_tags(500);
    let original_size3: usize = tags3.iter().map(|t| t.len()).sum();
    println!(
        "  Generated {} tags, {} bytes total",
        tags3.len(),
        original_size3
    );

    // Test case 4: Comma-separated format
    println!("\nTest Case 4: Comma-separated format");
    let parser4 = CommaSeparatedParser::new();
    let input4 = "tag1,tag2,tag3,anthro,biped,canid,canine,claws,collar,dialogue";
    let tags4 = parser4.parse(input4)?;
    println!(
        "  Parsed {} tags, {} bytes original",
        tags4.len(),
        input4.len()
    );

    // Benchmark all compression algorithms on each test case
    let test_cases = vec![
        ("Long tag string", tags1),
        ("Compressed codes", tags2),
        ("Large dataset", tags3),
        ("Comma-separated", tags4),
    ];

    let mut all_comparisons = Vec::new();

    for (name, tags) in test_cases {
        println!("\n=== Benchmarking: {} ===", name);

        // Test each compression algorithm
        println!("  Testing compression algorithms...");

        // Huffman
        let mut huffman = HuffmanCompressor::new();
        huffman.build_from_corpus(&tags)?;
        let huffman_compressed = huffman.compress(&tags)?;
        let original_size: usize = tags.iter().map(|t| t.len()).sum();
        println!(
            "    Huffman: {} bytes (ratio: {:.2}%)",
            huffman_compressed.len(),
            (huffman_compressed.len() as f64 / original_size as f64) * 100.0
        );

        // Arithmetic
        let mut arithmetic = ArithmeticCompressor::new();
        arithmetic.build_from_corpus(&tags)?;
        let arithmetic_compressed = arithmetic.compress(&tags)?;
        println!(
            "    Arithmetic: {} bytes (ratio: {:.2}%)",
            arithmetic_compressed.len(),
            (arithmetic_compressed.len() as f64 / original_size as f64) * 100.0
        );

        // Dictionary
        let mut dictionary = DictionaryCompressor::new();
        dictionary.build_from_corpus(&tags)?;
        let dictionary_compressed = dictionary.compress(&tags)?;
        println!(
            "    Dictionary: {} bytes (ratio: {:.2}%)",
            dictionary_compressed.len(),
            (dictionary_compressed.len() as f64 / original_size as f64) * 100.0
        );

        // Storage comparison
        println!("  Comparing storage methods...");
        let comparisons = MappyComparisonRunner::compare_all_storage(&tags, 10)?;

        // Add test case name to method names
        let mut named_comparisons = comparisons
            .into_iter()
            .map(|mut c| {
                c.method = format!("{}_{}", name.replace(" ", "_").to_lowercase(), c.method);
                c
            })
            .collect();
        all_comparisons.append(&mut named_comparisons);
    }

    // Generate comprehensive report
    println!("\n=== Generating Comprehensive Report ===");
    let report_path = ReportGenerator::generate_storage_report(
        &all_comparisons,
        "reports/comprehensive_storage_report.html",
    )?;
    println!("Report generated: {}", report_path);

    // Also generate algorithm comparison report
    println!("\n=== Generating Algorithm Comparison ===");
    use stilts::benchmark::BenchmarkRunner;
    use stilts::plotting::ChartGenerator;

    // Ensure reports directory exists
    std::fs::create_dir_all("reports")?;

    // Use the large dataset for algorithm comparison
    let large_tags = MappyComparisonRunner::generate_test_tags(1000);
    let algo_results = BenchmarkRunner::benchmark_all(&large_tags, 10)?;

    // Generate algorithm comparison charts in reports directory
    ChartGenerator::compression_ratio_chart(
        &algo_results,
        "reports/algorithm_compression_ratio.png",
    )?;
    ChartGenerator::speed_comparison_chart(
        &algo_results,
        "reports/algorithm_speed_comparison.png",
    )?;
    ChartGenerator::ratio_vs_speed_chart(&algo_results, "reports/algorithm_ratio_vs_speed.png")?;

    println!("Algorithm comparison charts generated in reports/:");
    println!("  - reports/algorithm_compression_ratio.png");
    println!("  - reports/algorithm_speed_comparison.png");
    println!("  - reports/algorithm_ratio_vs_speed.png");

    Ok(())
}

#[cfg(not(feature = "mappy-integration"))]
fn main() {
    println!("Please enable the 'mappy-integration' feature to run this example");
    println!("Run: cargo run --example comprehensive_benchmark --features mappy-integration");
}
