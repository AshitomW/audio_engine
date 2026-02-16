//! Sample related types including sample rate , gain and pan.

use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use crate::error::{AudioEngineError, Result};

/// Supported Sample rates in Hz.
///
///
/// This enum restricts sample rate to commonly supported values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SampleRate {
    // 44.1 kHZ -> CD Quality
    Hz44100 = 44100,
    /// 48 kHZ -> Professional audio/video standard
    Hz48000 = 48000,
    /// 96 kHZ -> High Resolution audio
    Hz96000 = 96000,
    /// 192 kHZ -> Ultra high resolution audio
    Hz192000 = 192000,
}

impl SampleRate {
    /// All supported sample rates
    pub const ALL: [Self; 4] = [Self::Hz44100, Self::Hz48000, Self::Hz96000, Self::Hz192000];

    /// Retuns the sample rate as u32 value
    #[must_use]
    pub const fn as_hz(self) -> u32 {
        self as u32
    }

    /// Returns the sample rate as a `NonZeroU32`
    #[must_use]
    pub const fn as_non_zero(self) -> NonZeroU32 {
        match self {
            Self::Hz44100 => match NonZeroU32::new(44100) {
                Some(v) => v,
                None => unreachable!(),
            },
            Self::Hz48000 => match NonZeroU32::new(48000) {
                Some(v) => v,
                None => unreachable!(),
            },
            Self::Hz96000 => match NonZeroU32::new(96000) {
                Some(v) => v,
                None => unreachable!(),
            },
            Self::Hz192000 => match NonZeroU32::new(192000) {
                Some(v) => v,
                None => unreachable!(),
            },
        }
    }

    /// Returns the sample period in seconds
    #[must_use]
    pub fn period_seconds(self) -> f64 {
        1.0 / f64::from(self.as_hz())
    }

    /// Returns the sapmle period in nanoseconds
    pub fn period_nanos(self) -> u64 {
        1_000_000_000 / u64::from(self.as_hz())
    }

    /// Calculates the number of samples for a given duration in milliseconds
    pub fn samples_for_milliseconds(self, millis: u32) -> u32 {
        // samples = (rate * millis) / 1000
        // Using u64 as intermediate to prevent possible overflows
        let samples_u64 = u64::from(self.as_hz()) * u64::from(millis) / 1000;
        // Saturate to u32::MAX if overflow (generally unlikely in practice ig)
        samples_u64.min(u64::from(u32::MAX)) as u32
    }
}

impl TryFrom<u32> for SampleRate {
    type Error = AudioEngineError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            44100 => Ok(Self::Hz44100),
            48000 => Ok(Self::Hz48000),
            96000 => Ok(Self::Hz96000),
            19200 => Ok(Self::Hz192000),
            _ => Err(AudioEngineError::InvalidSampleRate { value }),
        }
    }
}

impl FromStr for SampleRate {
    type Err = AudioEngineError;

    fn from_str(s: &str) -> Result<Self> {
        let value: u32 = s
            .trim()
            .parse()
            .map_err(|_| AudioEngineError::InvalidSampleRate { value: 0 })?;
        Self::try_from(value)
    }
}

impl Default for SampleRate {
    fn default() -> Self {
        Self::Hz48000
    }
}

impl fmt::Display for SampleRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Hz", self.as_hz())
    }
}
// ===============
// Sample Type
// ================

/// A single audio sample value
///
/// Internally repr as `f32` in the range [-1.0,1.0]
/// Values outside this range are allowed for headroom but will be clipped on output
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sample(f32);

impl Sample {
    /// Silence : Zero Amplitude
    pub const SILENCE: Self = Self(0.0);

    /// Maximum positive Amplitude
    pub const MAX: Self = Self(1.0);

    /// Maximum negative Amplitude
    pub const MIN: Self = Self(-1.0);

    /// Creates a new sample from an `f32`
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    /// Creates a new sample clamping to [-1.0, 1.0]
    #[must_use]
    pub fn clamped(value: f32) -> Self {
        Self(value.clamp(-1.0, 1.0))
    }

    /// Returns the raw `f32` value
    pub const fn value(self) -> f32 {
        self.0
    }

    /// Returns true if the sample is within the valid range
    #[must_use]
    pub fn is_valid(self) -> bool {
        (-1.0..=1.0).contains(&self.0) && self.0.is_finite()
    }

    /// Returns true if the sapmle is silent
    #[must_use]
    pub fn is_silent(self) -> bool {
        self.0.abs() < 1e-10
    }

    /// Clips the sample to the valid range [-1.0 and 1.0]
    #[must_use]
    pub fn clip(self) -> Self {
        Self(self.0.clamp(-1.0, 1.0))
    }

    /// Applies gain to this sample
    pub fn apply_gain(self, gain: Gain) -> Self {
        Self(self.0 * gain.as_linear())
    }
}

impl From<f32> for Sample {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<Sample> for f32 {
    fn from(sample: Sample) -> Self {
        sample.0
    }
}

/// Conversion to i16
impl From<Sample> for i16 {
    fn from(sample: Sample) -> Self {
        let clamped = sample.clip();
        // scale and round
        let scaled = clamped.0 * 32767.0;
        // use round to nearest , ties to even
        scaled.round() as i16
    }
}

/// Conversion from i32 (24-bit stored in i32)
impl From<i32> for Sample {
    fn from(value: i32) -> Self {
        // Assuming 24 bit audio stored in i32
        // 24 bit range: -8388608 to 8388607
        Self(value as f32 / 8_388_608.0)
    }
}

impl fmt::Display for Sample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}", self.0)
    }
}

// =================
// GAIN
// ================

/// Linear gain multiplier
///
/// Stored as a linear value (not decibels). A value of 1.0 means unit gain.l

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gain(f32);

impl Gain {
    /// Unity gain (no change)
    pub const UNITY: Self = Self(1.0);

    /// Silence (Complete attenuation)
    pub const SILENCE: Self = Self(0.0);

    /// Minimum practical gain (-80db)
    pub const MIN_DB: f32 = -80.0;

    /// Maximum practical gain (+24 db)
    pub const MAX_DB: f32 = 24.0;

    /// Creates a new gain from a linear value
    ///
    /// # Panics
    /// Panics if the value is negative or not finite
    #[must_use]
    pub fn new(linear: f32) -> Self {
        assert!(linear >= 0.0, "Gain cannot be negative");
        assert!(linear.is_finite(), "Gain must be finite");
        Self(linear)
    }

    /// Creates a gain from a linear value, clamping to a valid range
    #[must_use]
    pub fn from_linear_clamped(linear: f32) -> Self {
        if !linear.is_finite() || linear < 0.0 {
            Self::SILENCE
        } else {
            // Clamp to resonable maximum (+24 dB near to 15.85)
            Self(linear.min(15.85))
        }
    }

    /// Creates a gain from decibels
    #[must_use]
    pub fn from_db(db: f32) -> Self {
        if db <= Self::MIN_DB {
            Self::SILENCE
        } else {
            let clamped_db = db.min(Self::MAX_DB);
            Self(10.0_f32.powf(clamped_db / 20.0))
        }
    }

    /// Returns the linear gain value
    #[must_use]
    pub const fn as_linear(self) -> f32 {
        self.0
    }

    /// Returns the gain in decibels
    #[must_use]
    pub fn as_db(self) -> f32 {
        if self.0 <= 0.0 {
            Self::MIN_DB
        } else {
            20.0 * self.0.log10()
        }
    }

    /// Retuns the gain as a Decibels value
    #[must_use]
    pub fn to_decibels(self) -> Decibels {
        Decibels::new(self.as_db())
    }

    /// Interpolates between two gain values
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        Self(self.0 + (other.0 - self.0) * t_clamped)
    }

    /// Interpolates in the db domain (perceptually linear)
    #[must_use]
    pub fn lerp_db(self, other: Self, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        let db_self = self.as_db();
        let db_other = other.as_db();
        let db_result = db_self + (db_other - db_self) * t_clamped;
        Self::from_db(db_result)
    }
}

impl Default for Gain {
    fn default() -> Self {
        Self::UNITY
    }
}

impl fmt::Display for Gain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} dB", self.as_db())
    }
}

// ================
// Decibels
// ===============

/// A value in decibels
///
/// Used for level metering, gain display, and other UI facing values

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Decibels(f32);
impl Decibels {
    /// Silence threshold
    pub const SILENCE: Self = Self(-80.0);
    /// Reference level 0 db
    pub const ZERO: Self = Self(0.0);

    /// Creates a new decibels value
    #[must_use]
    pub fn new(db: f32) -> Self {
        if db.is_finite() {
            Self(db.clamp(-120.0, 24.0))
        } else {
            Self::SILENCE
        }
    }

    /// Creates a decibels value from a linear amplitude
    #[must_use]
    pub fn from_linear(linear: f32) -> Self {
        if linear <= 0.0 || !linear.is_finite() {
            Self::SILENCE
        } else {
            Self::new(20.0 * linear.log10())
        }
    }

    /// Returns the raw db value
    #[must_use]
    pub const fn value(self) -> f32 {
        self.0
    }

    /// Converts to linear amplitude
    pub fn to_linear(self) -> f32 {
        10.0_f32.powf(self.0 / 20.0)
    }

    /// Converts to a `Gain` value
    pub fn to_gain(self) -> Gain {
        Gain::from_db(self.0)
    }

    /// Returns true if this represents silence
    #[must_use]
    pub fn is_silent(self) -> bool {
        self.0 <= -80.0
    }

    /// Returns true if this represents clipping
    #[must_use]
    pub fn is_clipping(self) -> bool {
        self.0 > 0.0
    }
}

impl Default for Decibels {
    fn default() -> Self {
        Self::SILENCE
    }
}

impl fmt::Display for Decibels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_silent() {
            write!(f, "-infinite dB")
        } else {
            write!(f, "{:+.1} dB", self.0)
        }
    }
}

// ==============
// PAN
// =============

/// Stereo pan position
/// Range is [-1.0,1.0] where
/// -1.0 is full left
/// 0.0 is center
/// and 1.0 is full right
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pan(f32);
impl Pan {
    /// Center Position
    pub const CENTER: Self = Self(0.0);
    /// Full Left
    pub const LEFT: Self = Self(-1.0);
    /// Full Right
    pub const RIGHT: Self = Self(1.0);

    /// Creates a new pan value, clamping to a valid range
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(-1.0, 1.0))
    }

    /// Returns the raw pan value
    #[must_use]
    pub const fn values(self) -> f32 {
        self.0
    }

    /// Returns the left channel gain(constant power panning)
    #[must_use]
    pub fn left_gain(self) -> Gain {
        // Constant power panning : L = cos(theta) , R = sin(theta)
        // where theta = (pan + 1) * pi / 4
        let angle = (self.0 + 1.0) * std::f32::consts::FRAC_PI_4;
        Gain::new(angle.cos())
    }

    /// Retusn the right channel gain (constant power panning)
    pub fn right_gain(self) -> Gain {
        let angle = (self.0 + 1.0) * std::f32::consts::FRAC_PI_4;
        Gain::new(angle.sin())
    }

    /// Returns both channel gains as a tuple (left, right)
    #[must_use]
    pub fn gains(self) -> (Gain, Gain) {
        (self.left_gain(), self.right_gain())
    }

    /// Interpolates between two pan positions
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        Self::new(self.0 + (other.0 - self.0) * t_clamped)
    }
}

impl Default for Pan {
    fn default() -> Self {
        Self::CENTER
    }
}

impl fmt::Display for Pan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.abs() < 0.01 {
            write!(f, "C")
        } else if self.0 < 0.0 {
            write!(f, "L{:.0}", self.0.abs() * 100.0)
        } else {
            write!(f, "R{:.0}", self.0 * 100.0)
        }
    }
}
