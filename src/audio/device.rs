use crate::error::{AudioEngineError, Result};
use crate::types::{AudioFormat, DeviceId, DeviceInfo, DeviceType, SampleRate};
use cpal::traits::{DeviceTrait, HostTrait};
use std::fmt;

/// Sample format for audio data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// 16-bit signed integer
    I16,
    /// 32-bit signed integer
    I32,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// Unsigned 8-bit integer
    U8,
}

impl SampleFormat {
    /// Creates a `SampleFormat` from a CPAL format.
    fn from_cpal(format: cpal::SampleFormat) -> Self {
        match format {
            cpal::SampleFormat::I16 => Self::I16,
            cpal::SampleFormat::I32 => Self::I32,
            cpal::SampleFormat::F32 => Self::F32,
            cpal::SampleFormat::F64 => Self::F64,
            cpal::SampleFormat::U8 => Self::U8,
            _ => Self::F32,
        }
    }
}

/// A supported audio configuration
#[derive(Debug, Clone)]
pub struct SupportedConfig {
    /// Number of channels
    pub channels: u32,
    /// Supported sample rates
    pub sample_rates: Vec<SampleRate>,
    /// Sample format
    pub sample_format: SampleFormat,
}

impl SupportedConfig {
    /// Create a supporterd config from a cpal config
    fn from_cpal(config: &cpal::SupportedStreamConfigRange) -> Option<Self> {
        let channels = u32::from(config.channels());
        let sample_format = SampleFormat::from_cpal(config.sample_format());

        let min_rate = config.min_sample_rate().0;
        let max_rate = config.max_sample_rate().0;

        let sample_rates: Vec<SampleRate> = SampleRate::ALL
            .iter()
            .filter(|r| {
                let hz = r.as_hz();
                hz >= min_rate && hz < max_rate
            })
            .copied()
            .collect();

        if sample_rates.is_empty() {
            return None;
        }

        Some(Self {
            channels,
            sample_rates,
            sample_format,
        })
    }
}

/// Represents an audio device
pub struct AudioDevice {
    device: cpal::Device,
    info: DeviceInfo,
    supported_configs: Vec<SupportedConfig>,
}

impl AudioDevice {
    /// Creates an audiodevice from a cpal device
    fn from_cpal(device: cpal::Device, device_type: DeviceType) -> Result<Self> {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());

        let supported_configs: Vec<SupportedConfig> = match device_type {
            DeviceType::Input => device
                .supported_input_configs()
                .map_err(|e| AudioEngineError::DeviceAccess {
                    message: format!("Failed to get input configs: {e}"),
                })?
                .filter_map(|c| SupportedConfig::from_cpal(&c))
                .collect(),
            DeviceType::Output => device
                .supported_output_configs()
                .map_err(|e| AudioEngineError::DeviceAccess {
                    message: format!("Failed to get output configs: {e}"),
                })?
                .filter_map(|c| SupportedConfig::from_cpal(&c))
                .collect(),
        };

        let max_channels = supported_configs
            .iter()
            .map(|c| c.channels)
            .max()
            .unwrap_or(2);

        let supported_sample_rates: Vec<SampleRate> = supported_configs
            .iter()
            .flat_map(|c| c.sample_rates.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let device_id = DeviceId::new(&name, device_type);
        let info = DeviceInfo::new(device_id, &name)
            .with_max_channels(max_channels)
            .with_sample_rates(supported_sample_rates);

        Ok(Self {
            device,
            info,
            supported_configs,
        })
    }

    /// Returns the device name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.info.name
    }

    /// Returns the device info
    #[must_use]
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    /// Returns the device id.
    #[must_use]
    pub fn id(&self) -> &DeviceId {
        &self.info.id
    }

    /// Returns the device type
    #[must_use]
    pub fn device_type(&self) -> DeviceType {
        self.info.id.device_type()
    }

    /// Returns supported configuration
    #[must_use]
    pub fn supported_configs(&self) -> &[SupportedConfig] {
        &self.supported_configs
    }

    /// Check if a specific format is supported
    #[must_use]
    pub fn supports_format(&self, format: &AudioFormat) -> bool {
        self.supported_configs.iter().any(|c| {
            c.channels >= format.channels.count() && c.sample_rates.contains(&format.sample_rate)
        })
    }

    /// Fiinds the best matching configuration for the requested format
    #[must_use]
    pub fn best_config(&self, format: &AudioFormat) -> Option<cpal::StreamConfig> {
        // Exact Match?
        for config in &self.supported_configs {
            if config.channels == format.channels.count()
                && config.sample_rates.contains(&format.sample_rate)
            {
                return Some(cpal::StreamConfig {
                    channels: cpal::ChannelCount::from(u16::try_from(config.channels).unwrap_or(2)),
                    sample_rate: cpal::SampleRate(format.sample_rate.as_hz()),
                    buffer_size: cpal::BufferSize::Default,
                });
            }
        }

        // No perfect matching, find one with enough channels
        for config in &self.supported_configs {
            if config.channels >= format.channels.count()
                && config.sample_rates.contains(&format.sample_rate)
            {
                return Some(cpal::StreamConfig {
                    channels: cpal::ChannelCount::from(
                        u16::try_from(format.channels.count()).unwrap_or(2),
                    ),
                    sample_rate: cpal::SampleRate(format.sample_rate.as_hz()),
                    buffer_size: cpal::BufferSize::Default,
                });
            }
        }
        return None;
    }

    /// Gets the underlying CPAL device
    #[must_use]
    pub fn cpal_device(&self) -> &cpal::Device {
        &self.device
    }
}

impl fmt::Debug for AudioDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioDevice")
            .field("name", &self.name())
            .field("type", &self.device_type())
            .field("configs", &self.supported_configs.len())
            .finish()
    }
}

impl fmt::Display for AudioDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Managaes audio device enumeration and selection.
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
    /// Returns an error if th hose is not available
    pub fn with_host(host_id: cpal::HostId) -> Result<Self> {
        let host = cpal::host_from_id(host_id).map_err(|e| AudioEngineError::DeviceAccess {
            message: format!("Failed to initialize host: {e}"),
        })?;

        Ok(Self { host })
    }

    /// Returns the name of the current hose
    #[must_use]
    pub fn host_name(&self) -> &'static str {
        self.host.id().name()
    }

    /// List all available input devices
    /// Returns an error if device enumeration fails.
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

    /// List all the available output devices
    /// Return error if device enumeration fials
    pub fn output_devices(&self) -> Result<Vec<AudioDevice>> {
        let devices = self
            .host
            .output_devices()
            .map_err(|e| AudioEngineError::DeviceAccess {
                message: format!("Failed to enumerate output devices: {e}"),
            })?;

        Ok(devices
            .filter_map(|d| AudioDevice::from_cpal(d, DeviceType::Output).ok())
            .collect())
    }

    /// Returns the default input device
    /// Retusn an error if no default input device is available.
    pub fn default_input(&self) -> Result<AudioDevice> {
        let device = self
            .host
            .default_input_device()
            .ok_or(AudioEngineError::DeviceNotFound {
                device_name: "default input".to_string(),
            })?;
        AudioDevice::from_cpal(device, DeviceType::Input)
    }

    /// Returns the default output device
    /// Will return an error if no default output device is available
    pub fn default_output(&self) -> Result<AudioDevice> {
        let device = self
            .host
            .default_output_device()
            .ok_or(AudioEngineError::DeviceNotFound {
                device_name: "default output".to_string(),
            })?;
        AudioDevice::from_cpal(device, DeviceType::Output)
    }

    /// Find an input device by name
    /// Returns an error if the device is not found
    pub fn find_input(&self, name: &str) -> Result<AudioDevice> {
        self.input_devices()?
            .into_iter()
            .find(|d| d.name().contains(name))
            .ok_or_else(|| AudioEngineError::DeviceNotFound {
                device_name: name.to_string(),
            })
    }

    /// Find an output device by name.
    ///
    /// Returns an error if the device is not found.
    pub fn find_output(&self, name: &str) -> Result<AudioDevice> {
        self.output_devices()?
            .into_iter()
            .find(|d| d.name().contains(name))
            .ok_or_else(|| AudioEngineError::DeviceNotFound {
                device_name: name.to_string(),
            })
    }

    #[must_use]
    pub fn host(&self) -> &cpal::Host {
        &self.host
    }
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AudioDeviceManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioDeviceManager")
            .field("host", &self.host_name())
            .finish()
    }
}
