#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mappy_core::hash::HashFunction;
use mappy_core::quotient_filter::QuotientFilter;
use std::hint::black_box;

/// Benchmark quotient filter insertion performance
fn bench_quotient_filter_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_insert");

    for size in [1000, 10000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("QuotientFilter", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(size * 4, 8, HashFunction::AHash).unwrap();

                    for i in 0..size {
                        filter.insert(i as u64).unwrap();
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark quotient filter query performance
fn bench_quotient_filter_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_query");

    for size in [1000, 10000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Prepare filter with data
        let mut filter = QuotientFilter::new(size * 4, 8, HashFunction::AHash).unwrap();

        for i in 0..*size {
            filter.insert(i as u64).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::new("QuotientFilter", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        black_box(filter.query(i as u64));
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark quotient filter deletion performance
fn bench_quotient_filter_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_delete");

    for size in [1000, 10000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("QuotientFilter", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(size * 4, 8, HashFunction::AHash).unwrap();

                    // Insert values
                    for i in 0..size {
                        filter.insert(i as u64).unwrap();
                    }

                    // Delete values
                    for i in 0..size {
                        filter.delete(i as u64).unwrap();
                    }

                    black_box(filter)
                })
            },
        );
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

                for i in 0..size {
                    filter.insert(i as u64).unwrap();
                }

                for i in 0..size {
                    black_box(filter.query(i as u64));
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

    for size in [1000, 10000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Prepare filter with data
        let mut filter = QuotientFilter::new(size * 4, 8, HashFunction::AHash).unwrap();

        for i in 0..*size {
            filter.insert(i as u64).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::new("get_actual_slot", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        black_box(filter.get_actual_slot_for_fingerprint(i as u64));
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark multiset operations
fn bench_multiset_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiset_operations");

    for size in [1000, 10000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("multiset", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 4, 8, HashFunction::AHash).unwrap();

                // Insert values multiple times
                for i in 0..size {
                    let value = (i % (size / 10)) as u64; // Create some duplicates
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

    for size in [1000, 10000, 100_000, 1_000_000].iter() {
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
