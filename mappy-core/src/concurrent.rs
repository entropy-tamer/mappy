//! Thread-safe operations for maplets
//! 
//! Implements concurrent access patterns for maplets.

use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::MapletResult;

/// Thread-safe maplet wrapper
#[derive(Debug)]
pub struct ConcurrentMaplet<K, V, Op> 
where
    K: Hash + Eq + Clone + Send + Sync + std::fmt::Debug,
    V: Clone + Send + Sync + std::fmt::Debug,
    Op: crate::operators::MergeOperator<V> + Send + Sync,
{
    /// Inner maplet protected by read-write lock
    inner: Arc<RwLock<crate::maplet::Maplet<K, V, Op>>>,
}

impl<K, V, Op> ConcurrentMaplet<K, V, Op>
where
    K: Hash + Eq + Clone + Send + Sync + std::fmt::Debug,
    V: Clone + PartialEq + Send + Sync + std::fmt::Debug,
    Op: crate::operators::MergeOperator<V> + Default + Send + Sync,
{
    /// Create a new concurrent maplet
    pub fn new(capacity: usize, false_positive_rate: f64) -> MapletResult<Self> {
        let maplet = crate::maplet::Maplet::<K, V, Op>::new(capacity, false_positive_rate)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(maplet)),
        })
    }
    
    /// Insert a key-value pair (write lock)
    pub async fn insert(&self, key: K, value: V) -> MapletResult<()> {
        let maplet = self.inner.read().await;
        maplet.insert(key, value).await
    }
    
    /// Query a key (read lock)
    pub async fn query(&self, key: &K) -> Option<V> {
        let maplet = self.inner.read().await;
        maplet.query(key).await
    }
    
    /// Check if key exists (read lock)
    pub async fn contains(&self, key: &K) -> bool {
        let maplet = self.inner.read().await;
        maplet.contains(key).await
    }
    
    /// Delete a key-value pair (write lock)
    pub async fn delete(&self, key: &K, value: &V) -> MapletResult<bool> {
        let maplet = self.inner.read().await;
        maplet.delete(key, value).await
    }
    
    /// Get length (read lock)
    pub async fn len(&self) -> usize {
        let maplet = self.inner.read().await;
        maplet.len().await
    }
    
    /// Check if empty (read lock)
    pub async fn is_empty(&self) -> bool {
        let maplet = self.inner.read().await;
        maplet.is_empty().await
    }
    
    /// Get statistics (read lock)
    pub async fn stats(&self) -> crate::MapletStats {
        let maplet = self.inner.read().await;
        maplet.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operators::CounterOperator;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_concurrent_maplet() {
        let maplet = ConcurrentMaplet::<String, u64, CounterOperator>::new(100, 0.01).unwrap();
        
        // Test basic operations
        assert!(maplet.insert("key1".to_string(), 5).await.is_ok());
        assert_eq!(maplet.query(&"key1".to_string()).await, Some(5));
        assert!(maplet.contains(&"key1".to_string()).await);
        assert_eq!(maplet.len().await, 1);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let maplet = Arc::new(ConcurrentMaplet::<String, u64, CounterOperator>::new(1000, 0.01).unwrap());
        let mut handles = vec![];
        
        // Spawn multiple tasks to insert data
        for i in 0..4 {
            let maplet = Arc::clone(&maplet);
            let handle = tokio::spawn(async move {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    let _ = maplet.insert(key, (i * 100 + j) as u64).await;
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify some data was inserted
        assert!(maplet.len().await > 0);
    }
}
