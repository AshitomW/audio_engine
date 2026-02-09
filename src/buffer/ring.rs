//! Lock Free Ring buffer for real time audio communication
//!
//!
//! This module provides a single producer single consumer
//! SPSC ring buffer
//! Suitable for passing audio data between real-time and non real time threads

use rtrb::{Consumer, Producer, RingBuffer as RtrbRingBuffer};
use std::fmt;

use crate::error::{AudioEngineError, Result};
use crate::markers::{NonBlocking, RealtimeSafe};

/// Lock free single producer single consumer ring buffer
///
/// This will be a wrapper around the rtrb ringbuffer that provides
/// - safe seperation of producer and conumer
/// - error types consistent with rest of crate
/// - real time safety markers
pub struct RingBuffer<T> {
    capacity: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> RingBuffer<T> {
    /// Creates a new ring buffer with the given capacity.
    ///
    /// Returns a tuple of (writer, reader) for the producer consumer ends
    #[must_use]
    pub fn new(capacity: usize) -> (RingBufferWriter<T>, RingBufferReader<T>) {
        let (producer, consumer) = RtrbRingBuffer::new(capacity);

        (
            RingBufferWriter { inner: producer },
            RingBufferReader { inner: consumer },
        )
    }
}

/// Writer end of a ring buffer i.e producer.
///
/// This end is typically held by the thread producing data.
pub struct RingBufferWriter<T> {
    inner: Producer<T>,
}

impl<T> RingBufferWriter<T> {
    /// Returns the number of slots available for writing.
    #[must_use]
    pub fn slots(&self) -> usize {
        self.inner.slots()
    }

    /// Returns true if the buffer is full.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Attemps to push a single element.
    ///
    /// # Errors
    /// Returns an error if the buffer is full
    pub fn push(&mut self, value: T) -> Result<()> {
        self.inner
            .push(value)
            .map_err(|_| AudioEngineError::RingBufferFull { count: (1) })
    }

    /// Attempts to push multiple elements from a slice
    ///
    /// Returns the number of element actually pushed.
    pub fn push_slice(&mut self, slice: &[T]) -> usize
    where
        T: Copy,
    {
        let mut count = 0;
        for &item in slice {
            if self.inner.push(item).is_ok() {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Pushes all elements, blocking until done.  
    /// Can be used in normal threads for convenience, but not in real-time threads.  
    /// Typical use case: safely pushing a whole slice into a buffer without dropping data.
    pub fn push_all(&mut self, slice: &[T]) -> Result<()>
    where
        T: Copy,
    {
        let mut remaining = slice;
        while !remaining.is_empty() {
            let pushed = self.push_slice(remaining);
            if pushed == 0 {
                // Buffer is full, spin wait (not ideal , but should be safe enough)
                std::hint::spin_loop();
            } else {
                remaining = &remaining[pushed..];
            }
        }
        Ok(())
    }
}

impl<T: Send + 'static> RealtimeSafe for RingBufferWriter<T> {}
impl<T> NonBlocking for RingBufferWriter<T> {}

impl<T> fmt::Debug for RingBufferWriter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RingBufferWriter")
            .field("slots", &self.slots())
            .finish()
    }
}

/// Reader end of a ring buffer (consumer)
///
/// This end is typically held by the thread consuming data.
pub struct RingBufferReader<T> {
    inner: Consumer<T>,
}

impl<T> RingBufferReader<T> {
    /// Returns the number of elements available for reading.
    #[must_use]
    pub fn slots(&self) -> usize {
        self.inner.slots()
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Attempts to pop a single element
    /// Returns an error if the buffer is empty.
    pub fn pop(&mut self) -> Result<T> {
        self.inner
            .pop()
            .map_err(|_| AudioEngineError::RingBufferEmpty { count: 1 })
    }

    /// Attempts to pop multiple elements into a slice.
    ///
    /// Returns the number of elements actually popped.
    pub fn pop_slice(&mut self, slice: &mut [T]) -> usize
    where
        T: Copy,
    {
        let mut count = 0;
        for item in slice.iter_mut() {
            if let Ok(value) = self.inner.pop() {
                *item = value;
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Peeks at the next element without removing it
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.inner.peek().ok()
    }

    /// Discards up to count elements
    /// Returns the number of elements actually discarded.
    pub fn discard(&mut self, count: usize) -> usize {
        let mut discarded = 0;
        for _ in 0..count {
            if self.inner.pop().is_ok() {
                discarded += 1;
            } else {
                break;
            }
        }
        discarded
    }
}

impl<T: Send + 'static> RealtimeSafe for RingBufferReader<T> {}
impl<T> NonBlocking for RingBufferReader<T> {}

impl<T> fmt::Debug for RingBufferReader<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RingBufferReader")
            .field("slots", &self.slots())
            .finish()
    }
}

// some type aliases

/// Ring buffer for audio samples
pub type SampleRingBuffer = RingBuffer<crate::types::Sample>;

/// Ring buffer writer for audio samples.
pub type SampleRingWriter = RingBufferWriter<crate::types::Sample>;

/// Ring buffer reader for audio samples.
pub type SampleRingReader = RingBufferReader<crate::types::Sample>;
