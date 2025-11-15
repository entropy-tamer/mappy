//! # Mappy Core
//!
//! Core implementation of maplet data structures for space-efficient approximate key-value mappings.
//!
//! Based on the research paper "Time To Replace Your Filter: How Maplets Simplify System Design"
//! by Bender, Conway, Farach-Colton, Johnson, and Pandey.

pub mod concurrent;
pub mod deletion;
pub mod encoding;
pub mod engine;
pub mod error;
pub mod hash;
pub mod layout;
pub mod maplet;
pub mod operators;
pub mod quotient_filter;
pub mod resize;
pub mod storage;
pub mod ttl;
pub mod types;

// Re-export main types
pub use engine::{Engine, EngineConfig, EngineStats};
pub use maplet::Maplet;
pub use operators::{
    CounterOperator, MaxOperator, MergeOperator, MinOperator, SetOperator, StringOperator,
    VectorOperator,
};
pub use storage::{PersistenceMode, Storage, StorageConfig, StorageStats};
pub use ttl::{TTLConfig, TTLEntry, TTLManager, TTLStats};
pub use types::{MapletError, MapletResult, MapletStats};

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
