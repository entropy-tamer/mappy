//! K-mer counting example
//! 
//! Demonstrates using maplets for k-mer counting in computational biology,
//! as described in Section 4 of the research paper.

use mappy_core::{Maplet, CounterOperator};
use mappy_core::types::MapletConfig;
use std::collections::HashMap;

/// Simulate k-mer counting using a maplet
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("K-mer Counting with Maplets");
    println!("===========================");
    
    // Create a maplet for counting k-mers
    let config = MapletConfig::new(10000, 0.001); // 0.1% false positive rate
    let kmer_counter = Maplet::<String, u32, CounterOperator>::with_config(config)?;
    
    // Simulate DNA sequences
    let sequences = vec![
        "ATCGATCGATCG",
        "GCTAGCTAGCTA", 
        "ATCGATCGATCG", // Duplicate sequence
        "TTTTTTTTTTTT",
        "ATCGATCGATCG", // Another duplicate
    ];
    
    let k = 3; // 3-mer counting
    
    println!("Counting {}-mers in {} sequences...", k, sequences.len());
    
    // Count k-mers in each sequence
    for (i, sequence) in sequences.iter().enumerate() {
        println!("Processing sequence {}: {}", i + 1, sequence);
        
        for j in 0..=sequence.len() - k {
            let kmer = &sequence[j..j + k];
            kmer_counter.insert(kmer.to_string(), 1).await?;
        }
    }
    
    // Query some k-mers
    let test_kmers = vec!["ATC", "GAT", "TAG", "TTT", "XYZ"];
    
    println!("\nK-mer counts:");
    for kmer in &test_kmers {
        let count = kmer_counter.query(&kmer.to_string()).await;
        println!("  {}: {:?}", kmer, count);
    }
    
    // Compare with traditional HashMap
    let mut hashmap_counter: HashMap<String, u32> = HashMap::new();
    
    for sequence in &sequences {
        for j in 0..=sequence.len() - k {
            let kmer = &sequence[j..j + k];
            *hashmap_counter.entry(kmer.to_string()).or_insert(0) += 1;
        }
    }
    
    println!("\nComparison with HashMap:");
    for kmer in &test_kmers {
        let maplet_count = kmer_counter.query(&kmer.to_string()).await;
        let hashmap_count = hashmap_counter.get(&kmer.to_string());
        println!("  {}: Maplet={:?}, HashMap={:?}", kmer, maplet_count, hashmap_count);
    }
    
    // Show statistics
    let stats = kmer_counter.stats().await;
    println!("\nMaplet Statistics:");
    println!("  Capacity: {}", stats.capacity);
    println!("  Items: {}", stats.len);
    println!("  Load factor: {:.2}%", stats.load_factor * 100.0);
    println!("  Memory usage: {} bytes", stats.memory_usage);
    println!("  False positive rate: {:.4}", stats.false_positive_rate);
    
    // Calculate memory efficiency
    let hashmap_memory = hashmap_counter.len() * (std::mem::size_of::<String>() + std::mem::size_of::<u32>());
    let memory_savings = (1.0 - stats.memory_usage as f64 / hashmap_memory as f64) * 100.0;
    
    println!("\nMemory Efficiency:");
    println!("  HashMap memory: {} bytes", hashmap_memory);
    println!("  Maplet memory: {} bytes", stats.memory_usage);
    println!("  Memory savings: {:.1}%", memory_savings);
    
    Ok(())
}
