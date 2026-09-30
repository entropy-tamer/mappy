#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Storage benchmarks: mappy vs other data structures

use criterion::{Criterion, criterion_group, criterion_main};
use criterion::black_box;
use stilts::benchmark::mappy_comparison::MappyComparisonRunner;
use stilts::compression::{
    ArithmeticCompressor, Compressor, DictionaryCompressor, HuffmanCompressor,
};

fn generate_large_tag_set() -> Vec<String> {
    MappyComparisonRunner::generate_test_tags(1000)
}

#[cfg(feature = "mappy-integration")]
fn benchmark_mappy_storage(c: &mut Criterion) {
    let tags = generate_large_tag_set();

    c.bench_function("mappy_storage_comparison", |b| {
        b.iter(|| MappyComparisonRunner::compare_all_storage(black_box(&tags), 1))
    });
}

fn benchmark_compression_methods(c: &mut Criterion) {
    let tags = generate_large_tag_set();

    c.bench_function("compression_comparison", |b| {
        b.iter(|| {
            // Test all compression methods. `compress` comes from the `Compressor`
            // trait, so the trait must be in scope -- importing only the structs
            // gives E0599, not a missing-method-on-struct hint.
            let mut huffman = HuffmanCompressor::new();
            huffman.build_from_corpus(black_box(&tags)).unwrap();
            let _ = huffman.compress(black_box(&tags));

            let mut arithmetic = ArithmeticCompressor::new();
            arithmetic.build_from_corpus(black_box(&tags)).unwrap();
            let _ = arithmetic.compress(black_box(&tags));

            let mut dictionary = DictionaryCompressor::new();
            dictionary.build_from_corpus(black_box(&tags)).unwrap();
            let _ = dictionary.compress(black_box(&tags));
        })
    });
}

#[cfg(feature = "mappy-integration")]
criterion_group!(
    benches,
    benchmark_mappy_storage,
    benchmark_compression_methods
);

#[cfg(not(feature = "mappy-integration"))]
criterion_group!(benches, benchmark_compression_methods);

criterion_main!(benches);
