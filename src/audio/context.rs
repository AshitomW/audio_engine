use crate::audio::device::{AudioDevice, AudioDeviceManager};
use crate::audio::stream::{AudioInputStream, AudioOutputStream, StreamConfig};
use crate::error::{AudioEngineError, Result};
use crate::types::AudioFormat;

pub struct AudioContext {
    manager: AudioDeviceManager,
    config: StreamConfig,
    input_device: Option<AudioDevice>,
    output_device: Option<AudioDevice>,
}

impl AudioContext {
    pub fn new() -> Result<Self> {
        let manager = AudioDeviceManager::new();
        let input_device = manager.default_input().ok();
        let output_device = manager.default_output().ok();

        Ok(Self {
            manager,
            config: StreamConfig::default(),
            input_device,
            output_device,
        })
    }

    pub fn with_config(config: StreamConfig) -> Result<Self> {
        let manager = AudioDeviceManager::new();
        let input_device = manager.default_input().ok();
        let output_device = manager.default_output().ok();

        Ok(Self {
            manager,
            config,
            input_device,
            output_device,
        })
    }

    #[must_use]
    pub fn manager(&self) -> &AudioDeviceManager {
        &self.manager
    }

    #[must_use]
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: StreamConfig) {
        self.config = config
    }

    pub fn set_input_device(&mut self, device: AudioDevice) {
        self.input_device = Some(device);
    }

    pub fn set_output_device(&mut self, device: AudioDevice) {
        self.output_device = Some(device);
    }

    #[must_use]
    pub fn input_device(&self) -> Option<&AudioDevice> {
        self.input_device.as_ref()
    }

    #[must_use]
    pub fn output_device(&self) -> Option<&AudioDevice> {
        self.output_device.as_ref()
    }

    pub fn create_input_strea(&self) -> Result<AudioInputStream> {
        let device = self
            .input_device()
            .ok_or_else(|| AudioEngineError::DeviceNotFound {
                device_name: "input device not set".to_string(),
            })?;

        AudioInputStream::new(
            device,
            self.config.to_audio_format(),
            self.config.buffer_frames,
        )
    }

    pub fn create_output_stream(&self) -> Result<AudioOutputStream> {
        let device = self
            .output_device()
            .ok_or_else(|| AudioEngineError::DeviceNotFound {
                device_name: "output device not set".to_string(),
            })?;

        AudioOutputStream::new(
            device,
            self.config.to_audio_format(),
            self.config.buffer_frames,
        )
    }

    pub fn list_input_devices(&self) -> Result<Vec<AudioDevice>> {
        self.manager.input_devices()
    }

    pub fn list_output_devices(&self) -> Result<Vec<AudioDevice>> {
        self.manager.output_devices()
    }

    #[must_use]
    pub fn format(&self) -> AudioFormat {
        self.config.to_audio_format()
    }
}

impl std::fmt::Debug for AudioContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioContext")
            .field("config", &self.config)
            .field(
                "input_devices",
                &self.input_device.as_ref().map(|d| d.name()),
            )
            .field(
                "output_device",
                &self.output_device.as_ref().map(|d| d.name()),
            )
            .finish()
    }
}
