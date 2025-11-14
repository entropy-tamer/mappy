//! Comparison benchmarks with external libraries

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stilts::benchmark::{BenchmarkRunner, ComparisonRunner};

fn generate_test_data() -> Vec<u8> {
    let tags = vec![
        "2007", "3_toes", "4_fingers", "anthro", "biped",
        "black_and_white", "breasts", "canid", "canine", "claws",
    ];
    
    tags.iter()
        .flat_map(|t| t.as_bytes())
        .chain(std::iter::once(b' '))
        .collect()
}

fn benchmark_stilts_algorithms(c: &mut Criterion) {
    let tags: Vec<String> = vec![
        "2007".to_string(), "3_toes".to_string(), "4_fingers".to_string(),
        "anthro".to_string(), "biped".to_string(),
    ];
    
    c.bench_function("stilts_all_algorithms", |b| {
        b.iter(|| BenchmarkRunner::benchmark_all(black_box(&tags), 1))
    });
}

fn benchmark_external_libraries(c: &mut Criterion) {
    let data = generate_test_data();
    
    c.bench_function("external_all_libraries", |b| {
        b.iter(|| ComparisonRunner::compare_all(black_box(&data), 1))
    });
}

criterion_group!(benches, benchmark_stilts_algorithms, benchmark_external_libraries);
criterion_main!(benches);


