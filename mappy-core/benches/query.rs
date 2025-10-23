//! Benchmark for query operations

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use mappy_core::{Maplet, CounterOperator};

fn bench_query_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_operations");
    
    for size in &[100, 1000, 10000] {
        // Prepare maplet
        let mut maplet = Maplet::<String, u64, CounterOperator>::new(*size * 2, 0.01).unwrap();
        for i in 0..*size {
            let _ = maplet.insert(format!("key_{i}"), i as u64);
        }
        
        group.bench_with_input(BenchmarkId::new("maplet", size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    black_box(maplet.query(&format!("key_{i}")));
                }
            })
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_query_operations);
criterion_main!(benches);

