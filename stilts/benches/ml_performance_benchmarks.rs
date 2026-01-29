#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Performance benchmarks for ML tasks with Huffman-compressed mappy storage
//!
//! These benchmarks prove that there are no speed hits when using approximate
//! mappy storage for ML tasks compared to exact storage.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use stilts::benchmark::ml_benchmarks::MLBenchmarkRunner;
use stilts::benchmark::ml_tasks::{TagClustering, TagEmbedding, TagSimilarity};

#[cfg(feature = "mappy-integration")]
use tokio::runtime::Runtime;

fn generate_benchmark_data() -> Vec<Vec<String>> {
    MLBenchmarkRunner::generate_ml_test_data(100, 20)
}

#[cfg(feature = "mappy-integration")]
fn benchmark_similarity_search_exact(c: &mut Criterion) {
    let tag_sets = generate_benchmark_data();
    let query_tags = tag_sets[0].clone();

    c.bench_function("similarity_search_exact", |b| {
        b.iter(|| {
            let results =
                TagSimilarity::find_most_similar(black_box(&query_tags), black_box(&tag_sets), 10);
            black_box(results)
        })
    });
}

#[cfg(feature = "mappy-integration")]
fn benchmark_similarity_search_approximate(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tag_sets = generate_benchmark_data();
    let query_tags = tag_sets[0].clone();

    c.bench_function("similarity_search_approximate", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = MLBenchmarkRunner::benchmark_similarity_search(
                    black_box(&tag_sets),
                    black_box(&query_tags),
                    10,
                )
                .await;
                black_box(result)
            })
        })
    });
}

#[cfg(feature = "mappy-integration")]
fn benchmark_clustering_exact(c: &mut Criterion) {
    let tag_sets = generate_benchmark_data();

    c.bench_function("clustering_exact", |b| {
        b.iter(|| {
            let clusters = TagClustering::cluster(black_box(&tag_sets), 5, 10);
            black_box(clusters)
        })
    });
}

#[cfg(feature = "mappy-integration")]
fn benchmark_clustering_approximate(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tag_sets = generate_benchmark_data();

    c.bench_function("clustering_approximate", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = MLBenchmarkRunner::benchmark_clustering(black_box(&tag_sets), 5).await;
                black_box(result)
            })
        })
    });
}

#[cfg(feature = "mappy-integration")]
fn benchmark_embeddings_exact(c: &mut Criterion) {
    let tag_sets = generate_benchmark_data();
    let query_tags = tag_sets[0].clone();
    let vocabulary = TagEmbedding::build_vocabulary(&tag_sets);

    c.bench_function("embeddings_exact", |b| {
        b.iter(|| {
            let emb = TagEmbedding::embed(black_box(&query_tags), black_box(&vocabulary));
            black_box(emb)
        })
    });
}

#[cfg(feature = "mappy-integration")]
fn benchmark_embeddings_approximate(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tag_sets = generate_benchmark_data();
    let query_tags = tag_sets[0].clone();

    c.bench_function("embeddings_approximate", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = MLBenchmarkRunner::benchmark_embeddings(
                    black_box(&tag_sets),
                    black_box(&query_tags),
                )
                .await;
                black_box(result)
            })
        })
    });
}

#[cfg(feature = "mappy-integration")]
fn benchmark_ml_tasks_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tag_sets = generate_benchmark_data();
    let query_tags = tag_sets[0].clone();

    let mut group = c.benchmark_group("ml_tasks_comparison");

    // Similarity search
    group.bench_function("similarity_exact", |b| {
        b.iter(|| TagSimilarity::find_most_similar(&query_tags, &tag_sets, 10))
    });

    group.bench_function("similarity_approximate", |b| {
        b.iter(|| {
            rt.block_on(async {
                MLBenchmarkRunner::benchmark_similarity_search(&tag_sets, &query_tags, 10).await
            })
        })
    });

    // Clustering
    group.bench_function("clustering_exact", |b| {
        b.iter(|| TagClustering::cluster(&tag_sets, 5, 10))
    });

    group.bench_function("clustering_approximate", |b| {
        b.iter(|| {
            rt.block_on(async { MLBenchmarkRunner::benchmark_clustering(&tag_sets, 5).await })
        })
    });

    group.finish();
}

#[cfg(feature = "mappy-integration")]
criterion_group!(
    benches,
    benchmark_similarity_search_exact,
    benchmark_similarity_search_approximate,
    benchmark_clustering_exact,
    benchmark_clustering_approximate,
    benchmark_embeddings_exact,
    benchmark_embeddings_approximate,
    benchmark_ml_tasks_comparison
);

#[cfg(not(feature = "mappy-integration"))]
criterion_group!(benches);

criterion_main!(benches);
