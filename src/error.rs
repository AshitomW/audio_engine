//! Error Types

use crate::types::SampleRate;
use std::path::PathBuf;
use thiserror::Error;

/// Primary Result Type For the Audio Engine
pub type Result<T> = std::result::Result<T, AudioEngineError>;

/// Error Type for all audio engine operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AudioEngineError {
    /// Sample rate conversion or validation failed
    #[error("Invalid sample rate: {value}Hz (expected one of : 44100,48000,96000,192000)")]
    InvalidSampleRate {
        /// The invalid sample rate value
        value: u32,
    },

    /// channel count validation failed
    #[error("Invalid channel count: {value} (expected 1-8)")]
    InvalidChannelCount {
        /// The invalid channel count
        value: u32,
    },

    /// Buffer size validation failed
    #[error("Invalid buffer size: {value} (must be power of 2, range 64-8192)")]
    InvalidBufferSize {
        /// The invalid buffer size
        value: u32,
    },

    /// Numeric conversion failed
    #[error("Numeric conversion failed: {message}")]
    NumericConversion {
        /// Description of conversion failure
        message: String,
    },

    /// Buffer capacity exceeded
    #[error("Buffer overflow: attempted to write {attempted} samples, capacity is {capacity}")]
    BufferOverflow {
        /// Number of samples attempted to write
        attempted: usize,
        /// Available capacity
        capacity: usize,
    },

    /// Buffer underrun (not enough data)
    #[error("Buffer underrun: requested {requested} samples, available {available}")]
    BufferUnderRun {
        /// Number of samples requested
        requested: usize,
        /// Number of available samples
        available: usize,
    },

    /// Ring buffer is full
    #[error("Ring buffer full: Cannot push {count} elements")]
    RingBufferFull {
        /// Number of elements that couldn't be pushed
        count: usize,
    },
    /// Ring buffer is empty
    #[error("Ring buffer empty: cannot pop {count} elements")]
    RingBufferEmpty {
        /// Number of elements that couldn't be popped
        count: usize,
    },

    /// Audio Format Mismatch
    #[error("Audio format mismatch: expected {expected} , got {actual}")]
    FormatMismatch {
        /// Expected format description
        expected: String,
        /// Actual format description
        actual: String,
    },

    /// Sample rate mismatch between components
    #[error("Sample rate mismatch: from_rate={from_rate}, to={to_rate}")]
    SampleRateMismatch {
        /// Source sample rate
        from_rate: SampleRate,
        /// Target sample rate
        to_rate: SampleRate,
    },

    /// Channel coutn mismatch between the components
    #[error("Channel count mismatch: source={source}, target={target}")]
    ChannelCountMismatch {
        /// Source channel count,
        source: ChannelCount,
        /// Target channelCount,
        target: ChannelCount,
    },

    #[error("Audio device not found: {device_name}")]
    DeviceNotFound {
        /// Name of the device that wasn't found
        device_name: String,
    },

    /// Device access error
    #[error("Failed to access audio device : {message}")]
    DeviceAccess {
        /// Error Message
        message: String,
    },

    /// File not found
    #[error("Audio file not found: {path}")]
    FileNotFound {
        /// path to the missing file
        path: PathBuf,
    },

    /// Unsupported audio format
    #[error("Unsupported audio format: {format}")]
    UnsupportedFormat {
        /// The unsupported format
        format: String,
    },

    /// Network URL Parsing error
    #[error("Invalid stream URL: {url} - {reason}")]
    InvalidStreamUrl {
        /// The invalid url
        url: String,
        /// Reason for invalidity
        reason: String,
    },

    /// NetworkConnectionError
    #[error("Network Connection Failed: {message}")]
    NetworkConnection {
        /// Error message
        message: String,
    },

    /// Channel send error (receiver dropped)
    #[error("Channel send failed: receiver disconnected")]
    ChannelSendFailed,

    /// Channel receive error (sender dropped)
    #[error("Channel receive failed: sender disconnected")]
    ChannelRecvFailed,

    /// Configuration Error
    #[error("Configuration Error: {message}")]
    Configuration {
        /// Error message
        message: String,
    },

    /// Pipeline state error
    #[error("Pipeline state error: {message}")]
    PipelineState {
        /// Error Message
        message: String,
    },

    /// I/O Error Wrapper
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl AudioEngineError {
    /// Creates a numeric conversion error with the given message
    #[must_use]
    pub fn numeric_conversion(message: impl Into<String>) -> Self {
        Self::NumericConversion {
            message: message.into(),
        }
    }

    /// Creates a configuration error with the given messsage
    #[must_use]
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Creates a pipeline state error with the given message
    #[must_use]
    pub fn pipeline_state(message: impl Into<String>) -> Self {
        Self::PipelineState {
            message: message.into(),
        }
    }

    /// Returns true if this error is recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::BufferUnderRun { .. }
                | Self::RingBufferEmpty { .. }
                | Self::RingBufferFull { .. }
        )
    }

    /// Returns true if this error indicates a fatal condition
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::DeviceNotFound { .. }
                | Self::DeviceAccess { .. }
                | Self::ChannelSendFailed
                | Self::ChannelRecvFailed
        )
    }
}
