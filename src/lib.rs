//! Real Time Audio Processing Engine In Rust

#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_sign_loss)]
#![deny(clippy::cast_precision_loss)]
#![deny(clippy::cast_possible_wrap)]
#![allow(clippy::module_name_repetitions)]

pub mod buffer;
pub mod channel;
pub mod error;
pub mod io;
pub mod markers;
pub mod types;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::buffer::{RealtimeBuffer, RingBuffer, RingBufferReader, RingBufferWriter};
    pub use crate::channel::{ControlReceiver, ControlSender, RealtimeReceiver};
    pub use crate::error::{AudioEngineError, Result};
    pub use crate::io::{InputSource, OutputTarget};
    pub use crate::markers::{HeapFree, NonBlocking, RealtimeSafe};
    pub use crate::types::{
        AudioFormat, BitDepth, BufferSize, ChannelCount, ChannelLayout, Decibels, FrameCount, Gain,
        Pan, Sample, SampleRate,
    };
}
