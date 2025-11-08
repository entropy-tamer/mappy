//! Compression algorithm benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stilts::compression::{HuffmanCompressor, ArithmeticCompressor, DictionaryCompressor, Compressor};

fn generate_test_tags(count: usize) -> Vec<String> {
    let tags = vec![
        "2007", "3_toes", "4_fingers", "anthro", "biped",
        "black_and_white", "breasts", "canid", "canine", "claws",
    ];
    
    (0..count)
        .map(|i| tags[i % tags.len()].to_string())
        .collect()
}

fn benchmark_huffman(c: &mut Criterion) {
    let tags = generate_test_tags(100);
    let mut compressor = HuffmanCompressor::new();
    compressor.build_from_corpus(&tags).unwrap();
    
    c.bench_function("huffman_compress", |b| {
        b.iter(|| compressor.compress(black_box(&tags)))
    });
    
    let compressed = compressor.compress(&tags).unwrap();
    c.bench_function("huffman_decompress", |b| {
        b.iter(|| compressor.decompress(black_box(&compressed)))
    });
}

fn benchmark_arithmetic(c: &mut Criterion) {
    let tags = generate_test_tags(100);
    let mut compressor = ArithmeticCompressor::new();
    compressor.build_from_corpus(&tags).unwrap();
    
    c.bench_function("arithmetic_compress", |b| {
        b.iter(|| compressor.compress(black_box(&tags)))
    });
    
    let compressed = compressor.compress(&tags).unwrap();
    c.bench_function("arithmetic_decompress", |b| {
        b.iter(|| compressor.decompress(black_box(&compressed)))
    });
}

fn benchmark_dictionary(c: &mut Criterion) {
    let tags = generate_test_tags(100);
    let mut compressor = DictionaryCompressor::new();
    compressor.build_from_corpus(&tags).unwrap();
    
    c.bench_function("dictionary_compress", |b| {
        b.iter(|| compressor.compress(black_box(&tags)))
    });
    
    let compressed = compressor.compress(&tags).unwrap();
    c.bench_function("dictionary_decompress", |b| {
        b.iter(|| compressor.decompress(black_box(&compressed)))
    });
}

criterion_group!(benches, benchmark_huffman, benchmark_arithmetic, benchmark_dictionary);
criterion_main!(benches);

