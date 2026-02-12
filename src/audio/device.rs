use crate::error::{AudioEngineError, Result};
use crate::types::{AudioFormat, ChannelCount, DeviceId, DeviceInfo, DeviceType, SampleRate};
use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait};
///! Audio device management with CPAL
use std::fmt;

/// Manages audio device enumeration and selection.
pub struct AudioDeviceManager {
    host: cpal::Host,
}

impl AudioDeviceManager {
    /// Creates a new device manager using the default host.
    #[must_use]
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// Creates a device manager for a specific host
    ///
    /// Errors, Returns an error if the host is not available
    pub fn with_host(host_id: cpal::HostId) -> Result<Self> {
        let host = cpal::host_from_id(host_id).map_err(|e| AudioEngineError::DeviceAccess {
            message: format!("Failed to initialize host: {e}"),
        })?;
        Ok(Self { host })
    }

    /// Returns the name of the current host
    #[must_use]
    pub fn host_name(&self) -> &'static str {
        self.host.id().name()
    }

    /// Lists all available input devices.
    ///
    /// Will return an error if device enumeration fails
    pub fn input_devices(&self) -> Result<Vec<AudioDevice>> {
        let devices = self
            .host
            .input_devices()
            .map_err(|e| AudioEngineError::DeviceAccess {
                message: format!("Failed to enumerate input devices: {e}"),
            })?;
        Ok(devices
            .filter_map(|d| AudioDevice::from_cpal(d, DeviceType::Input).ok())
            .collect())
    }

    /// Lists all available output devices
    ///
    /// Returns an error if the device enumeration fails
    pub fn output_devices(&self) -> Result<Vec<AudioDevice>> {
        let devices = self
            .host
            .output_devices()
            .map_err(|e| AudioEngineError::DeviceAccess {
                message: format!("Failed to enumerate output device : {e}"),
            })?;

        Ok(devices
            .filter_map(|d| AudioDevice::from_cpal(d, DeviceType::Output).ok())
            .collect())
    }
}

/// Represents an audio device
pub struct AudioDevice {
    device: cpal::Device,
    info: DeviceInfo,
    supported_configs: Vec<SupportedConfig>,
}

impl AudioDevice {
    /// Creates an AudioDevice from a cpal devices
    fn from_cpal(device: cpal::Device, device_type: DeviceType) -> Result<Self> {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
    }
}

/// A supported audio configuration
#[derive(Debug, Clone)]
pub struct SupportedConfig {
    /// Number of channels
    pub channels: u32,
    /// Supported sample rates
    pub sample_rates: Vec<SampleRate>,
    /// Sample Format
    pub sample_format: SampleFormat,
}

impl SupportedConfig{
  /// Creates a supported config from a cpal configuration
  
  fn from_cpal(config: &cpal::SupportedStreamConfigRange) -> Option<Self>{
    let channels = u32::from(config.channels());
    let sample_format = SampleFormat::from

  }
}