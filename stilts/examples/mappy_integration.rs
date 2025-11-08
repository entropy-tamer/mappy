//! Example: Storing compressed tags in mappy

#[cfg(feature = "mappy-integration")]
use stilts::mappy_integration::MappyTagStorage;
use stilts::formats::{SpaceSeparatedParser, TagParser};
use stilts::benchmark::mappy_comparison::MappyComparisonRunner;
use stilts::plotting::ReportGenerator;

#[cfg(feature = "mappy-integration")]
fn main() -> anyhow::Result<()> {
    // Parse tags from the example format
    let parser = SpaceSeparatedParser::new();
    let input = "2007 3_toes 4_fingers anthro biped black_and_white breasts canid canine claws collar dialogue domestic_dog english_text eyewear fangs feet female fingers genitals glasses humanoid_genitalia humanoid_vulva inks legs_together lineart mammal monochrome nipples nude pen(artwork) pince-nez raine_(rainedog) rainedog razor sharp_teeth shaving signature simple_background small_breasts solo standing stubble tail teeth text toe_claws toes tools traditionalmedia(artwork) vulva what white_background";
    
    let tags = parser.parse(input)?;
    println!("Parsed {} tags", tags.len());
    
    // Test different compression methods
    println!("\n=== Compression Test ===");
    
    // Huffman
    let mut huffman_storage = MappyTagStorage::with_huffman();
    let huffman_compressed = huffman_storage.compress_tags(&tags)?;
    println!("Huffman: {} bytes (ratio: {:.2}%)", 
        huffman_compressed.len(),
        (huffman_compressed.len() as f64 / input.len() as f64) * 100.0);
    
    // Arithmetic
    let mut arithmetic_storage = MappyTagStorage::with_arithmetic();
    let arithmetic_compressed = arithmetic_storage.compress_tags(&tags)?;
    println!("Arithmetic: {} bytes (ratio: {:.2}%)", 
        arithmetic_compressed.len(),
        (arithmetic_compressed.len() as f64 / input.len() as f64) * 100.0);
    
    // Dictionary
    let mut dictionary_storage = MappyTagStorage::with_dictionary();
    let dictionary_compressed = dictionary_storage.compress_tags(&tags)?;
    println!("Dictionary: {} bytes (ratio: {:.2}%)", 
        dictionary_compressed.len(),
        (dictionary_compressed.len() as f64 / input.len() as f64) * 100.0);
    
    // Benchmark all storage methods
    println!("\n=== Storage Comparison ===");
    let comparisons = MappyComparisonRunner::compare_all_storage(&tags, 10)?;
    
    for comp in &comparisons {
        println!("{}: {} bytes stored (ratio: {:.4}, memory: {} bytes)",
            comp.method, comp.storage_size, comp.compression_ratio, comp.memory_usage_bytes);
    }
    
    // Generate report
    println!("\n=== Generating Report ===");
    let report_path = ReportGenerator::generate_storage_report(&comparisons, "reports/storage_report.html")?;
    println!("Report generated: {}", report_path);
    
    Ok(())
}

#[cfg(not(feature = "mappy-integration"))]
fn main() {
    println!("Please enable the 'mappy-integration' feature to run this example");
    println!("Run: cargo run --example mappy_integration --features mappy-integration");
}

