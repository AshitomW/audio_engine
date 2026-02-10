//! Output target defintions

use std::fmt;
use std::path::PathBuf;

use crate::types::{AudioFormat, DeviceId, StreamBitrate, StreamUrl};

/// Audio output targets.
///
/// This enum will represent all supported output targets with thier configuration paramets.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum OutputTarget {
    /// Live audio playback to a device
    Device(DeviceOutputConfig),
    /// Audio file recording
    File(FileOutput),
    /// Network Stream output
    Network(NetworkOutput),
    /// Null output (discard the audio)
    Null,
}

impl OutputTarget {
    /// Creates a  device output with default settings.
    #[must_use]
    pub fn device(device_id: DeviceId) -> Self {
        Self::Device(DeviceOutputConfig::new(device_id))
    }

    /// Creates a device output using the default output device
    #[must_use]
    pub fn default_device() -> Self {
        Self::Device(DeviceOutputConfig::default())
    }
    /// Creates a file output.
    #[must_use]
    pub fn file(path: impl Into<PathBuf>, format: OutputFileFormat) -> Self {
        Self::File(FileOutput::new(path, format))
    }

    /// Creates a null output
    #[must_use]
    pub const fn null() -> Self {
        Self::Null
    }

    /// Returns a description of the output target
    pub fn description(&self) -> String {
        match self {
            Self::Device(config) => format!("Device: {}", config.device_id),
            Self::File(file) => format!("File: {}", file.path.display()),
            Self::Network(net) => format!("Network: {}", net.url),
            Self::Null => "Null".to_string(),
        }
    }
}

impl fmt::Display for OutputTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Configuration for device output.
#[derive(Debug, Clone)]
pub struct DeviceOutputConfig {
    /// Device identifier
    pub device_id: DeviceId,
    /// desired audio format (if supported)
    pub format: Option<AudioFormat>,
    /// Exclusive mode if available
    pub exclusive: bool,
}

impl DeviceOutputConfig {
    /// Creates a new device output configuration
    #[must_use]
    pub fn new(device_id: DeviceId) -> Self {
        Self {
            device_id,
            format: None,
            exclusive: false,
        }
    }

    /// Sets the desired format.
    #[must_use]
    pub fn with_format(mut self, format: AudioFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Enables exclusive mode.
    #[must_use]
    pub const fn exclusive(mut self) -> Self {
        self.exclusive = true;
        self
    }
}

impl Default for DeviceOutputConfig {
    fn default() -> Self {
        Self::new(DeviceId::default_output())
    }
}

/// Audio file output configuration
#[derive(Debug, Clone)]
pub struct FileOutput {
    /// Path to the output file
    pub path: PathBuf,
    /// Output format
    pub format: OutputFileFormat,
    /// Audio format (sample rate, channels, etc)
    pub audio_format: Option<AudioFormat>,
}

impl FileOutput {
    /// Creates a new file output.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, format: OutputFileFormat) -> Self {
        Self {
            path: path.into(),
            format,
            audio_format: None,
        }
    }

    /// Sets the audio format
    #[must_use]
    pub fn with_audio_format(mut self, audio_format: AudioFormat) -> Self {
        self.audio_format = Some(audio_format);
        self
    }

    /// Creates a wave file output
    #[must_use]
    pub fn wav(path: impl Into<PathBuf>) -> Self {
        Self::new(path, OutputFileFormat::Wav)
    }
    /// Creates an MP3 file output.
    #[must_use]
    pub fn mp3(path: impl Into<PathBuf>) -> Self {
        Self::new(path, OutputFileFormat::Mp3(Mp3Settings::default()))
    }
}

/// Supported output file formats.
#[derive(Debug, Clone)]
pub enum OutputFileFormat {
    /// Waveform audio file format
    Wav,
    /// MPEG Audio Layer 3
    Mp3(Mp3Settings),
}

impl OutputFileFormat {
    /// Returns the default file extension for this format
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3(_) => "mp3",
        }
    }
}

impl fmt::Display for OutputFileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wav => write!(f, "WAV"),
            Self::Mp3(settings) => write!(f, "MP3 ({})", settings.bitrate),
        }
    }
}

/// MP3 encoding settings
#[derive(Debug, Clone)]
pub struct Mp3Settings {
    /// Bitrate
    pub bitrate: StreamBitrate,
    /// Quality (0-9, lower => better quality)
    pub quality: u8,
}

impl Default for Mp3Settings {
    fn default() -> Self {
        Self {
            bitrate: StreamBitrate::KBPS_192,
            quality: 2,
        }
    }
}

/// Network Stream output configuration
#[derive(Debug, Clone)]
pub struct NetworkOutput {
    /// Stream url
    pub url: StreamUrl,
    /// Audio Bitrate
    pub audio_bitrate: StreamBitrate,
    /// Buffer size in milliseconds
    pub buffer_ms: u32,
}

impl NetworkOutput {
    /// Creates a new network output.
    #[must_use]
    pub fn new(url: StreamUrl) -> Self {
        Self {
            url,
            audio_bitrate: StreamBitrate::KBPS_192,
            buffer_ms: 1000,
        }
    }

    /// Sets the audio bitrate
    #[must_use]
    pub const fn with_audio_bitrate(mut self, bitrate: StreamBitrate) -> Self {
        self.audio_bitrate = bitrate;
        self
    }

    /// Sets the buffer size
    #[must_use]
    pub const fn with_buffer_ms(mut self, ms: u32) -> Self {
        self.buffer_ms = ms;
        self
    }
}
