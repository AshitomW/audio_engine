/// Audio format and buffer related types
use std::fmt;
use std::num::NonZeroU32;

use crate::error::{AudioEngineError, Result};
use crate::types::SampleRate;

// ============
// Channel count
// ============

/// Number of audio channels
///
/// Supports 1-8 Channels with dedicated variants for commong configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelCount {
    /// Mono (1 Channel)
    Mono,
    /// Stereo (2 channels)
    Stereo,
    /// Quad (4 channels)
    Quad,
    ///5.1 Surround Sound (6 channels)
    Surround51,
    ///7.1 Surround Sound (8 channels)
    Surround71,
}

impl ChannelCount {
    /// Returns the number of channels as  a u32
    #[must_use]
    pub const fn count(self) -> u32 {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Quad => 4,
            Self::Surround51 => 6,
            Self::Surround71 => 8,
        }
    }

    /// Returns the number of channels as a usize
    #[must_use]
    pub const fn count_usize(self) -> usize {
        self.count() as usize
    }

    /// Returns the number of channels a NonZeroU32
    #[must_use]
    pub const fn as_non_zero(self) -> NonZeroU32 {
        match self {
            Self::Mono => match NonZeroU32::new(1) {
                Some(v) => v,
                None => unreachable!(),
            },
            Self::Stereo => match NonZeroU32::new(2) {
                Some(v) => v,
                None => unreachable!(),
            },
            Self::Quad => match NonZeroU32::new(4) {
                Some(v) => v,
                None => unreachable!(),
            },
            Self::Surround51 => match NonZeroU32::new(6) {
                Some(v) => v,
                None => unreachable!(),
            },
            Self::Surround71 => match NonZeroU32::new(8) {
                Some(v) => v,
                None => unreachable!(),
            },
        }
    }

    /// Returns true if this is a stereo compatible format
    #[must_use]
    pub const fn is_stereo_compatible(self) -> bool {
        matches!(
            self,
            Self::Stereo | Self::Quad | Self::Surround51 | Self::Surround71
        )
    }
}

impl TryFrom<u32> for ChannelCount {
    type Error = AudioEngineError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            1 => Ok(Self::Mono),
            2 => Ok(Self::Stereo),
            4 => Ok(Self::Quad),
            6 => Ok(Self::Surround51),
            8 => Ok(Self::Surround71),
            _ => Err(AudioEngineError::InvalidChannelCount { value }),
        }
    }
}

impl Default for ChannelCount {
    fn default() -> Self {
        Self::Stereo
    }
}

impl fmt::Display for ChannelCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mono => write!(f, "Mono"),
            Self::Stereo => write!(f, "Stereo"),
            Self::Quad => write!(f, "Quad"),
            Self::Surround51 => write!(f, "5.1"),
            Self::Surround71 => write!(f, "7.1"),
        }
    }
}

// ==============
// Channel Layout
// =============

/// Describes the spatial layout of the audio channels
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub enum ChannelLayout {
    /// Single Channel , no spatial positions
    Mono,
    /// Standard Stereo (Left, Right)
    #[default]
    Stereo,
    /// Quadraphonic (FL, FR, RL, RR)
    Quad,
    /// 5.1 Surround (FL, FR, C, LFE, RL, RR)
    Surround51,
    /// 7.1 Surround (FL, FR, C, LFE, RL, RR, SL, SR)
    Surround71,
}

impl ChannelLayout {
    /// Returns the coresponding channel count
    #[must_use]
    pub const fn channel_count(self) -> ChannelCount {
        match self {
            Self::Mono => ChannelCount::Mono,
            Self::Stereo => ChannelCount::Stereo,
            Self::Quad => ChannelCount::Quad,
            Self::Surround51 => ChannelCount::Surround51,
            Self::Surround71 => ChannelCount::Surround71,
        }
    }

    /// Returns the channel labels for the layout
    #[must_use]
    pub const fn channel_labels(self) -> &'static [&'static str] {
        match self {
            Self::Mono => &["M"],
            Self::Stereo => &["L", "R"],
            Self::Quad => &["FL", "FR", "RL", "RR"],
            Self::Surround51 => &["FL", "FR", "C", "LFE", "RL", "RR"],
            Self::Surround71 => &["FL", "FR", "C", "LFE", "RL", "RR", "SL", "SR"],
        }
    }
}

impl From<ChannelCount> for ChannelLayout {
    fn from(count: ChannelCount) -> Self {
        match count {
            ChannelCount::Mono => Self::Mono,
            ChannelCount::Stereo => Self::Stereo,
            ChannelCount::Quad => Self::Quad,
            ChannelCount::Surround51 => Self::Surround51,
            ChannelCount::Surround71 => Self::Surround71,
        }
    }
}

// =================
// Buffer Size
// =================

/// Audio buffer size in sample per channel.
///
/// Must be a power of 2 in the range of 62-8192
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferSize(NonZeroU32);
impl BufferSize {
    /// Minimum allowed buffer size
    pub const MIN: u32 = 64;
    /// Maximum allowed buffer size
    pub const MAX: u32 = 8192;
    /// Common buffer sizes
    pub const SIZE_64: Self = Self(match NonZeroU32::new(64) {
        Some(v) => v,
        None => unreachable!(),
    });
    pub const SIZE_128: Self = Self(match NonZeroU32::new(128) {
        Some(v) => v,
        None => unreachable!(),
    });
    pub const SIZE_256: Self = Self(match NonZeroU32::new(256) {
        Some(v) => v,
        None => unreachable!(),
    });
    pub const SIZE_512: Self = Self(match NonZeroU32::new(512) {
        Some(v) => v,
        None => unreachable!(),
    });
    pub const SIZE_1024: Self = Self(match NonZeroU32::new(1024) {
        Some(v) => v,
        None => unreachable!(),
    });
    pub const SIZE_2048: Self = Self(match NonZeroU32::new(2048) {
        Some(v) => v,
        None => unreachable!(),
    });
    pub const SIZE_4096: Self = Self(match NonZeroU32::new(4096) {
        Some(v) => v,
        None => unreachable!(),
    });
    pub const SIZE_8192: Self = Self(match NonZeroU32::new(8192) {
        Some(v) => v,
        None => unreachable!(),
    });

    /// All valid buffer sizes
    pub const ALL: [Self; 8] = [
        Self::SIZE_64,
        Self::SIZE_128,
        Self::SIZE_256,
        Self::SIZE_512,
        Self::SIZE_1024,
        Self::SIZE_2048,
        Self::SIZE_4096,
        Self::SIZE_8192,
    ];

    /// Creates a new buffer size if the value is valid
    ///
    /// # Errors
    /// Returns an error if the value is not a power of 2 or outside the valid range.
    pub fn new(value: u32) -> Result<Self> {
        if !value.is_power_of_two() || value < Self::MIN || value > Self::MAX {
            return Err(AudioEngineError::InvalidBufferSize { value });
        }

        NonZeroU32::new(value)
            .map(Self)
            .ok_or(AudioEngineError::InvalidBufferSize { value })
    }

    /// Returns the buffer size as a `u32`
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0.get()
    }

    /// Returns the buffer size as a `usize`
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0.get() as usize
    }

    /// Returns the buffer size as a `NonZeroU32`
    #[must_use]
    pub const fn as_non_zero(self) -> NonZeroU32 {
        self.0
    }

    /// Calculates latency in milliseconds for a given sample rate
    #[must_use]
    pub fn latency_ms(self, sample_rate: SampleRate) -> f32 {
        let samples = f32::from(u16::try_from(self.0.get()).unwrap_or(u16::MAX));

        let rate = sample_rate.as_hz() as f32;
        return (samples / rate) * 1000.0;
    }

    /// Returns the next larger buffer size, if available
    #[must_use]
    pub fn next_larger(self) -> Option<Self> {
        let next = self.as_u32().saturating_mul(2);
        if next <= Self::MAX {
            Self::new(next).ok()
        } else {
            None
        }
    }

    /// Returns the next smaller buffer size, if available
    #[must_use]
    pub fn next_smaller(self) -> Option<Self> {
        let next = self.as_u32() / 2;
        if next >= Self::MIN {
            Self::new(next).ok()
        } else {
            None
        }
    }
}

impl TryFrom<u32> for BufferSize {
    type Error = AudioEngineError;

    fn try_from(value: u32) -> Result<Self> {
        Self::new(value)
    }
}

impl Default for BufferSize {
    fn default() -> Self {
        Self::SIZE_512
    }
}

impl fmt::Display for BufferSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} samples", self.0.get())
    }
}

// ============================
// Frame Count
// ============================

/// Number of audio frames ( a frame = one sample per channel).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct FrameCount(u64);

impl FrameCount {
    /// Zero frames
    pub const ZERO: Self = Self(0);

    /// Creates a new frame count
    #[must_use]
    pub const fn new(frames: u64) -> Self {
        Self(frames)
    }

    /// Returns the frame count as a `u64`
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the total sample cou9nt for the given channel count
    #[must_use]
    pub fn total_samples(self, channels: ChannelCount) -> u64 {
        self.0.saturating_mul(u64::from(channels.count()))
    }

    /// Adds frames, staturating on overflow
    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Subtracts frames, saturating on underflow
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Calculates duration in seconds for the given sample rrate
    #[must_use]
    pub fn duration_seconds(self, sample_rate: SampleRate) -> f64 {
        self.0 as f64 / f64::from(sample_rate.as_hz())
    }
}

impl From<u64> for FrameCount {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<usize> for FrameCount {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl fmt::Display for FrameCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} frames", self.0)
    }
}

// ==========
// Bit Depth
// ==========

/// Audio bit depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BitDepth {
    /// 16 bit integer
    I16,
    /// 24 bit integer (stored in 32 bit)
    I24,
    /// 32 bit integer
    I32,
    /// 32 bit floating point
    #[default]
    F32,
    /// 64 bit floatig point
    F64,
}

impl BitDepth {
    /// Returns the number of bits
    #[must_use]
    pub const fn bits(self) -> u32 {
        match self {
            Self::I16 => 16,
            Self::I24 => 24,
            Self::I32 => 32,
            Self::F32 => 32,
            Self::F64 => 64,
        }
    }

    /// Retursn the number of bytes per sample
    #[must_use]
    pub const fn bytes_per_sample(self) -> u32 {
        match self {
            Self::I16 => 2,
            Self::I24 => 3,
            Self::I32 | Self::F32 => 4,
            Self::F64 => 8,
        }
    }

    /// Returns true if this is a floating point format
    #[must_use]
    pub const fn is_float(self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }

    /// Returns true if this is an integer format
    #[must_use]
    pub const fn is_integer(self) -> bool {
        !self.is_float()
    }
}

impl fmt::Display for BitDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I16 => write!(f, "16-bit int"),
            Self::I24 => write!(f, "24-bit int"),
            Self::I32 => write!(f, "32-bit int"),
            Self::F32 => write!(f, "32-bit float"),
            Self::F64 => write!(f, "64-bit float"),
        }
    }
}

// ===========
// Audio Format
// ===========

/// Complete Audio Format Specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioFormat {
    /// Sample rate
    pub sample_rate: SampleRate,
    /// Number of channels
    pub channels: ChannelCount,
    /// Bit Depth
    pub bit_depth: BitDepth,
}

impl AudioFormat {
    /// Standard CD quality (44.1kHZ, 16-bit stereo)
    pub const CD_QUALITY: Self = Self {
        sample_rate: SampleRate::Hz44100,
        channels: ChannelCount::Stereo,
        bit_depth: BitDepth::I16,
    };
    /// Standard professional quality (48kHz, 24bit stereo)
    pub const PROFESSIONAL: Self = Self {
        sample_rate: SampleRate::Hz48000,
        channels: ChannelCount::Stereo,
        bit_depth: BitDepth::I24,
    };
    /// High resolution (96khz, 32bit float stereo)
    pub const HIGH_RES: Self = Self {
        sample_rate: SampleRate::Hz96000,
        channels: ChannelCount::Stereo,
        bit_depth: BitDepth::F32,
    };

    /// Creates a new audio format
    #[must_use]
    pub const fn new(sample_rate: SampleRate, channels: ChannelCount, bit_depth: BitDepth) -> Self {
        Self {
            sample_rate,
            channels,
            bit_depth,
        }
    }

    /// Returns the byte rate (bytes per second)
    #[must_use]
    pub fn byte_rate(self) -> u32 {
        self.sample_rate.as_hz() * self.channels.count() * self.bit_depth.bytes_per_sample()
    }

    /// Returns the frame size in bytes
    #[must_use]
    pub fn frame_size(self) -> u32 {
        self.channels.count() * self.bit_depth.bytes_per_sample()
    }

    /// Checks if this format is compatible with another for mixing
    #[must_use]
    pub fn is_compatible_with(self, other: Self) -> bool {
        self.sample_rate == other.sample_rate && self.channels == other.channels
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self {
            sample_rate: SampleRate::default(),
            channels: ChannelCount::default(),
            bit_depth: BitDepth::default(),
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{}",
            self.sample_rate, self.channels, self.bit_depth
        )
    }
}
