//! Benchmark comparison between maplets and traditional data structures

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use mappy_core::{Maplet, CounterOperator};
use std::collections::HashMap;
use std::hint::black_box;

fn bench_insert_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_comparison");
    
    for size in [100, 1000, 10000].iter() {
        // Benchmark maplet insertions
        group.bench_with_input(BenchmarkId::new("maplet", size), size, |b, size| {
            b.iter(|| {
                let maplet = Maplet::<String, u64, CounterOperator>::new(size * 2, 0.01).unwrap();
                for i in 0..*size {
                    let _ = maplet.insert(format!("key_{}", i), i as u64);
                }
                black_box(maplet)
            })
        });
        
        // Benchmark HashMap insertions
        group.bench_with_input(BenchmarkId::new("hashmap", size), size, |b, size| {
            b.iter(|| {
                let mut hashmap = HashMap::new();
                for i in 0..*size {
                    hashmap.insert(format!("key_{}", i), i as u64);
                }
                black_box(hashmap)
            })
        });
    }
    
    group.finish();
}

fn bench_query_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_comparison");
    
    for size in [100, 1000, 10000].iter() {
        // Prepare maplet
        let mut maplet = Maplet::<String, u64, CounterOperator>::new(*size, 0.01).unwrap();
        for i in 0..*size {
            let _ = maplet.insert(format!("key_{}", i), i as u64);
        }
        
        // Prepare HashMap
        let mut hashmap = HashMap::new();
        for i in 0..*size {
            hashmap.insert(format!("key_{}", i), i as u64);
        }
        
        // Benchmark maplet queries
        group.bench_with_input(BenchmarkId::new("maplet", size), size, |b, size| {
            b.iter(|| {
                for i in 0..*size {
                    black_box(maplet.query(&format!("key_{}", i)));
                }
            })
        });
        
        // Benchmark HashMap queries
        group.bench_with_input(BenchmarkId::new("hashmap", size), size, |b, size| {
            b.iter(|| {
                for i in 0..*size {
                    black_box(hashmap.get(&format!("key_{}", i)));
                }
            })
        });
    }
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    for size in [100, 1000, 10000].iter() {
        // Measure maplet memory usage
        group.bench_with_input(BenchmarkId::new("maplet", size), size, |b, size| {
            b.iter(|| {
                let maplet = Maplet::<String, u64, CounterOperator>::new(size * 2, 0.01).unwrap();
                for i in 0..*size {
                    let _ = maplet.insert(format!("key_{}", i), i as u64);
                }
                // Skip memory usage measurement for now
                black_box(0)
            })
        });
        
        // Measure HashMap memory usage (approximate)
        group.bench_with_input(BenchmarkId::new("hashmap", size), size, |b, size| {
            b.iter(|| {
                let mut hashmap = HashMap::new();
                for i in 0..*size {
                    hashmap.insert(format!("key_{}", i), i as u64);
                }
                let memory_usage = hashmap.len() * (std::mem::size_of::<String>() + std::mem::size_of::<u64>());
                black_box(memory_usage)
            })
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_insert_comparison, bench_query_comparison, bench_memory_usage);
criterion_main!(benches);

