//! Marker traits for real time safety
//!
//! Using traits here just as lablels , not for behavior.
//! The goal is to let the compiler from using the wrong types in real-time audio code.
//!

///This trait marks types that are safe to use on a real time audio thread.
///
///
/// It does not contain anyt methods
/// Impementing it means we make sure the following things are true:
///
///
/// - No heap allocations
/// - No blocking
/// - No unpredictable or very long loops
/// - The code runs in a predicatble amount of time
///
///
/// The compiler cannot actually check these rules.
/// So this trait is more about discipline an design.
pub trait RealtimeSafe: Send + 'static {}

/// This trait makrs types that do not allocate on the heap
///
/// If a type implements this, it should be safe to create and use without
/// causing memory allocation
pub trait HeapFree {}

/// This trait markes types that never block
///
/// BLlocking can be bad in audio threads because it can cause glitches or dropouts
pub trait NonBlocking {}

// ======================
// Basic implementation
// ======================

// Primitive types are simple values stored directly in memory
// They do not allocate and do not block , so they are safe

impl RealtimeSafe for i8 {}
impl RealtimeSafe for i16 {}
impl RealtimeSafe for i32 {}
impl RealtimeSafe for i64 {}
impl RealtimeSafe for u8 {}
impl RealtimeSafe for u16 {}
impl RealtimeSafe for u32 {}
impl RealtimeSafe for u64 {}
impl RealtimeSafe for f32 {}
impl RealtimeSafe for f64 {}
impl RealtimeSafe for bool {}
impl RealtimeSafe for () {}

impl HeapFree for i8 {}
impl HeapFree for i16 {}
impl HeapFree for i32 {}
impl HeapFree for i64 {}
impl HeapFree for u8 {}
impl HeapFree for u16 {}
impl HeapFree for u32 {}
impl HeapFree for u64 {}
impl HeapFree for f32 {}
impl HeapFree for f64 {}
impl HeapFree for bool {}
impl HeapFree for () {}

impl NonBlocking for i8 {}
impl NonBlocking for i16 {}
impl NonBlocking for i32 {}
impl NonBlocking for i64 {}
impl NonBlocking for u8 {}
impl NonBlocking for u16 {}
impl NonBlocking for u32 {}
impl NonBlocking for u64 {}
impl NonBlocking for f32 {}
impl NonBlocking for f64 {}
impl NonBlocking for bool {}
impl NonBlocking for () {}

// -------
// Arrays
// ------

// Arrays have a fixed size known at compile time.
// If the elemnt type is safe then the array should also be sfae

impl<T: RealtimeSafe, const N: usize> RealtimeSafe for [T; N] {}
impl<T: HeapFree, const N: usize> HeapFree for [T; N] {}
impl<T: NonBlocking, const N: usize> NonBlocking for [T; N] {}

// Tuples
// Tuples just group values together, as long as each value is safe , the tuple should also be safe.

impl<A: RealtimeSafe, B: RealtimeSafe> RealtimeSafe for (A, B) {}
impl<A: HeapFree, B: HeapFree> HeapFree for (A, B) {}
impl<A: NonBlocking, B: NonBlocking> NonBlocking for (A, B) {}

impl<A: RealtimeSafe, B: RealtimeSafe, C: RealtimeSafe> RealtimeSafe for (A, B, C) {}
impl<A: HeapFree, B: HeapFree, C: HeapFree> HeapFree for (A, B, C) {}
impl<A: NonBlocking, B: NonBlocking, C: NonBlocking> NonBlocking for (A, B, C) {}

// Option and Result are enums and do not allocate by themselves.
// They are safe as long as the types inside them are safe.

impl<T: RealtimeSafe> RealtimeSafe for Option<T> {}
impl<T: HeapFree> HeapFree for Option<T> {}
impl<T: NonBlocking> NonBlocking for Option<T> {}

impl<T: RealtimeSafe, E: RealtimeSafe> RealtimeSafe for Result<T, E> {}
impl<T: HeapFree, E: HeapFree> HeapFree for Result<T, E> {}
impl<T: NonBlocking, E: NonBlocking> NonBlocking for Result<T, E> {}

// Audio / domain-specific types
// These are types defined in this project.
// After checking their implementation, we mark them as safe.

use crate::types::{
    AudioFormat, BitDepth, BufferSize, ChannelCount, ChannelLayout, Decibels, FrameCount, Gain,
    Pan, Sample, SampleRate,
};

// These types are small value types and do not allocate or block.

impl RealtimeSafe for Sample {}
impl RealtimeSafe for SampleRate {}
impl RealtimeSafe for Gain {}
impl RealtimeSafe for Decibels {}
impl RealtimeSafe for Pan {}
impl RealtimeSafe for ChannelCount {}
impl RealtimeSafe for ChannelLayout {}
impl RealtimeSafe for BufferSize {}
impl RealtimeSafe for FrameCount {}
impl RealtimeSafe for BitDepth {}
impl RealtimeSafe for AudioFormat {}

impl HeapFree for Sample {}
impl HeapFree for SampleRate {}
impl HeapFree for Gain {}
impl HeapFree for Decibels {}
impl HeapFree for Pan {}
impl HeapFree for ChannelCount {}
impl HeapFree for ChannelLayout {}
impl HeapFree for BufferSize {}
impl HeapFree for FrameCount {}
impl HeapFree for BitDepth {}
impl HeapFree for AudioFormat {}

impl NonBlocking for Sample {}
impl NonBlocking for SampleRate {}
impl NonBlocking for Gain {}
impl NonBlocking for Decibels {}
impl NonBlocking for Pan {}
impl NonBlocking for ChannelCount {}
impl NonBlocking for ChannelLayout {}
impl NonBlocking for BufferSize {}
impl NonBlocking for FrameCount {}
impl NonBlocking for BitDepth {}
impl NonBlocking for AudioFormat {}

/// This function is used only to make the compiler check trait bounds.
/// It does nothing at runtime.
///
/// If `T` is not real-time safe, this will fail to compile.
#[inline]
pub const fn assert_realtime_safe<T: RealtimeSafe>() {}

/// Compile-time check for heap-free types.
#[inline]
pub const fn assert_heap_free<T: HeapFree>() {}

/// Compile-time check for non-blocking types.
#[inline]
pub const fn assert_non_blocking<T: NonBlocking>() {}
