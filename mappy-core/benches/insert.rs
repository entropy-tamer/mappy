//! Benchmark for insert operations

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use mappy_core::{Maplet, CounterOperator};

fn bench_insert_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_operations");
    
    for size in &[100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("maplet", size), size, |b, &size| {
            b.iter(|| {
                let mut maplet = Maplet::<String, u64, CounterOperator>::new(size, 0.01).unwrap();
                for i in 0..size {
                    let _ = maplet.insert(format!("key_{i}"), i as u64);
                }
                black_box(maplet);
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_insert_operations);
criterion_main!(benches);




