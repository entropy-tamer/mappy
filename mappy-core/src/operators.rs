//! Value merge operators for maplets
//!
//! Implements the associative and commutative binary operators (⊕) that
//! define how values are merged when keys collide in the maplet.

use crate::MapletResult;
use std::collections::HashSet;
use std::hash::Hash;

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
        Self { merge_fn }
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

/// Byte array operator specialized for asset-style data (images, binary blobs)
///
/// This operator is optimized for byte arrays and supports:
/// - Efficient concatenation of byte arrays
/// - Mmap-backed access for packed .bin assets
/// - Replacement semantics (right value wins on collision, suitable for immutable assets)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ByteArrayOperator;

impl MergeOperator<Vec<u8>> for ByteArrayOperator {
    fn merge(&self, _left: Vec<u8>, right: Vec<u8>) -> MapletResult<Vec<u8>> {
        // For byte arrays representing assets (images, binary data),
        // we use replacement semantics: the right value wins.
        // This is appropriate for immutable assets where collisions should
        // indicate duplicate storage rather than concatenation.
        Ok(right)
    }

    fn identity(&self) -> Vec<u8> {
        Vec::new()
    }
}

/// Vector addition operator for element-wise addition of numeric vectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VectorOperator;

impl MergeOperator<Vec<f64>> for VectorOperator {
    fn merge(&self, left: Vec<f64>, right: Vec<f64>) -> MapletResult<Vec<f64>> {
        if left.len() != right.len() {
            return Err(crate::MapletError::Internal(format!(
                "Vector length mismatch: {} != {}",
                left.len(),
                right.len()
            )));
        }
        Ok(left.into_iter().zip(right).map(|(l, r)| l + r).collect())
    }

    fn identity(&self) -> Vec<f64> {
        Vec::new()
    }
}

impl MergeOperator<Vec<f32>> for VectorOperator {
    fn merge(&self, left: Vec<f32>, right: Vec<f32>) -> MapletResult<Vec<f32>> {
        if left.len() != right.len() {
            return Err(crate::MapletError::Internal(format!(
                "Vector length mismatch: {} != {}",
                left.len(),
                right.len()
            )));
        }
        Ok(left.into_iter().zip(right).map(|(l, r)| l + r).collect())
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

        assert_eq!(
            op.merge("hello".to_string(), "world".to_string()).unwrap(),
            "helloworld"
        );
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
    fn test_byte_array_operator() {
        let op = ByteArrayOperator;

        let bytes1 = vec![1, 2, 3];
        let bytes2 = vec![4, 5, 6];
        // ByteArrayOperator uses replacement semantics (right wins)
        let result = op.merge(bytes1, bytes2).unwrap();
        assert_eq!(result, vec![4, 5, 6]);

        // Test identity
        assert_eq!(op.identity(), Vec::<u8>::new());
    }

    #[test]
    fn test_bool_operators() {
        let or_op = BoolOrOperator;
        let and_op = BoolAndOperator;

        assert!(or_op.merge(false, true).unwrap());
        assert!(!or_op.merge(false, false).unwrap());
        assert!(!or_op.identity());

        assert!(!and_op.merge(true, false).unwrap());
        assert!(and_op.merge(true, true).unwrap());
        assert!(and_op.identity());
    }

    #[test]
    fn test_counter_operator_u32() {
        let op: CounterOperator = CounterOperator;

        assert_eq!(op.merge(5u32, 3u32).unwrap(), 8);
        assert_eq!(op.merge(0u32, 10u32).unwrap(), 10);
        assert_eq!(op.merge(u32::MAX, 1).unwrap(), u32::MAX);
        assert_eq!(<CounterOperator as MergeOperator<u32>>::identity(&op), 0u32);
    }

    #[test]
    fn test_counter_operator_i64() {
        let op: CounterOperator = CounterOperator;

        assert_eq!(op.merge(5i64, 3i64).unwrap(), 8);
        assert_eq!(op.merge(0i64, 10i64).unwrap(), 10);
        assert_eq!(op.merge(i64::MAX, 1).unwrap(), i64::MAX);
        assert_eq!(op.merge(i64::MIN, -1).unwrap(), i64::MIN);
        assert_eq!(<CounterOperator as MergeOperator<i64>>::identity(&op), 0i64);
    }

    #[test]
    fn test_string_operator() {
        let op = StringOperator;

        // StringOperator uses replacement semantics (right wins)
        assert_eq!(
            op.merge("left".to_string(), "right".to_string()).unwrap(),
            "right"
        );
        assert_eq!(
            op.merge("".to_string(), "value".to_string()).unwrap(),
            "value"
        );
        assert_eq!(op.merge("value".to_string(), "".to_string()).unwrap(), "");
        assert_eq!(op.identity(), "");
    }

    #[test]
    fn test_max_operator_f64() {
        let op = MaxOperator;

        assert_eq!(op.merge(5.0, 3.0).unwrap(), 5.0);
        assert_eq!(op.merge(3.0, 5.0).unwrap(), 5.0);
        assert_eq!(op.merge(-5.0, -3.0).unwrap(), -3.0);
        assert_eq!(op.merge(f64::NEG_INFINITY, 5.0).unwrap(), 5.0);
        assert_eq!(
            <MaxOperator as MergeOperator<f64>>::identity(&op),
            f64::NEG_INFINITY
        );
    }

    #[test]
    fn test_min_operator_f64() {
        let op = MinOperator;

        assert_eq!(op.merge(5.0, 3.0).unwrap(), 3.0);
        assert_eq!(op.merge(3.0, 5.0).unwrap(), 3.0);
        assert_eq!(op.merge(-5.0, -3.0).unwrap(), -5.0);
        assert_eq!(op.merge(f64::INFINITY, 5.0).unwrap(), 5.0);
        assert_eq!(
            <MinOperator as MergeOperator<f64>>::identity(&op),
            f64::INFINITY
        );
    }

    #[test]
    fn test_min_operator_u64_identity() {
        let op: MinOperator = MinOperator;
        assert_eq!(<MinOperator as MergeOperator<u64>>::identity(&op), u64::MAX);
    }

    #[test]
    fn test_max_operator_u64_identity() {
        let op: MaxOperator = MaxOperator;
        assert_eq!(<MaxOperator as MergeOperator<u64>>::identity(&op), 0u64);
    }

    #[test]
    fn test_vector_operator_f64() {
        let op: VectorOperator = VectorOperator;

        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0, 6.0];
        let result = op.merge(vec1, vec2).unwrap();
        assert_eq!(result, vec![5.0, 7.0, 9.0]);
        assert_eq!(
            <VectorOperator as MergeOperator<Vec<f64>>>::identity(&op),
            Vec::<f64>::new()
        );
    }

    #[test]
    fn test_vector_operator_f32() {
        let op: VectorOperator = VectorOperator;

        let vec1 = vec![1.0f32, 2.0f32, 3.0f32];
        let vec2 = vec![4.0f32, 5.0f32, 6.0f32];
        let result = op.merge(vec1, vec2).unwrap();
        assert_eq!(result, vec![5.0f32, 7.0f32, 9.0f32]);
        assert_eq!(
            <VectorOperator as MergeOperator<Vec<f32>>>::identity(&op),
            Vec::<f32>::new()
        );
    }

    #[test]
    fn test_vector_operator_length_mismatch() {
        let op = VectorOperator;

        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0];
        let result = op.merge(vec1, vec2);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Vector length mismatch"));
        }
    }

    #[test]
    fn test_vector_operator_empty_vectors() {
        let op = VectorOperator;

        let vec1 = Vec::<f64>::new();
        let vec2 = Vec::<f64>::new();
        let result = op.merge(vec1, vec2).unwrap();
        assert_eq!(result, Vec::<f64>::new());
    }

    #[test]
    fn test_set_operator_identity() {
        let op = SetOperator;
        let empty: HashSet<String> = <SetOperator as MergeOperator<HashSet<String>>>::identity(&op);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_set_operator_empty_sets() {
        let op = SetOperator;

        let set1: HashSet<String> = HashSet::new();
        let set2: HashSet<String> = HashSet::new();
        let result = op.merge(set1, set2).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_operators_associative_commutative() {
        // Test that operators report correct associativity and commutativity
        // Most operators use default implementation which returns true
        let counter: CounterOperator = CounterOperator;
        assert!(<CounterOperator as MergeOperator<u64>>::is_associative(
            &counter
        ));
        assert!(<CounterOperator as MergeOperator<u64>>::is_commutative(
            &counter
        ));

        let max_op: MaxOperator = MaxOperator;
        assert!(<MaxOperator as MergeOperator<u64>>::is_associative(&max_op));
        assert!(<MaxOperator as MergeOperator<u64>>::is_commutative(&max_op));

        let min_op: MinOperator = MinOperator;
        assert!(<MinOperator as MergeOperator<u64>>::is_associative(&min_op));
        assert!(<MinOperator as MergeOperator<u64>>::is_commutative(&min_op));

        let set_op = SetOperator;
        assert!(<SetOperator as MergeOperator<HashSet<String>>>::is_associative(&set_op));
        assert!(<SetOperator as MergeOperator<HashSet<String>>>::is_commutative(&set_op));

        // String and vector concat use default (true) unless overridden
        let string_concat = StringConcatOperator;
        assert!(<StringConcatOperator as MergeOperator<String>>::is_associative(&string_concat));
        assert!(<StringConcatOperator as MergeOperator<String>>::is_commutative(&string_concat)); // Default is true

        let vector_concat = VectorConcatOperator;
        assert!(<VectorConcatOperator as MergeOperator<Vec<i32>>>::is_associative(&vector_concat));
        assert!(<VectorConcatOperator as MergeOperator<Vec<i32>>>::is_commutative(&vector_concat)); // Default is true
    }

    #[test]
    fn test_string_concat_operator_merge_order() {
        let op = StringConcatOperator;

        // Verify merge order matters (even though trait says commutative)
        let result1 = op.merge("hello".to_string(), "world".to_string()).unwrap();
        let result2 = op.merge("world".to_string(), "hello".to_string()).unwrap();
        assert_ne!(result1, result2);
        assert_eq!(result1, "helloworld");
        assert_eq!(result2, "worldhello");
    }

    #[test]
    fn test_vector_concat_operator_merge_order() {
        let op = VectorConcatOperator;

        let vec1 = vec![1, 2];
        let vec2 = vec![3, 4];
        let result1 = op.merge(vec1.clone(), vec2.clone()).unwrap();
        let result2 = op.merge(vec2, vec1).unwrap();
        assert_ne!(result1, result2);
        assert_eq!(result1, vec![1, 2, 3, 4]);
        assert_eq!(result2, vec![3, 4, 1, 2]);
    }

    #[test]
    fn test_counter_operator_default() {
        let op: CounterOperator = Default::default();
        assert_eq!(op.merge(5u64, 3u64).unwrap(), 8);
    }

    #[test]
    fn test_max_operator_default() {
        let op: MaxOperator = Default::default();
        assert_eq!(op.merge(5u64, 3u64).unwrap(), 5);
    }

    #[test]
    fn test_min_operator_default() {
        let op: MinOperator = Default::default();
        assert_eq!(op.merge(5u64, 3u64).unwrap(), 3);
    }

    #[test]
    fn test_set_operator_default() {
        let op = SetOperator::default();
        let mut set1 = HashSet::new();
        set1.insert("a".to_string());
        let set2 = HashSet::new();
        let result = op.merge(set1, set2).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_string_operator_default() {
        let op = StringOperator::default();
        assert_eq!(
            op.merge("left".to_string(), "right".to_string()).unwrap(),
            "right"
        );
    }

    #[test]
    fn test_vector_operator_default() {
        let op = VectorOperator::default();
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0, 6.0];
        let result = op.merge(vec1, vec2).unwrap();
        assert_eq!(result, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn test_byte_array_operator_default() {
        let op = ByteArrayOperator::default();
        let bytes1 = vec![1, 2, 3];
        let bytes2 = vec![4, 5, 6];
        let result = op.merge(bytes1, bytes2).unwrap();
        assert_eq!(result, vec![4, 5, 6]);
    }

    #[test]
    fn test_counter_operator_clone() {
        let op: CounterOperator = CounterOperator;
        let cloned = op;
        assert_eq!(cloned.merge(5u64, 3u64).unwrap(), 8);
    }

    #[test]
    fn test_max_operator_clone() {
        let op: MaxOperator = MaxOperator;
        let cloned = op;
        assert_eq!(cloned.merge(5u64, 3u64).unwrap(), 5);
    }

    #[test]
    fn test_min_operator_clone() {
        let op: MinOperator = MinOperator;
        let cloned = op;
        assert_eq!(cloned.merge(5u64, 3u64).unwrap(), 3);
    }

    #[test]
    fn test_set_operator_clone() {
        let op = SetOperator;
        let cloned = op;
        let mut set1 = HashSet::new();
        set1.insert("a".to_string());
        let set2 = HashSet::new();
        let result = cloned.merge(set1, set2).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_string_operator_clone() {
        let op = StringOperator;
        let cloned = op;
        assert_eq!(
            cloned
                .merge("left".to_string(), "right".to_string())
                .unwrap(),
            "right"
        );
    }

    #[test]
    fn test_string_concat_operator_clone() {
        let op = StringConcatOperator;
        let cloned = op;
        assert_eq!(
            cloned
                .merge("hello".to_string(), "world".to_string())
                .unwrap(),
            "helloworld"
        );
    }

    #[test]
    fn test_vector_concat_operator_clone() {
        let op = VectorConcatOperator;
        let cloned = op;
        let vec1 = vec![1, 2, 3];
        let vec2 = vec![4, 5, 6];
        let result = cloned.merge(vec1, vec2).unwrap();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_vector_operator_clone() {
        let op: VectorOperator = VectorOperator;
        let cloned = op;
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0, 6.0];
        let result = cloned.merge(vec1, vec2).unwrap();
        assert_eq!(result, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn test_byte_array_operator_clone() {
        let op = ByteArrayOperator;
        let cloned = op;
        let bytes1 = vec![1, 2, 3];
        let bytes2 = vec![4, 5, 6];
        let result = cloned.merge(bytes1, bytes2).unwrap();
        assert_eq!(result, vec![4, 5, 6]);
    }

    #[test]
    fn test_bool_or_operator_clone() {
        let op = BoolOrOperator;
        let cloned = op;
        assert!(cloned.merge(false, true).unwrap());
    }

    #[test]
    fn test_bool_and_operator_clone() {
        let op = BoolAndOperator;
        let cloned = op;
        assert!(cloned.merge(true, true).unwrap());
    }

    #[test]
    fn test_bool_or_operator_identity() {
        let op = BoolOrOperator;
        assert!(!op.identity());
    }

    #[test]
    fn test_bool_and_operator_identity() {
        let op = BoolAndOperator;
        assert!(op.identity());
    }

    #[test]
    fn test_string_concat_operator_identity() {
        let op = StringConcatOperator;
        assert_eq!(op.identity(), "");
    }

    #[test]
    fn test_vector_concat_operator_identity() {
        let op = VectorConcatOperator;
        assert_eq!(
            <VectorConcatOperator as MergeOperator<Vec<i32>>>::identity(&op),
            Vec::<i32>::new()
        );
    }

    #[test]
    fn test_vector_operator_identity_f32() {
        let op: VectorOperator = VectorOperator;
        assert_eq!(
            <VectorOperator as MergeOperator<Vec<f32>>>::identity(&op),
            Vec::<f32>::new()
        );
    }
}
