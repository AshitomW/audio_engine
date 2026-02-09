//! Time related types for audio processing
//!

use crate::types::SampleRate;
use std::fmt;
use std::time::Duration;

/// A Timestamp in the audio timeline, measured in samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Zero timestamp (start)
    pub const ZERO: Self = Self(0);
    /// Creates a new timestamp from a smaple count
    #[must_use]
    pub const fn from_samples(samples: u64) -> Self {
        Self(samples)
    }

    /// Creates a timestamp from a duration at the given sample rate
    #[must_use]
    pub fn from_duration(duration: Duration, smaple_rate: SampleRate) -> Self {
        let samples = duration.as_secs_f64() * f64::from(smaple_rate.as_hz());

        Self(samples as u64)
    }

    /// Returns the timestampp as a sample count
    #[must_use]
    pub const fn as_samples(self) -> u64 {
        self.0
    }

    /// Converts to a duration at the gievn sample rate
    #[must_use]
    pub fn to_duration(self, sample_rate: SampleRate) -> Duration {
        let seconds = self.0 as f64 / f64::from(sample_rate.as_hz());
        Duration::from_secs_f64(seconds)
    }

    /// Subtracts samples from this timestamp
    #[must_use]
    pub const fn sub_samples(self, samples: u64) -> Self {
        Self(self.0.saturating_sub(samples))
    }
    /// Returns the difference between two timestamps in samples
    #[must_use]
    pub const fn diff(self, other: Self) -> u64 {
        if self.0 >= other.0 {
            self.0 - other.0
        } else {
            other.0 - self.0
        }
    }
}

impl From<u64> for Timestamp {
    fn from(samples: u64) -> Self {
        Self(samples)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

/// Transport position with time code formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TransportPosition {
    hours: u8,
    minutes: u8,
    seconds: u8,
    millis: u16,
}

impl TransportPosition {
    /// Zero position
    pub const ZERO: Self = Self {
        hours: 0,
        minutes: 0,
        seconds: 0,
        millis: 0,
    };

    /// Creates a transport position from milliseconds
    #[must_use]
    pub fn from_millis(total_millis: u64) -> Self {
        let millis = { total_millis % 1000 } as u16;
        let total_seconds = total_millis / 1000;
        let seconds = (total_seconds % 60) as u8;
        let total_minutes = total_seconds / 60;
        let minutes = (total_minutes % 60) as u8;
        let hours = (total_minutes / 60).min(255) as u8;

        Self {
            hours,
            minutes,
            seconds,
            millis,
        }
    }

    /// Crreates a transport position from seconds
    #[must_use]
    pub fn from_seconds_f64(seconds: f64) -> Self {
        let total_millis = (seconds * 1000.0) as u64;
        Self::from_millis(total_millis)
    }

    /// Creates a transport position from a timestamp and sample rate
    #[must_use]
    pub fn from_timestamp(timestamp: Timestamp, sample_rate: SampleRate) -> Self {
        let total_seconds = timestamp.as_samples() as f64 / f64::from(sample_rate.as_hz());

        Self::from_seconds_f64(total_seconds)
    }

    #[must_use]
    pub fn total_millis(self) -> u64 {
        u64::from(self.hours) * 3_600_000
            + u64::from(self.minutes) * 60_000
            + u64::from(self.seconds) * 1000
            + u64::from(self.millis)
    }

    #[must_use]
    pub fn total_seconds_f64(self) -> f64 {
        self.total_millis() as f64 / 1000.0
    }
}

impl fmt::Display for TransportPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! {
            f,
            "{:02}:{:02}:{:02}.{:03}",
            self.hours,
            self.minutes,
            self.seconds,
            self.millis
        }
    }
}
