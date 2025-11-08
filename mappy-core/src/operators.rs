//! Value merge operators for maplets
//! 
//! Implements the associative and commutative binary operators (⊕) that
//! define how values are merged when keys collide in the maplet.

use std::collections::HashSet;
use std::hash::Hash;
use crate::MapletResult;

/// Trait for merge operators that define how values are combined
pub trait MergeOperator<V>: Clone + Send + Sync {
    /// Merge two values using the operator ⊕
    /// 
    /// # Errors
    /// 
    /// Returns an error if the merge operation fails
    fn merge(&self, left: V, right: V) -> MapletResult<V>;
    
    /// Get the identity element for this operator
    fn identity(&self) -> V;
    
    /// Check if the operator is associative
    fn is_associative(&self) -> bool {
        true // Most operators are associative
    }
    
    /// Check if the operator is commutative
    fn is_commutative(&self) -> bool {
        true // Most operators are commutative
    }
}

/// Counter operator for counting use cases (addition)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CounterOperator;

impl MergeOperator<u64> for CounterOperator {
    fn merge(&self, left: u64, right: u64) -> MapletResult<u64> {
        Ok(left.saturating_add(right))
    }
    
    fn identity(&self) -> u64 {
        0
    }
}

impl MergeOperator<u32> for CounterOperator {
    fn merge(&self, left: u32, right: u32) -> MapletResult<u32> {
        Ok(left.saturating_add(right))
    }
    
    fn identity(&self) -> u32 {
        0
    }
}

impl MergeOperator<i64> for CounterOperator {
    fn merge(&self, left: i64, right: i64) -> MapletResult<i64> {
        Ok(left.saturating_add(right))
    }
    
    fn identity(&self) -> i64 {
        0
    }
}

/// Set operator for set-valued maps (union)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetOperator;

impl<T: Clone + Hash + Eq> MergeOperator<HashSet<T>> for SetOperator {
    fn merge(&self, mut left: HashSet<T>, right: HashSet<T>) -> MapletResult<HashSet<T>> {
        left.extend(right);
        Ok(left)
    }
    
    fn identity(&self) -> HashSet<T> {
        HashSet::new()
    }
}

/// String operator for string-valued maps (replacement)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StringOperator;

impl MergeOperator<String> for StringOperator {
    fn merge(&self, _left: String, right: String) -> MapletResult<String> {
        // For strings, we'll use the right value (replacement semantics)
        Ok(right)
    }
    
    fn identity(&self) -> String {
        String::new()
    }
}

/// Max operator for tracking maximum values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MaxOperator;

impl MergeOperator<u64> for MaxOperator {
    fn merge(&self, left: u64, right: u64) -> MapletResult<u64> {
        Ok(left.max(right))
    }
    
    fn identity(&self) -> u64 {
        0
    }
}

impl MergeOperator<f64> for MaxOperator {
    fn merge(&self, left: f64, right: f64) -> MapletResult<f64> {
        Ok(left.max(right))
    }
    
    fn identity(&self) -> f64 {
        f64::NEG_INFINITY
    }
}

/// Min operator for tracking minimum values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MinOperator;

impl MergeOperator<u64> for MinOperator {
    fn merge(&self, left: u64, right: u64) -> MapletResult<u64> {
        Ok(left.min(right))
    }
    
    fn identity(&self) -> u64 {
        u64::MAX
    }
}

impl MergeOperator<f64> for MinOperator {
    fn merge(&self, left: f64, right: f64) -> MapletResult<f64> {
        Ok(left.min(right))
    }
    
    fn identity(&self) -> f64 {
        f64::INFINITY
    }
}

/// Custom operator that allows user-defined merge logic
#[derive(Clone)]
pub struct CustomOperator<F> {
    #[allow(dead_code)]
    merge_fn: F,
}

impl<F> CustomOperator<F> {
    /// Create a new custom operator
    pub const fn new(merge_fn: F) -> Self {
        Self {
            merge_fn,
        }
    }
}

/// String concatenation operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringConcatOperator;

impl MergeOperator<String> for StringConcatOperator {
    fn merge(&self, left: String, right: String) -> MapletResult<String> {
        Ok(format!("{left}{right}"))
    }
    
    fn identity(&self) -> String {
        String::new()
    }
}

/// Vector concatenation operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VectorConcatOperator;

impl<T: Clone> MergeOperator<Vec<T>> for VectorConcatOperator {
    fn merge(&self, mut left: Vec<T>, right: Vec<T>) -> MapletResult<Vec<T>> {
        left.extend(right);
        Ok(left)
    }
    
    fn identity(&self) -> Vec<T> {
        Vec::new()
    }
}

/// Vector addition operator for element-wise addition of numeric vectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VectorOperator;

impl MergeOperator<Vec<f64>> for VectorOperator {
    fn merge(&self, left: Vec<f64>, right: Vec<f64>) -> MapletResult<Vec<f64>> {
        if left.len() != right.len() {
            return Err(crate::MapletError::Internal(
                format!("Vector length mismatch: {} != {}", left.len(), right.len())
            ));
        }
        Ok(left.into_iter().zip(right.into_iter()).map(|(l, r)| l + r).collect())
    }
    
    fn identity(&self) -> Vec<f64> {
        Vec::new()
    }
}

impl MergeOperator<Vec<f32>> for VectorOperator {
    fn merge(&self, left: Vec<f32>, right: Vec<f32>) -> MapletResult<Vec<f32>> {
        if left.len() != right.len() {
            return Err(crate::MapletError::Internal(
                format!("Vector length mismatch: {} != {}", left.len(), right.len())
            ));
        }
        Ok(left.into_iter().zip(right.into_iter()).map(|(l, r)| l + r).collect())
    }
    
    fn identity(&self) -> Vec<f32> {
        Vec::new()
    }
}

/// Boolean OR operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolOrOperator;

impl MergeOperator<bool> for BoolOrOperator {
    fn merge(&self, left: bool, right: bool) -> MapletResult<bool> {
        Ok(left || right)
    }
    
    fn identity(&self) -> bool {
        false
    }
}

/// Boolean AND operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolAndOperator;

impl MergeOperator<bool> for BoolAndOperator {
    fn merge(&self, left: bool, right: bool) -> MapletResult<bool> {
        Ok(left && right)
    }
    
    fn identity(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_counter_operator() {
        let op: CounterOperator = CounterOperator;
        
        assert_eq!(op.merge(5u64, 3u64).unwrap(), 8);
        assert_eq!(op.merge(0u64, 10u64).unwrap(), 10);
        
        // Test saturation
        assert_eq!(op.merge(u64::MAX, 1).unwrap(), u64::MAX);
    }

    #[test]
    fn test_set_operator() {
        let op = SetOperator;
        
        let mut set1 = HashSet::new();
        set1.insert("a".to_string());
        set1.insert("b".to_string());
        
        let mut set2 = HashSet::new();
        set2.insert("b".to_string());
        set2.insert("c".to_string());
        
        let result = op.merge(set1, set2).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(result.contains("c"));
    }

    #[test]
    fn test_max_operator() {
        let op: MaxOperator = MaxOperator;
        
        assert_eq!(op.merge(5u64, 3u64).unwrap(), 5);
        assert_eq!(op.merge(3u64, 5u64).unwrap(), 5);
        assert_eq!(op.merge(5.0, 3.0).unwrap(), 5.0);
    }

    #[test]
    fn test_min_operator() {
        let op: MinOperator = MinOperator;
        
        assert_eq!(op.merge(5u64, 3u64).unwrap(), 3);
        assert_eq!(op.merge(3u64, 5u64).unwrap(), 3);
        assert_eq!(op.merge(5.0, 3.0).unwrap(), 3.0);
    }

    #[test]
    fn test_string_concat_operator() {
        let op = StringConcatOperator;
        
        assert_eq!(op.merge("hello".to_string(), "world".to_string()).unwrap(), "helloworld");
        assert_eq!(op.identity(), "");
    }

    #[test]
    fn test_vector_concat_operator() {
        let op = VectorConcatOperator;
        
        let vec1 = vec![1, 2, 3];
        let vec2 = vec![4, 5, 6];
        let result = op.merge(vec1, vec2).unwrap();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_bool_operators() {
        let or_op = BoolOrOperator;
        let and_op = BoolAndOperator;
        
        assert_eq!(or_op.merge(false, true).unwrap(), true);
        assert_eq!(or_op.merge(false, false).unwrap(), false);
        assert_eq!(or_op.identity(), false);
        
        assert_eq!(and_op.merge(true, false).unwrap(), false);
        assert_eq!(and_op.merge(true, true).unwrap(), true);
        assert_eq!(and_op.identity(), true);
    }
}
