//! ML task implementations for benchmarking tag operations
//!
//! This module provides ML task implementations that work with compressed tags
//! stored in mappy, demonstrating that the approximate nature of mappy doesn't
//! hurt ML performance.

use std::collections::{HashMap, HashSet};

/// Tag similarity metrics
pub struct TagSimilarity;

impl TagSimilarity {
    /// Calculate Jaccard similarity between two tag sets
    pub fn jaccard_similarity(tags1: &[String], tags2: &[String]) -> f64 {
        let set1: HashSet<&str> = tags1.iter().map(|s| s.as_str()).collect();
        let set2: HashSet<&str> = tags2.iter().map(|s| s.as_str()).collect();
        
        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
    
    /// Calculate cosine similarity between tag frequency vectors
    pub fn cosine_similarity(tags1: &[String], tags2: &[String]) -> f64 {
        let freq1 = Self::tag_frequencies(tags1);
        let freq2 = Self::tag_frequencies(tags2);
        
        let all_tags: HashSet<&str> = freq1.keys()
            .chain(freq2.keys()).copied()
            .collect();
        
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;
        
        for tag in all_tags {
            let count1 = freq1.get(tag).copied().unwrap_or(0) as f64;
            let count2 = freq2.get(tag).copied().unwrap_or(0) as f64;
            
            dot_product += count1 * count2;
            norm1 += count1 * count1;
            norm2 += count2 * count2;
        }
        
        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1.sqrt() * norm2.sqrt())
        }
    }
    
    /// Calculate tag frequency map
    fn tag_frequencies(tags: &[String]) -> HashMap<&str, usize> {
        let mut freq = HashMap::new();
        for tag in tags {
            *freq.entry(tag.as_str()).or_insert(0) += 1;
        }
        freq
    }
    
    /// Find most similar tag sets using Jaccard similarity
    pub fn find_most_similar(
        query_tags: &[String],
        candidate_sets: &[Vec<String>],
        top_k: usize,
    ) -> Vec<(usize, f64)> {
        let mut similarities: Vec<(usize, f64)> = candidate_sets
            .iter()
            .enumerate()
            .map(|(idx, tags)| (idx, Self::jaccard_similarity(query_tags, tags)))
            .collect();
        
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(top_k);
        similarities
    }
}

/// Tag-based classification
pub struct TagClassifier {
    /// Tag sets grouped by class
    class_tags: HashMap<String, Vec<Vec<String>>>,
}

impl TagClassifier {
    /// Create a new classifier from training data
    pub fn new(training_data: Vec<(String, Vec<String>)>) -> Self {
        let mut class_tags: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        
        for (class, tags) in training_data {
            class_tags.entry(class).or_default().push(tags);
        }
        
        Self { class_tags }
    }
    
    /// Classify tags using k-nearest neighbors
    pub fn classify_knn(&self, query_tags: &[String], k: usize) -> Vec<(String, f64)> {
        let mut all_similarities: Vec<(String, f64)> = Vec::new();
        
        for (class, tag_sets) in &self.class_tags {
            for tags in tag_sets {
                let similarity = TagSimilarity::jaccard_similarity(query_tags, tags);
                all_similarities.push((class.clone(), similarity));
            }
        }
        
        // Sort by similarity and get top k
        all_similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Aggregate by class (average similarity)
        let mut class_scores: HashMap<String, (f64, usize)> = HashMap::new();
        for (class, score) in all_similarities.iter().take(k) {
            let entry = class_scores.entry(class.clone()).or_insert((0.0, 0));
            entry.0 += score;
            entry.1 += 1;
        }
        
        let mut results: Vec<(String, f64)> = class_scores
            .into_iter()
            .map(|(class, (sum, count))| (class, sum / count as f64))
            .collect();
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results
    }
}

/// Tag clustering using simple k-means-like approach
pub struct TagClustering;

impl TagClustering {
    /// Cluster tag sets into k clusters
    pub fn cluster(
        tag_sets: &[Vec<String>],
        k: usize,
        max_iterations: usize,
    ) -> Vec<usize> {
        if tag_sets.is_empty() || k == 0 {
            return vec![];
        }
        
        let k = k.min(tag_sets.len());
        let mut assignments = vec![0; tag_sets.len()];
        
        // Initialize centroids randomly
        let mut centroids: Vec<Vec<String>> = Vec::new();
        for i in 0..k {
            if i < tag_sets.len() {
                centroids.push(tag_sets[i].clone());
            }
        }
        
        for _iteration in 0..max_iterations {
            let mut changed = false;
            
            // Assign each tag set to nearest centroid
            for (idx, tags) in tag_sets.iter().enumerate() {
                let mut best_cluster = 0;
                let mut best_similarity = -1.0;
                
                for (cluster_idx, centroid) in centroids.iter().enumerate() {
                    let similarity = TagSimilarity::jaccard_similarity(tags, centroid);
                    if similarity > best_similarity {
                        best_similarity = similarity;
                        best_cluster = cluster_idx;
                    }
                }
                
                if assignments[idx] != best_cluster {
                    changed = true;
                    assignments[idx] = best_cluster;
                }
            }
            
            if !changed {
                break;
            }
            
            // Update centroids (average tags in each cluster)
            for cluster_idx in 0..k {
                let cluster_tags: Vec<&Vec<String>> = tag_sets
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| assignments[*idx] == cluster_idx)
                    .map(|(_, tags)| tags)
                    .collect();
                
                if !cluster_tags.is_empty() {
                    centroids[cluster_idx] = Self::compute_centroid(cluster_tags);
                }
            }
        }
        
        assignments
    }
    
    /// Compute centroid as most common tags in cluster
    fn compute_centroid(cluster_tags: Vec<&Vec<String>>) -> Vec<String> {
        let mut tag_counts: HashMap<&str, usize> = HashMap::new();
        
        for tags in &cluster_tags {
            for tag in *tags {
                *tag_counts.entry(tag.as_str()).or_insert(0) += 1;
            }
        }
        
        // Take top tags that appear in at least 50% of tag sets
        let threshold = cluster_tags.len() / 2;
        let mut centroid: Vec<String> = tag_counts
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .map(|(tag, _)| tag.to_string())
            .collect();
        
        centroid.sort();
        centroid
    }
}

/// Tag embedding generator (simple TF-IDF like approach)
pub struct TagEmbedding;

impl TagEmbedding {
    /// Generate embedding vector for tags
    pub fn embed(tags: &[String], vocabulary: &[String]) -> Vec<f64> {
        let tag_set: HashSet<&str> = tags.iter().map(|s| s.as_str()).collect();
        
        vocabulary
            .iter()
            .map(|vocab_tag| {
                if tag_set.contains(vocab_tag.as_str()) {
                    1.0
                } else {
                    0.0
                }
            })
            .collect()
    }
    
    /// Build vocabulary from multiple tag sets
    pub fn build_vocabulary(tag_sets: &[Vec<String>]) -> Vec<String> {
        let mut vocab_set: HashSet<String> = HashSet::new();
        
        for tags in tag_sets {
            for tag in tags {
                vocab_set.insert(tag.clone());
            }
        }
        
        let mut vocab: Vec<String> = vocab_set.into_iter().collect();
        vocab.sort();
        vocab
    }
    
    /// Calculate embedding similarity (dot product for binary embeddings)
    pub fn embedding_similarity(emb1: &[f64], emb2: &[f64]) -> f64 {
        if emb1.len() != emb2.len() {
            return 0.0;
        }
        
        let dot_product: f64 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f64 = emb1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm2: f64 = emb2.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }
}

/// ML task results for benchmarking
#[derive(Debug, Clone)]
pub struct MLTaskResults {
    pub task_name: String,
    pub exact_accuracy: f64,
    pub approximate_accuracy: f64,
    pub exact_time_ms: f64,
    pub approximate_time_ms: f64,
    pub accuracy_difference: f64,
    pub speed_ratio: f64,
}

impl MLTaskResults {
    pub fn new(
        task_name: String,
        exact_accuracy: f64,
        approximate_accuracy: f64,
        exact_time_ms: f64,
        approximate_time_ms: f64,
    ) -> Self {
        let accuracy_difference = (exact_accuracy - approximate_accuracy).abs();
        let speed_ratio = approximate_time_ms / exact_time_ms.max(0.001);
        
        Self {
            task_name,
            exact_accuracy,
            approximate_accuracy,
            exact_time_ms,
            approximate_time_ms,
            accuracy_difference,
            speed_ratio,
        }
    }
    
    /// Check if approximate results are acceptable (within threshold)
    pub fn is_acceptable(&self, accuracy_threshold: f64, speed_threshold: f64) -> bool {
        self.accuracy_difference <= accuracy_threshold && self.speed_ratio <= speed_threshold
    }
}

