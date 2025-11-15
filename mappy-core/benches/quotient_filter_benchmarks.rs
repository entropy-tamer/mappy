#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mappy_core::hash::HashFunction;
use mappy_core::quotient_filter::QuotientFilter;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use std::hint::black_box;

/// Benchmark quotient filter insertion performance
fn bench_quotient_filter_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_insert");

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("QuotientFilter", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                    let mut rng = StdRng::seed_from_u64(42);

                    for _ in 0..*size {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
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

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Prepare filter with data
        let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
        let mut rng = StdRng::seed_from_u64(42);
        let mut test_values = Vec::new();

        for _ in 0..size {
            let value = rng.r#gen::<u64>();
            filter.insert(value).unwrap();
            test_values.push(value);
        }

        group.bench_with_input(
            BenchmarkId::new("QuotientFilter", size),
            &test_values,
            |b, values| {
                b.iter(|| {
                    for &value in values {
                        black_box(filter.query(value));
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

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("QuotientFilter", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                    let mut rng = StdRng::seed_from_u64(42);
                    let mut values = Vec::new();

                    // Insert values
                    for _ in 0..*size {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
                        values.push(value);
                    }

                    // Delete values
                    for value in values {
                        filter.delete(value).unwrap();
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark quotient filter vs HashSet performance
fn bench_quotient_filter_vs_hashset(c: &mut Criterion) {
    let mut group = c.benchmark_group("quotient_filter_vs_hashset");

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Benchmark QuotientFilter
        group.bench_with_input(
            BenchmarkId::new("QuotientFilter", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                    let mut rng = StdRng::seed_from_u64(42);

                    for _ in 0..*size {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
                    }

                    for _ in 0..*size {
                        let value = rng.r#gen::<u64>();
                        black_box(filter.query(value));
                    }

                    black_box(filter)
                })
            },
        );

        // Benchmark HashSet
        group.bench_with_input(BenchmarkId::new("HashSet", size), size, |b, &size| {
            b.iter(|| {
                let mut set = HashSet::new();
                let mut rng = StdRng::seed_from_u64(42);

                for _ in 0..*size {
                    let value = rng.r#gen::<u64>();
                    set.insert(value);
                }

                for _ in 0..*size {
                    let value = rng.r#gen::<u64>();
                    black_box(set.contains(&value));
                }

                black_box(set)
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
        ("FNV", HashFunction::FNV),
    ];

    for (name, hash_fn) in hash_functions {
        group.bench_with_input(BenchmarkId::new(name, 10000), &10000, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 2, 8, hash_fn).unwrap();
                let mut rng = StdRng::seed_from_u64(42);

                for _ in 0..*size {
                    let value = rng.r#gen::<u64>();
                    filter.insert(value).unwrap();
                }

                for _ in 0..*size {
                    let value = rng.r#gen::<u64>();
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

        for _ in 0..size {
            let value = rng.r#gen::<u64>();
            filter.insert(value).unwrap();
            test_values.push(value);
        }

        group.bench_with_input(
            BenchmarkId::new("get_actual_slot", size),
            &test_values,
            |b, values| {
                b.iter(|| {
                    for &value in values {
                        black_box(filter.get_actual_slot_for_fingerprint(value));
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

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("multiset", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                let mut rng = StdRng::seed_from_u64(42);

                // Insert values multiple times
                for _ in 0..*size {
                    let value = rng.r#gen::<u64>() % (*size / 10) as u64; // Create some duplicates
                    filter.insert(value).unwrap();
                }

                // Count values
                for i in 0..(*size / 10) {
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

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    for num_threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent", num_threads),
            num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    use std::sync::Arc;
                    use std::thread;

                    let filter =
                        Arc::new(QuotientFilter::new(10000, 8, HashFunction::AHash).unwrap());
                    let mut handles = vec![];

                    for thread_id in 0..num_threads {
                        let filter_clone = Arc::clone(&filter);
                        let handle = thread::spawn(move || {
                            for i in 0..1000 {
                                let value = thread_id * 1000 + i;
                                filter_clone.insert(value).unwrap();
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark false positive rate
fn bench_false_positive_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("false_positive_rate");

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("false_positive", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                    let mut rng = StdRng::seed_from_u64(42);
                    let mut inserted = HashSet::new();

                    // Insert values
                    for _ in 0..*size {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
                        inserted.insert(value);
                    }

                    // Test for false positives
                    let mut false_positives = 0;
                    let mut total_tested = 0;

                    for _ in 0..*size {
                        let value = rng.r#gen::<u64>();
                        if !inserted.contains(&value) {
                            total_tested += 1;
                            if filter.query(value) {
                                false_positives += 1;
                            }
                        }
                    }

                    let rate = if total_tested > 0 {
                        false_positives as f64 / total_tested as f64
                    } else {
                        0.0
                    };

                    black_box(rate)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark load factor impact
fn bench_load_factor_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_factor_impact");

    let capacity = 10000;
    let load_factors = vec![0.1, 0.3, 0.5, 0.7, 0.9];

    for &load_factor in &load_factors {
        let size = (capacity as f64 * load_factor) as usize;

        group.bench_with_input(
            BenchmarkId::new("load_factor", load_factor),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(capacity, 8, HashFunction::AHash).unwrap();
                    let mut rng = StdRng::seed_from_u64(42);

                    // Insert values
                    for _ in 0..*size {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
                    }

                    // Query values
                    for _ in 0..*size {
                        let value = rng.r#gen::<u64>();
                        black_box(filter.query(value));
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark run detection performance
fn bench_run_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_detection");

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("run_detection", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size, 4, HashFunction::AHash).unwrap(); // Small remainder bits to force runs
                let mut rng = StdRng::seed_from_u64(42);

                // Insert values that might create runs
                for _ in 0..*size {
                    let value = rng.r#gen::<u64>();
                    filter.insert(value).unwrap();
                }

                // Get statistics (which includes run detection)
                let stats = filter.stats();
                black_box(stats.runs)
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
    bench_quotient_filter_vs_hashset,
    bench_hash_functions,
    bench_slot_finding,
    bench_multiset_operations,
    bench_memory_usage,
    bench_concurrent_operations,
    bench_false_positive_rate,
    bench_load_factor_impact,
    bench_run_detection
);

criterion_main!(benches);
