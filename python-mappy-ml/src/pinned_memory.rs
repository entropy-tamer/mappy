//! Pinned memory allocation for efficient GPU transfers
//! 
//! Provides pinned CPU memory buffers that can be reused for batch transfers

use std::sync::{Arc, Mutex};

/// Pinned memory buffer for efficient GPU transfers
/// 
/// Pinned memory is page-locked memory that enables faster CPU->GPU transfers
/// via DMA (Direct Memory Access). This buffer can be reused across batches.
pub struct PinnedBuffer {
    data: Vec<f32>,
    size: usize,
}

impl PinnedBuffer {
    /// Create a new pinned buffer of the specified size
    /// 
    /// Note: True pinned memory requires platform-specific allocation.
    /// For now, we use regular Vec which can be pinned by PyTorch.
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0.0f32; size],
            size,
        }
    }
    
    /// Get mutable slice to the buffer
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data
    }
    
    /// Get immutable slice to the buffer
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }
    
    /// Get the size of the buffer
    pub fn size(&self) -> usize {
        self.size
    }
}

/// Pool of pinned buffers for reuse
pub struct PinnedBufferPool {
    buffers: Arc<Mutex<Vec<PinnedBuffer>>>,
    buffer_size: usize,
}

impl PinnedBufferPool {
    /// Create a new buffer pool
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(Vec::new())),
            buffer_size,
        }
    }
    
    /// Get a buffer from the pool (or create a new one if pool is empty)
    pub fn acquire(&self) -> PinnedBuffer {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.pop().unwrap_or_else(|| PinnedBuffer::new(self.buffer_size))
    }
    
    /// Return a buffer to the pool for reuse
    pub fn release(&self, buffer: PinnedBuffer) {
        // Only keep buffers of the correct size
        if buffer.size() == self.buffer_size {
            let mut buffers = self.buffers.lock().unwrap();
            buffers.push(buffer);
        }
    }
    
    /// Clear all buffers from the pool
    pub fn clear(&self) {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.clear();
    }
}

