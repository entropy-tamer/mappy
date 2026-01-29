#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Basic compression example

use stilts::{HuffmanCompressor, SpaceSeparatedParser, TagParser};

fn main() -> anyhow::Result<()> {
    // Parse tags
    let parser = SpaceSeparatedParser::new();
    let input = "2007 3_toes 4_fingers anthro biped black_and_white chest canid canine";
    let tags = parser.parse(input)?;

    println!("Original tags: {:?}", tags);
    println!("Original size: {} bytes", input.len());

    // Compress with Huffman
    let mut compressor = HuffmanCompressor::new();
    compressor.build_from_corpus(&tags)?;

    let compressed = compressor.compress(&tags)?;
    println!("Compressed size: {} bytes", compressed.len());
    println!(
        "Compression ratio: {:.2}%",
        (compressed.len() as f64 / input.len() as f64) * 100.0
    );

    // Decompress
    let decompressed = compressor.decompress(&compressed)?;
    println!("Decompressed tags: {:?}", decompressed);

    assert_eq!(tags, decompressed);
    println!("Round-trip successful!");

    Ok(())
}
