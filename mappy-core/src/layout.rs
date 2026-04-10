//! Cache-friendly memory layout optimization
//!
//! Implements memory layout optimizations for better cache locality.

use crate::MapletResult;

/// Memory layout optimizer
#[derive(Debug, Clone)]
pub struct LayoutOptimizer {
    /// Cache line size in bytes
    cache_line_size: usize,
    /// Whether to use interleaved layout
    #[allow(dead_code)]
    use_interleaved: bool,
}

impl LayoutOptimizer {
    /// Create a new layout optimizer
    #[must_use]
    pub const fn new(cache_line_size: usize) -> Self {
        Self {
            cache_line_size,
            use_interleaved: true,
        }
    }

    /// Calculate optimal alignment for a type
    #[must_use]
    pub fn calculate_alignment<T>(&self) -> usize {
        std::cmp::max(std::mem::align_of::<T>(), self.cache_line_size)
    }

    /// Calculate padding needed for alignment
    #[must_use]
    pub const fn calculate_padding(&self, size: usize, alignment: usize) -> usize {
        (alignment - (size % alignment)) % alignment
    }

    /// Check if layout is cache-friendly
    #[must_use]
    pub const fn is_cache_friendly<T>(&self, data: &[T]) -> bool {
        let size = std::mem::size_of_val(data);
        size <= self.cache_line_size
    }
}

/// Interleaved storage for better cache locality
#[derive(Debug, Clone)]
pub struct InterleavedStorage<T: Clone> {
    /// Interleaved data
    data: Vec<T>,
    /// Number of elements per cache line
    #[allow(dead_code)]
    elements_per_line: usize,
}

impl<T: Clone> InterleavedStorage<T> {
    /// Create new interleaved storage
    #[must_use]
    pub fn new(capacity: usize, cache_line_size: usize) -> Self {
        let element_size = std::mem::size_of::<T>();
        let elements_per_line = cache_line_size / element_size;

        Self {
            data: Vec::with_capacity(capacity),
            elements_per_line,
        }
    }

    /// Get element at index
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Set element at index
    ///
    /// # Errors
    /// Returns an error if the operation fails.
    pub fn set(&mut self, index: usize, value: T) -> MapletResult<()> {
        if index >= self.data.len() {
            self.data.resize(index + 1, unsafe { std::mem::zeroed() });
        }
        self.data[index] = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_optimizer() {
        let optimizer = LayoutOptimizer::new(64);

        let alignment = optimizer.calculate_alignment::<u64>();
        assert!(alignment >= 8);

        let padding = optimizer.calculate_padding(10, 8);
        assert_eq!(padding, 6);
    }

    #[test]
    fn test_layout_optimizer_alignment_u8() {
        let optimizer = LayoutOptimizer::new(64);
        let alignment = optimizer.calculate_alignment::<u8>();
        assert!(alignment >= 1);
    }

    #[test]
    fn test_layout_optimizer_alignment_u32() {
        let optimizer = LayoutOptimizer::new(64);
        let alignment = optimizer.calculate_alignment::<u32>();
        assert!(alignment >= 4);
    }

    #[test]
    fn test_layout_optimizer_padding_zero() {
        let optimizer = LayoutOptimizer::new(64);
        let padding = optimizer.calculate_padding(8, 8);
        assert_eq!(padding, 0);
    }

    #[test]
    fn test_layout_optimizer_padding_max() {
        let optimizer = LayoutOptimizer::new(64);
        let padding = optimizer.calculate_padding(7, 8);
        assert_eq!(padding, 1);
    }

    #[test]
    fn test_layout_optimizer_cache_friendly() {
        let optimizer = LayoutOptimizer::new(64);
        let data = vec![1u8, 2, 3];
        assert!(optimizer.is_cache_friendly(&data));
    }

    #[test]
    fn test_layout_optimizer_not_cache_friendly() {
        let optimizer = LayoutOptimizer::new(64);
        let data = vec![0u8; 100];
        assert!(!optimizer.is_cache_friendly(&data));
    }

    #[test]
    fn test_layout_optimizer_clone() {
        let optimizer = LayoutOptimizer::new(64);
        let cloned = optimizer.clone();
        assert_eq!(cloned.cache_line_size, optimizer.cache_line_size);
    }

    #[test]
    fn test_interleaved_storage() {
        let mut storage = InterleavedStorage::<u64>::new(100, 64);

        assert!(storage.set(0, 42).is_ok());
        assert_eq!(storage.get(0), Some(&42));
    }

    #[test]
    fn test_interleaved_storage_get_none() {
        let storage = InterleavedStorage::<u64>::new(100, 64);
        assert_eq!(storage.get(0), None);
    }

    #[test]
    fn test_interleaved_storage_set_multiple() {
        let mut storage = InterleavedStorage::<u64>::new(100, 64);

        assert!(storage.set(0, 1).is_ok());
        assert!(storage.set(1, 2).is_ok());
        assert!(storage.set(2, 3).is_ok());

        assert_eq!(storage.get(0), Some(&1));
        assert_eq!(storage.get(1), Some(&2));
        assert_eq!(storage.get(2), Some(&3));
    }

    #[test]
    fn test_interleaved_storage_set_resize() {
        let mut storage = InterleavedStorage::<u64>::new(10, 64);

        assert!(storage.set(5, 42).is_ok());
        assert_eq!(storage.get(5), Some(&42));
    }

    #[test]
    fn test_interleaved_storage_clone() {
        let mut storage = InterleavedStorage::<u64>::new(100, 64);
        storage.set(0, 42).ok();

        let cloned = storage.clone();
        assert_eq!(cloned.get(0), Some(&42));
    }

    #[test]
    fn test_interleaved_storage_u8() {
        let mut storage = InterleavedStorage::<u8>::new(100, 64);

        assert!(storage.set(0, 255).is_ok());
        assert_eq!(storage.get(0), Some(&255));
    }
}
