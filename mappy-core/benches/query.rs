#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Benchmark for query operations

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use mappy_core::{CounterOperator, Maplet};
use std::hint::black_box;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_query_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("query_operations");

    for size in &[100, 1000, 10000] {
        let size = *size;
        // Prepare maplet
        let maplet = rt.block_on(async {
            let maplet = Maplet::<String, u64, CounterOperator>::new(size * 2, 0.01).unwrap();
            for i in 0..size {
                maplet.insert(format!("key_{i}"), i as u64).await.unwrap();
            }
            Arc::new(maplet)
        });
        let maplet_clone = maplet.clone();

        group.bench_with_input(BenchmarkId::new("maplet", size), &size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                for i in 0..size {
                    black_box(maplet_clone.query(&format!("key_{i}")).await);
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_query_operations);
criterion_main!(benches);
