#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Debug timing to see what's slow

use std::time::Instant;
use stilts::formats::{SpaceSeparatedParser, TagParser};
#[cfg(feature = "mappy-integration")]
use stilts::mappy_integration::MappyTagStorage;

#[cfg(feature = "mappy-integration")]
fn main() -> anyhow::Result<()> {
    let parser = SpaceSeparatedParser::new();
    let input = "2007 3_toes 4_fingers anthro biped black_and_white chest canid canine";
    let tags = parser.parse(input)?;

    println!(
        "Testing {} tags, {} bytes original\n",
        tags.len(),
        input.len()
    );

    // Test Huffman
    println!("=== Huffman Timing ===");
    let mut storage = MappyTagStorage::with_huffman();

    // Time corpus building
    let start = Instant::now();
    storage.build_corpus(&tags)?;
    let corpus_time = start.elapsed().as_secs_f64() * 1000.0;
    println!("  Corpus building: {:.4} ms", corpus_time);

    // Time compression (corpus already built)
    let start = Instant::now();
    for _ in 0..100 {
        let _ = storage.compress_tags(&tags)?;
    }
    let compress_time = start.elapsed().as_secs_f64() * 1000.0 / 100.0;
    println!(
        "  Compression (100x, corpus built): {:.4} ms per operation",
        compress_time
    );

    // Test Dictionary
    println!("\n=== Dictionary Timing ===");
    let mut storage = MappyTagStorage::with_dictionary();

    let start = Instant::now();
    storage.build_corpus(&tags)?;
    let corpus_time = start.elapsed().as_secs_f64() * 1000.0;
    println!("  Corpus building: {:.4} ms", corpus_time);

    let start = Instant::now();
    for _ in 0..100 {
        let _ = storage.compress_tags(&tags)?;
    }
    let compress_time = start.elapsed().as_secs_f64() * 1000.0 / 100.0;
    println!(
        "  Compression (100x, corpus built): {:.4} ms per operation",
        compress_time
    );

    // Test Arithmetic
    println!("\n=== Arithmetic Timing ===");
    let mut storage = MappyTagStorage::with_arithmetic();

    let start = Instant::now();
    storage.build_corpus(&tags)?;
    let corpus_time = start.elapsed().as_secs_f64() * 1000.0;
    println!("  Corpus building: {:.4} ms", corpus_time);

    let start = Instant::now();
    for _ in 0..100 {
        let _ = storage.compress_tags(&tags)?;
    }
    let compress_time = start.elapsed().as_secs_f64() * 1000.0 / 100.0;
    println!(
        "  Compression (100x, corpus built): {:.4} ms per operation",
        compress_time
    );

    Ok(())
}

#[cfg(not(feature = "mappy-integration"))]
fn main() {
    println!("Please enable the 'mappy-integration' feature");
}
