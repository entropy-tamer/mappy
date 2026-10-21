#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mappy_core::hash::HashFunction;
use mappy_core::quotient_filter::QuotientFilter;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::hint::black_box;

/// Benchmark quotient filter insertion performance.
fn bench_quotient_filter_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_insert");

    for size in [1_000usize, 10_000, 100_000] {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("QuotientFilter", size), &size, |b, &s| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(s * 2, 8, HashFunction::AHash).unwrap();

                for _ in 0..s {
                    filter
                        .insert(StdRng::seed_from_u64(42).random::<u64>())
                        .unwrap();
                }

                black_box(filter)
            });
        });
    }

    group.finish();
}

/// Benchmark quotient filter query performance.
fn bench_quotient_filter_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_query");

    for size in [1_000usize, 10_000, 100_000] {
        group.throughput(Throughput::Elements(size as u64));

        // Prepare filter with data.
        let mut values: Vec<u64> = Vec::new();
        for _ in 0..size {
            let val = StdRng::seed_from_u64(42).random::<u64>();
            values.push(val);
        }

        group.bench_with_input(
            BenchmarkId::new("QuotientFilter", size),
            &values,
            |b, vals| {
                b.iter(|| {
                    let filter =
                        QuotientFilter::new(vals.len() * 2, 8, HashFunction::AHash).unwrap();

                    for v in vals {
                        black_box(filter.query(*v));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark quotient filter deletion performance.
fn bench_quotient_filter_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_delete");

    for size in [1_000usize, 10_000, 100_000] {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("QuotientFilter", size), &size, |b, &s| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(s * 2, 8, HashFunction::AHash).unwrap();

                // Insert values.
                for _ in 0..s {
                    filter
                        .insert(StdRng::seed_from_u64(42).random::<u64>())
                        .unwrap();
                }

                black_box(filter)
            });
        });
    }

    group.finish();
}

/// Benchmark different hash functions.
fn bench_hash_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_functions");

    let hashes = [
        ("AHash", HashFunction::AHash),
        ("TwoX", HashFunction::TwoX),
        ("Fnv", HashFunction::Fnv),
    ];

    for (name, hash_fn) in hashes {
        group.bench_with_input(
            BenchmarkId::new(name, 10_000usize),
            &10_000usize,
            |b, &s| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(s * 2, 8, hash_fn).unwrap();

                    for _ in 0..s {
                        filter
                            .insert(StdRng::seed_from_u64(42).random::<u64>())
                            .unwrap();
                    }

                    black_box(filter)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark slot finding performance.
fn bench_slot_finding(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding");

    for size in [1_000usize, 10_000, 100_000] {
        group.throughput(Throughput::Elements(size as u64));

        // Prepare values.
        let mut rng = StdRng::seed_from_u64(42);
        let values: Vec<u64> = (0..size).map(|_| rng.random::<u64>()).collect();

        group.bench_with_input(
            BenchmarkId::new("get_actual_slot", size),
            &values,
            |b, vals| {
                b.iter(|| {
                    for v in vals {
                        let filter =
                            QuotientFilter::new(vals.len() * 2, 8, HashFunction::AHash).unwrap();
                        black_box(filter.get_actual_slot_for_fingerprint(*v));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark multiset operations.
fn bench_multiset_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiset_operations");

    for size in [1_000usize, 10_000, 100_000] {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("multiset", size), &size, |b, &s| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(s * 2, 8, HashFunction::AHash).unwrap();
                let mut rng = StdRng::seed_from_u64(42);

                // Insert values multiple times.
                for _ in 0..s {
                    filter.insert(rng.random_range(0..(s as u64 / 10))).unwrap();
                }

                // Count values.
                for i in 0..(s / 10) {
                    black_box(filter.count(i as u64));
                }

                black_box(filter)
            });
        });
    }

    group.finish();
}

/// Benchmark memory usage.
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    for size in [1_000usize, 10_000, 100_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::new("memory", size), &size, |b, &s| {
            b.iter(|| {
                let filter = QuotientFilter::new(s, 8, HashFunction::AHash).unwrap();
                black_box(filter.stats())
            });
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
