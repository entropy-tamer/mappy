//! # Mappy Core
//! 
//! Core implementation of maplet data structures for space-efficient approximate key-value mappings.
//! 
//! Based on the research paper "Time To Replace Your Filter: How Maplets Simplify System Design"
//! by Bender, Conway, Farach-Colton, Johnson, and Pandey.

pub mod maplet;
pub mod hash;
pub mod quotient_filter;
pub mod operators;
pub mod deletion;
pub mod resize;
pub mod encoding;
pub mod error;
pub mod layout;
pub mod concurrent;
pub mod types;
pub mod storage;
pub mod engine;
pub mod ttl;

// Re-export main types
pub use maplet::Maplet;
pub use operators::{CounterOperator, SetOperator, StringOperator, MaxOperator, MinOperator, MergeOperator};
pub use types::{MapletStats, MapletError, MapletResult};
pub use engine::{Engine, EngineConfig, EngineStats};
pub use storage::{Storage, StorageStats, StorageConfig, PersistenceMode};
pub use ttl::{TTLManager, TTLConfig, TTLStats, TTLEntry};

/// Common result type for maplet operations
pub type Result<T> = std::result::Result<T, MapletError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_maplet_creation() {
        let maplet = Maplet::<String, u64, CounterOperator>::new(100, 0.01).unwrap();
        assert_eq!(maplet.len().await, 0);
        assert!(maplet.error_rate() <= 0.01);
    }
}
