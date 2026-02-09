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
    /// Returns the capacity of the buffer
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

    /// Gets a mutable reference to an element, if in bounds.
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            self.data.get_mut(index)
        } else {
            None
        }
    }

    /// Sets the length without modifying data.
    ///
    /// Panics,
    /// Panics if new_len is greater than the capacity
    pub fn set_len(&mut self, new_len: usize) {
        assert!(
            new_len <= self.data.len(),
            "new_len ({new_len}) exceeds capacity ({})",
            self.data.len()
        );
        self.len = new_len;
    }

    /// Returns an iterator over valid elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data[..self.len].iter()
    }

    /// Returns a mutable iterator over valid elements
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data[..self.len].iter_mut()
    }
}

impl<T: Send + 'static> RealtimeSafe for RealtimeBuffer<T> {}
impl<T> HeapFree for RealtimeBuffer<T> {} // No allocations allowed after construction
impl<T> NonBlocking for RealtimeBuffer<T> {}

impl<T> Deref for RealtimeBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len]
    }
}

impl<T> DerefMut for RealtimeBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data[..self.len]
    }
}

impl<T> Index<usize> for RealtimeBuffer<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[..self.len][index]
    }
}

impl<T> IndexMut<usize> for RealtimeBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[..self.len][index]
    }
}

impl<T: fmt::Debug> fmt::Debug for RealtimeBuffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Realtime Buffer")
            .field("len", &self.len)
            .field("capacity", &self.data.len())
            .field("data", &&self.data[..self.len])
            .finish()
    }
}

// ================
// Specialized Audio Buffer
// ===============

/// A specialized audio buffer with channel-aware access.
#[derive(Clone)]
pub struct AudioBuffer {
    /// Interleaved sample data
    data: RealtimeBuffer<Sample>,
    /// Number of channels
    channels: ChannelCount,
    /// Number of frames
    frames: usize,
}

impl AudioBuffer {
    /// Creates a new audio buffer with the given frame count and channel count.
    #[must_use]
    pub fn new(frames: usize, channels: ChannelCount) -> Self {
        let total_samples = frames * channels.count_usize();
        Self {
            data: RealtimeBuffer::with_value(total_samples, Sample::SILENCE),
            channels,
            frames,
        }
    }

    /// Returns the number of frames
    #[must_use]
    pub const fn frames(&self) -> usize {
        self.frames
    }

    /// Returns the channel count
    pub const fn channels(&self) -> ChannelCount {
        self.channels
    }
    /// Returns the total number of smaples
    pub fn sample_count(&self) -> usize {
        self.data.len()
    }

    /// Gets a sample at given frame and channel
    #[must_use]
    pub fn get_sample(&self, frame: usize, channel: usize) -> Option<Sample> {
        if frame < self.frames && channel < self.channels.count_usize() {
            let index = frame * self.channels.count_usize() + channel;
            self.data.get(index).copied()
        } else {
            None
        }
    }

    /// Sets a sample at the given frame and channel
    pub fn set_sample(&mut self, frame: usize, channel: usize, sample: Sample) {
        if frame < self.frames && channel < self.channels.count_usize() {
            let index = frame * self.channels.count_usize() + channel;
            if let Some(s) = self.data.get_mut(index) {
                *s = sample
            }
        }
    }

    /// Returns a mutable slice of samples for a single frame.
    #[must_use]
    pub fn frame_mut(&mut self, frame_index: usize) -> Option<&mut [Sample]> {
        if frame_index < self.frames {
            let start = frame_index * self.channels.count_usize();
            let end = start + self.channels.count_usize();
            Some(&mut self.data.as_full_mut_slice()[start..end])
        } else {
            None
        }
    }

    /// Returns the raw sample buffer
    #[must_use]
    pub fn samples(&self) -> &[Sample] {
        self.data.as_full_slice()
    }

    /// Returns the raw sample buffer mutably.
    #[must_use]
    pub fn samples_mut(&mut self) -> &mut [Sample] {
        self.data.as_full_mut_slice()
    }

    /// Fills the buffer with silence
    pub fn silence(&mut self) {
        self.data.fill(Sample::SILENCE);
    }

    /// Applies gain to all samples
    pub fn apply_gain(&mut self, gain: crate::types::Gain) {
        for sample in self.data.as_full_mut_slice() {
            *sample = sample.apply_gain(gain);
        }
    }
}

impl RealtimeSafe for AudioBuffer {}
impl HeapFree for AudioBuffer {}
impl NonBlocking for AudioBuffer {}

impl fmt::Debug for AudioBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioBuffer")
            .field("frames", &self.frames)
            .field("channels", &self.channels)
            .finish()
    }
}
