use std::num::NonZeroU32;
/// Audio format and buffer related types
use std::{default, fmt};

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
