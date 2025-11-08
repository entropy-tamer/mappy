//! Benchmark for insert operations

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use mappy_core::{Maplet, CounterOperator};
use tokio::runtime::Runtime;

fn bench_insert_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("insert_operations");
    
    for size in &[100, 1000, 10000] {
        let size = *size;
        group.bench_with_input(BenchmarkId::new("maplet", size), &size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let maplet = Maplet::<String, u64, CounterOperator>::new(size, 0.01).unwrap();
                for i in 0..size {
                    maplet.insert(format!("key_{i}"), i as u64).await.unwrap();
                }
                black_box(maplet);
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_insert_operations);
criterion_main!(benches);




