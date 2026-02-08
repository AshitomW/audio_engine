//! Audio device types

use std::fmt;

/// Type of audio device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    /// Audio input device (microphone, line-in)
    Input,
    /// Audio output device (speakers, headphones)
    Output,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => write!(f, "input"),
            Self::Output => write!(f, "output"),
        }
    }
}

/// Opaque device identifier
///
///
/// This newtype wraps the device ID to prevent accidental misuse
/// and provides type safety for device related operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId {
    /// Internal identifier (could be system specific)
    id: String,
    /// Device Type
    device_type: DeviceType,
}

impl DeviceId {
    /// Creates a new device ID
    #[must_use]
    pub fn new(id: impl Into<String>, device_type: DeviceType) -> Self {
        Self {
            id: id.into(),
            device_type,
        }
    }

    /// Returns the raw ID string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.id
    }

    /// Returns the device type
    #[must_use]
    pub const fn device_type(&self) -> DeviceType {
        self.device_type
    }

    /// Returns true if this is an input device
    #[must_use]
    pub const fn is_input(&self) -> bool {
        matches!(self.device_type, DeviceType::Input)
    }

    /// Returns true if this is an output device
    #[must_use]
    pub const fn is_output(&self) -> bool {
        matches!(self.device_type, DeviceType::Output)
    }

    /// Creates a default input device ID
    #[must_use]
    pub fn default_input() -> Self {
        Self::new("default", DeviceType::Input)
    }

    /// Creates a default output device ID
    #[must_use]
    pub fn default_output() -> Self {
        Self::new("default", DeviceType::Output)
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.device_type, self.id)
    }
}

/// Information aobut an audio device
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device identifier
    pub id: DeviceId,
    /// Human-readable device name
    pub name: String,
    /// Maximum number of channels supported
    pub max_channels: u32,
    /// Supported sample rates
    pub supported_sample_rates: Vec<crate::types::SampleRate>,
    /// Wether this is the system default device
    pub is_default: bool,
}

impl DeviceInfo {
    /// Creates new device information
    pub fn new(id: DeviceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            max_channels: 2,
            supported_sample_rates: vec![crate::types::SampleRate::default()],
            is_default: false,
        }
    }

    /// Sets the maximum channels
    #[must_use]
    pub const fn with_max_channels(mut self, max_channels: u32) -> Self {
        self.max_channels = max_channels;
        self
    }
    /// Sets supported sample rates
    #[must_use]
    pub fn with_sample_rates(mut self, rates: Vec<crate::types::SampleRate>) -> Self {
        self.supported_sample_rates = rates;
        self
    }

    /// Marks this as the default device
    pub const fn as_default(mut self) -> Self {
        self.is_default = true;
        self
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}, {} ch)",
            self.name,
            self.id.device_type(),
            self.max_channels
        )
    }
}
