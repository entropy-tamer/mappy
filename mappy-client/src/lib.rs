//! # Mappy Client
//! 
//! Client library for mappy maplet data structures.

pub use mappy_core::*;

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use mappy_core::{
        Maplet, CounterOperator, SetOperator, MaxOperator, MinOperator,
        MapletStats, MapletError, MapletResult,
    };
    pub use mappy_core::types::MapletConfig;
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[tokio::test]
    async fn test_client_basic_usage() {
        let maplet = Maplet::<String, u64, CounterOperator>::new(100, 0.01).unwrap();
        
        maplet.insert("test".to_string(), 42).await.unwrap();
        assert_eq!(maplet.query(&"test".to_string()).await, Some(42));
    }
}
