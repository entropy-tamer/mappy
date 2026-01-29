#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark calculations
//! Accuracy tests comparing exact vs approximate tag retrieval for ML tasks
//!
//! These tests prove that mappy's approximate nature doesn't significantly
//! degrade ML task accuracy when using Huffman-compressed tags.

#[cfg(test)]
#[cfg(feature = "mappy-integration")]
mod tests {
    use crate::benchmark::ml_benchmarks::MLBenchmarkRunner;
    use crate::benchmark::ml_tasks::{MLTaskResults, TagSimilarity};

    #[tokio::test]
    async fn test_similarity_search_accuracy() {
        let tag_sets = MLBenchmarkRunner::generate_ml_test_data(50, 15);
        let query_tags = tag_sets[0].clone();

        let result = MLBenchmarkRunner::benchmark_similarity_search(&tag_sets, &query_tags, 10)
            .await
            .unwrap();

        // Approximate should have >80% accuracy (8/10 top results match)
        assert!(
            result.approximate_accuracy >= 0.8,
            "Similarity search accuracy too low: {:.2}%",
            result.approximate_accuracy * 100.0
        );

        // Speed should be within 2x of exact (no significant degradation)
        assert!(
            result.speed_ratio <= 2.0,
            "Speed degradation too high: {:.2}x",
            result.speed_ratio
        );
    }

    #[tokio::test]
    async fn test_clustering_accuracy() {
        let tag_sets = MLBenchmarkRunner::generate_ml_test_data(30, 10);

        let result = MLBenchmarkRunner::benchmark_clustering(&tag_sets, 3)
            .await
            .unwrap();

        // Cluster assignments should match >70% of the time
        assert!(
            result.approximate_accuracy >= 0.7,
            "Clustering accuracy too low: {:.2}%",
            result.approximate_accuracy * 100.0
        );

        // Speed should be reasonable
        assert!(
            result.speed_ratio <= 3.0,
            "Speed degradation too high: {:.2}x",
            result.speed_ratio
        );
    }

    #[tokio::test]
    async fn test_embeddings_accuracy() {
        let tag_sets = MLBenchmarkRunner::generate_ml_test_data(40, 12);
        let query_tags = tag_sets[0].clone();

        let result = MLBenchmarkRunner::benchmark_embeddings(&tag_sets, &query_tags)
            .await
            .unwrap();

        // Embedding similarity should be >0.9 (very similar)
        assert!(
            result.approximate_accuracy >= 0.9,
            "Embedding similarity too low: {:.2}",
            result.approximate_accuracy
        );

        // Speed should be reasonable
        assert!(
            result.speed_ratio <= 2.5,
            "Speed degradation too high: {:.2}x",
            result.speed_ratio
        );
    }

    #[test]
    fn test_jaccard_similarity_correctness() {
        let tags1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let tags2 = vec!["a".to_string(), "b".to_string(), "d".to_string()];

        let similarity = TagSimilarity::jaccard_similarity(&tags1, &tags2);

        // Intersection: {a, b} = 2, Union: {a, b, c, d} = 4
        // Jaccard = 2/4 = 0.5
        assert!((similarity - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_correctness() {
        let tags1 = vec!["a".to_string(), "b".to_string(), "b".to_string()];
        let tags2 = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let similarity = TagSimilarity::cosine_similarity(&tags1, &tags2);

        // Should be positive similarity
        assert!(similarity > 0.0);
        assert!(similarity <= 1.0);
    }

    #[tokio::test]
    async fn test_comprehensive_ml_benchmarks() {
        let results = MLBenchmarkRunner::run_comprehensive_benchmarks()
            .await
            .unwrap();

        // All tasks should complete successfully
        assert_eq!(results.len(), 4);

        // Check each task meets quality thresholds
        for result in &results {
            println!(
                "Task: {}, Accuracy: {:.2}%, Speed ratio: {:.2}x",
                result.task_name,
                result.approximate_accuracy * 100.0,
                result.speed_ratio
            );

            // Accuracy should be reasonable for all tasks
            match result.task_name.as_str() {
                "similarity_search" => assert!(result.approximate_accuracy >= 0.8),
                "clustering" => assert!(result.approximate_accuracy >= 0.7),
                "embeddings" => assert!(result.approximate_accuracy >= 0.9),
                "classification" => assert!(result.approximate_accuracy >= 0.6),
                _ => {}
            }

            // Speed should not degrade more than 3x
            assert!(
                result.speed_ratio <= 3.0,
                "Task {} has excessive speed degradation: {:.2}x",
                result.task_name,
                result.speed_ratio
            );
        }
    }
}
