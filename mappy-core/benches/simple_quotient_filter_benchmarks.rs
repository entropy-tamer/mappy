use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use mappy_core::quotient_filter::QuotientFilter;
use mappy_core::hash::HashFunction;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::hint::black_box;

/// Benchmark quotient filter insertion performance
fn bench_quotient_filter_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_insert");
    
    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::new("QuotientFilter", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                let mut rng = StdRng::seed_from_u64(42);
                
                for _ in 0..*size {
                    let value = rng.next_u64();
                    filter.insert(value).unwrap();
                }
                
                black_box(filter)
            })
        });
    }
    
    group.finish();
}

/// Benchmark quotient filter query performance
fn bench_quotient_filter_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_query");
    
    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Prepare filter with data
        let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
        let mut rng = StdRng::seed_from_u64(42);
        let mut test_values = Vec::new();
        
        for _ in 0..*size {
            let value = rng.next_u64();
            filter.insert(value).unwrap();
            test_values.push(value);
        }
        
        group.bench_with_input(BenchmarkId::new("QuotientFilter", size), &test_values, |b, values| {
            b.iter(|| {
                for &value in values {
                    black_box(filter.query(value));
                }
            })
        });
    }
    
    group.finish();
}

/// Benchmark quotient filter deletion performance
fn bench_quotient_filter_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_delete");
    
    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::new("QuotientFilter", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                let mut rng = StdRng::seed_from_u64(42);
                let mut values = Vec::new();
                
                // Insert values
                for _ in 0..*size {
                    let value = rng.next_u64();
                    filter.insert(value).unwrap();
                    values.push(value);
                }
                
                // Delete values
                for value in values {
                    filter.delete(value).unwrap();
                }
                
                black_box(filter)
            })
        });
    }
    
    group.finish();
}

/// Benchmark different hash functions
fn bench_hash_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_functions");
    
    let hash_functions = vec![
        ("AHash", HashFunction::AHash),
        ("TwoX", HashFunction::TwoX),
        ("Fnv", HashFunction::Fnv),
    ];
    
    for (name, hash_fn) in hash_functions {
        group.bench_with_input(BenchmarkId::new(name, 10000), &10000, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 2, 8, hash_fn).unwrap();
                let mut rng = StdRng::seed_from_u64(42);
                
                for _ in 0..*size {
                    let value = rng.next_u64();
                    filter.insert(value).unwrap();
                }
                
                for _ in 0..*size {
                    let value = rng.next_u64();
                    black_box(filter.query(value));
                }
                
                black_box(filter)
            })
        });
    }
    
    group.finish();
}

/// Benchmark slot finding performance
fn bench_slot_finding(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding");
    
    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Prepare filter with data
        let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
        let mut rng = StdRng::seed_from_u64(42);
        let mut test_values = Vec::new();
        
        for _ in 0..*size {
            let value = rng.next_u64();
            filter.insert(value).unwrap();
            test_values.push(value);
        }
        
        group.bench_with_input(BenchmarkId::new("get_actual_slot", size), &test_values, |b, values| {
            b.iter(|| {
                for &value in values {
                    black_box(filter.get_actual_slot_for_fingerprint(value));
                }
            })
        });
    }
    
    group.finish();
}

/// Benchmark multiset operations
fn bench_multiset_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiset_operations");
    
    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::new("multiset", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                let mut rng = StdRng::seed_from_u64(42);
                
                // Insert values multiple times
                for _ in 0..*size {
                    let value = rng.gen::<u64>() % (size / 10) as u64; // Create some duplicates
                    filter.insert(value).unwrap();
                }
                
                // Count values
                for i in 0..(size / 10) {
                    black_box(filter.count(i as u64));
                }
                
                black_box(filter)
            })
        });
    }
    
    group.finish();
}

/// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    for size in [1000, 10000, 100000, 1000000].iter() {
        group.bench_with_input(BenchmarkId::new("memory", size), size, |b, &size| {
            b.iter(|| {
                let filter = QuotientFilter::new(size, 8, HashFunction::AHash).unwrap();
                let stats = filter.stats();
                black_box(stats.capacity)
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_quotient_filter_insert,
    bench_quotient_filter_query,
    bench_quotient_filter_delete,
    bench_hash_functions,
    bench_slot_finding,
    bench_multiset_operations,
    bench_memory_usage
);

criterion_main!(benches);
