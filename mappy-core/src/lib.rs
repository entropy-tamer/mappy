//! # Mappy Core
//!
//! Core implementation of maplet data structures for space-efficient approximate key-value mappings.
//!
//! Based on the research paper "Time To Replace Your Filter: How Maplets Simplify System Design"
//! by Bender, Conway, Farach-Colton, Johnson, and Pandey.

// The workspace mirrors pin two libc lines through transitive dependencies; this
// crate does not choose either version.
#![allow(clippy::multiple_crate_versions)]

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
    ByteArrayOperator, CounterOperator, MaxOperator, MergeOperator, MinOperator, SetOperator,
    StringOperator, VectorConcatOperator, VectorOperator,
};
pub use storage::aof::{AofCompactionReport, CompactOptions, analyze_aof, compact_aof};
pub use storage::{PersistenceMode, Storage, StorageConfig, StorageStats};
pub use ttl::{TTLConfig, TTLEntry, TTLManager, TTLStats};
// `MapletConfig` lives in `types` and is part of the primary configuration surface:
// `EngineConfig` has a `maplet: MapletConfig` field, so a consumer that cannot name the
// type cannot construct an `EngineConfig` literal. It was reachable only as
// `mappy_core::types::MapletConfig`, which is why mappy-client's own `prelude` had to
// re-export it by path. Re-exported here alongside its siblings.
pub use types::{MapletConfig, MapletError, MapletResult, MapletStats};

/// Common result type for maplet operations
pub type Result<T> = std::result::Result<T, MapletError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_basic_maplet_creation() {
        let maplet = Arc::new(Maplet::<String, u64, CounterOperator>::new(100, 0.01).unwrap());
        assert_eq!(Arc::clone(&maplet).len().await, 0);
        assert!(maplet.error_rate() <= 0.01);
    }
}
