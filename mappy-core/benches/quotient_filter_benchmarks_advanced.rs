#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use mappy_core::maplet::{CounterOperator, Maplet};
use mappy_core::quotient_filter::{HashFunction, QuotientFilter};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tokio::runtime::Runtime;

/// Benchmark advanced slot finding performance
fn bench_advanced_slot_finding(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("advanced_slot_finding");

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("get_actual_slot", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                    let mut rng = StdRng::seed_from_u64(42);
                    let mut test_values = Vec::new();

                    // Insert values
                    for _ in 0..size {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
                        test_values.push(value);
                    }

                    // Test slot finding
                    for &value in &test_values {
                        black_box(filter.get_actual_slot_for_fingerprint(value));
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark maplet slot finding performance
fn bench_maplet_slot_finding(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("maplet_slot_finding");

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("find_slot_for_key", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let maplet =
                        Maplet::<String, u64, CounterOperator>::new(size * 2, 0.01).unwrap();
                    let mut rng = StdRng::seed_from_u64(42);
                    let mut test_keys = Vec::new();

                    // Insert keys
                    for _ in 0..size {
                        let key = format!("key_{}", rng.r#gen::<u64>());
                        maplet
                            .insert(key.clone(), rng.r#gen::<u64>())
                            .await
                            .unwrap();
                        test_keys.push(key);
                    }

                    // Test slot finding
                    for key in &test_keys {
                        black_box(maplet.find_slot_for_key(key).await);
                    }

                    black_box(maplet)
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
                for _ in 0..size {
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

/// Benchmark slot finding with different hash functions
fn bench_slot_finding_hash_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding_hash_functions");

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
                let mut test_values = Vec::new();

                // Insert values
                for _ in 0..size {
                    let value = rng.r#gen::<u64>();
                    filter.insert(value).unwrap();
                    test_values.push(value);
                }

                // Test slot finding
                for &value in &test_values {
                    black_box(filter.get_actual_slot_for_fingerprint(value));
                }

                black_box(filter)
            })
        });
    }

    group.finish();
}

/// Benchmark slot finding with different remainder bit sizes
fn bench_slot_finding_remainder_bits(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding_remainder_bits");

    let remainder_bits = vec![4, 8, 12, 16];

    for bits in remainder_bits {
        group.bench_with_input(
            BenchmarkId::new("remainder_bits", bits),
            &bits,
            |b, &bits| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(10000, bits, HashFunction::AHash).unwrap();
                    let mut rng = StdRng::seed_from_u64(42);
                    let mut test_values = Vec::new();

                    // Insert values
                    for _ in 0..5000 {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
                        test_values.push(value);
                    }

                    // Test slot finding
                    for &value in &test_values {
                        black_box(filter.get_actual_slot_for_fingerprint(value));
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark slot finding with collisions
fn bench_slot_finding_collisions(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding_collisions");

    for collision_factor in [1, 2, 4, 8].iter() {
        let capacity = 1000;
        let size = capacity / collision_factor;

        group.bench_with_input(
            BenchmarkId::new("collision_factor", collision_factor),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut filter = QuotientFilter::new(capacity, 4, HashFunction::AHash).unwrap(); // Small remainder bits to force collisions
                    let mut rng = StdRng::seed_from_u64(42);
                    let mut test_values = Vec::new();

                    // Insert values
                    for _ in 0..size {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
                        test_values.push(value);
                    }

                    // Test slot finding
                    for &value in &test_values {
                        black_box(filter.get_actual_slot_for_fingerprint(value));
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark slot finding with multiset operations
fn bench_slot_finding_multiset(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding_multiset");

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("multiset", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                let mut rng = StdRng::seed_from_u64(42);
                let mut test_values = Vec::new();

                // Insert values multiple times
                for _ in 0..size {
                    let value = rng.r#gen::<u64>() % (size / 10); // Create some duplicates
                    filter.insert(value).unwrap();
                    test_values.push(value);
                }

                // Test slot finding
                for &value in &test_values {
                    black_box(filter.get_actual_slot_for_fingerprint(value));
                }

                black_box(filter)
            })
        });
    }

    group.finish();
}

/// Benchmark slot finding with concurrent operations
fn bench_slot_finding_concurrent(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("slot_finding_concurrent");

    for num_threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent", num_threads),
            num_threads,
            |b, &num_threads| {
                b.to_async(&rt).iter(|| async {
                    use std::sync::Arc;
                    use tokio::task;

                    let filter =
                        Arc::new(QuotientFilter::new(10000, 8, HashFunction::AHash).unwrap());
                    let mut handles = vec![];

                    // Spawn multiple tasks that insert values and find slots
                    for thread_id in 0..num_threads {
                        let filter_clone = Arc::clone(&filter);
                        let handle = task::spawn(async move {
                            for i in 0..1000 {
                                let value = thread_id * 1000 + i;
                                filter_clone.insert(value).unwrap();

                                // Test slot finding
                                black_box(filter_clone.get_actual_slot_for_fingerprint(value));
                            }
                        });
                        handles.push(handle);
                    }

                    // Wait for all tasks to complete
                    for handle in handles {
                        handle.await.unwrap();
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark slot finding accuracy
fn bench_slot_finding_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding_accuracy");

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::new("accuracy", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size * 2, 8, HashFunction::AHash).unwrap();
                let mut rng = StdRng::seed_from_u64(42);
                let mut value_to_slot = std::collections::HashMap::new();

                // Insert values and track their slots
                for _ in 0..size {
                    let value = rng.r#gen::<u64>();
                    filter.insert(value).unwrap();
                    if let Some(slot) = filter.get_actual_slot_for_fingerprint(value) {
                        value_to_slot.insert(value, slot);
                    }
                }

                // Test slot finding consistency
                let mut consistent = 0;
                for (value, expected_slot) in &value_to_slot {
                    if let Some(actual_slot) = filter.get_actual_slot_for_fingerprint(*value) {
                        if actual_slot == *expected_slot {
                            consistent += 1;
                        }
                    }
                }

                black_box(consistent)
            })
        });
    }

    group.finish();
}

/// Benchmark slot finding with stress test
fn bench_slot_finding_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding_stress");

    for size in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("stress", size), size, |b, &size| {
            b.iter(|| {
                let mut filter = QuotientFilter::new(size, 8, HashFunction::AHash).unwrap();
                let mut rng = StdRng::seed_from_u64(42);
                let mut test_values = Vec::new();

                // Insert random values
                for _ in 0..size {
                    let value = rng.r#gen::<u64>();
                    filter.insert(value).unwrap();
                    test_values.push(value);
                }

                // Test slot finding for all values
                for &value in &test_values {
                    black_box(filter.get_actual_slot_for_fingerprint(value));
                }

                // Test slot finding for random values (some might not exist)
                for _ in 0..(size / 10) {
                    let value = rng.r#gen::<u64>();
                    black_box(filter.get_actual_slot_for_fingerprint(value));
                }

                black_box(filter)
            })
        });
    }

    group.finish();
}

/// Benchmark slot finding with different load factors
fn bench_slot_finding_load_factor(c: &mut Criterion) {
    let mut group = c.benchmark_group("slot_finding_load_factor");

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
                    let mut test_values = Vec::new();

                    // Insert values
                    for _ in 0..size {
                        let value = rng.r#gen::<u64>();
                        filter.insert(value).unwrap();
                        test_values.push(value);
                    }

                    // Test slot finding
                    for &value in &test_values {
                        black_box(filter.get_actual_slot_for_fingerprint(value));
                    }

                    black_box(filter)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_advanced_slot_finding,
    bench_maplet_slot_finding,
    bench_run_detection,
    bench_slot_finding_hash_functions,
    bench_slot_finding_remainder_bits,
    bench_slot_finding_collisions,
    bench_slot_finding_multiset,
    bench_slot_finding_concurrent,
    bench_slot_finding_accuracy,
    bench_slot_finding_stress,
    bench_slot_finding_load_factor
);

criterion_main!(benches);
