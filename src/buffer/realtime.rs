//! Pre allocated buffer for audio processing

use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::error::{AudioEngineError, Result};
use crate::markers::{HeapFree, NonBlocking, RealtimeSafe};
use crate::types::{ChannelCount, Sample};

/// A pre allocated buffer that never resizes.
///
///
/// This buffer is for real time processing where memory allocation is not allowed. The buffer is allocated once at creation time and provides near to O(1) access to all elements.
///
/// The buffer will implement RealtimeSafe because
///   It should never allocate after constructions
/// all operations are about O(1)
///It willl implmement Send for cross thread usage
#[derive(Clone)]
pub struct RealtimeBuffer<T> {
    /// Pre allocated storage
    data: Box<[T]>,
    /// Number of valid elements that can be less than capacity
    len: usize,
}

impl<T: Clone + Default> RealtimeBuffer<T> {
    /// Creating a new buffer with the specified capacity
    /// All elements are intialized to T::Default
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let data = vec![T::default(); capacity].into_boxed_slice();
        Self { data, len: 0 }
    }

    /// Creating a new buffer with the specified capacitya and fill value
    pub fn with_value(capacity: usize, value: T) -> Self {
        let data = vec![value; capacity].into_boxed_slice();
        Self {
            data,
            len: capacity,
        }
    }

    /// Creating a buffer from an existing boxed slice.
    #[must_use]
    pub fn from_boxed_slice(data: Box<[T]>) -> Self {
        let len = data.len();
        Self { data, len }
    }

    //// Clearing the buffer, setting the length to 0
    ///
    /// This will not deallocate, the capacity remains the same.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Fills the buffer with the default value and sets length to capacity.
    pub fn fill_default(&mut self) {
        self.data.fill(T::default());
        self.len = self.data.len();
    }

    /// Fills the buffer with the given value and sets length to capacity
    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
        self.len = self.data.len()
    }

    /// Resizing the valid portion of the buffer
    ///
    /// If `new_len` is greater than the current length, we fill it with default
    /// If new len is greater than capacity, it will be clamped to capacity

    pub fn resize(&mut self, new_len: usize) {
        let new_len = new_len.min(self.data.len());
        if new_len > self.len {
            self.data[self.len..new_len].fill(T::default());
        }

        self.len = new_len
    }

    /// Copies data from a slice into this buffer
    ///
    /// Errros
    /// Returns an error if the source is larger than the capacity
    pub fn copy_from_slice(&mut self, src: &[T]) -> Result<()> {
        if src.len() > self.data.len() {
            return Err(AudioEngineError::BufferOverflow {
                attempted: src.len(),
                capacity: self.data.len(),
            });
        }
        self.data[..src.len()].clone_from_slice(src);
        self.len = src.len();
        Ok(())
    }
}

impl<T> RealtimeBuffer<T> {
    /// Returning the capacity of the buffer
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of valid elements
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the buffer is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if the buffer is at capacity
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len == self.data.len()
    }

    /// Retrusn the remaining capacity
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len() - self.len
    }

    /// Returns a slice of valid elements
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }

    /// Returns a mutable slice of valid elements
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data[..self.len]
    }

    /// Returns the entire buffer including usused capaciity
    #[must_use]
    pub fn as_full_slice(&mut self) -> &mut [T] {
        &mut self.data[..self.len]
    }

    /// Retruns the entire buffer including unused capacity
    #[must_use]
    pub fn as_full_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the entire buffer as a mutable slice
    #[must_use]
    pub fn as_full_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Getting a reference to an element if in bounds.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            self.data.get(index)
        } else {
            None
        }
    }
}
