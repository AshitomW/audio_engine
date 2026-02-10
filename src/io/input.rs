//! Input source defintions

use std::fmt;
use std::path::PathBuf;

use crate::types::{AudioFormat, DeviceId, NetworkProtocol, StreamUrl};

/// Audio input source
///
/// This enum represents all supported input sources with their
/// configuration parameters
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InputSource {
    /// Live audio capture from a device
    Device(DeviceInputConfig),
    /// Audio File Playback
    File(FileInput),
    /// Network Stream input
    Network(NetworkInput),
    /// Generated signal (!! FOR TESTING PURPOSES !!)
    Signal(SignalGenerator),
}

impl InputSource {
    /// Creates a device ipnut with default settings
    #[must_use]
    pub fn device(device_id: DeviceId) -> Self {
        Self::Device(DeviceInputConfig::new(device_id))
    }
    /// Creates a device input using hte default input device
    #[must_use]
    pub fn default_device() -> Self {
        Self::Device(DeviceInputConfig::default())
    }
    /// Creates a file input.
    #[must_use]
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(FileInput::new(path))
    }
    /// Creates a network input
    #[must_use]
    pub fn network(url: StreamUrl) -> Self {
        Self::Network(NetworkInput::new(url))
    }
    /// Creates a silence generator
    #[must_use]
    pub fn silence() -> Self {
        Self::Signal(SignalGenerator::Silence)
    }

    /// Creates a sine wave generator
    #[must_use]
    pub fn sine(frequency_hz: f32) -> Self {
        Self::Signal(SignalGenerator::Sine { frequency_hz })
    }

    /// Returns a description of the input source
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::Device(config) => format!("Device: {}", config.device_id),
            Self::File(file) => format!("File :{}", file.path.display()),
            Self::Network(net) => format!("Network: {}", net.url),
            Self::Signal(sig) => format!("Signal: {sig}"),
        }
    }
}

impl fmt::Display for InputSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Configuration for device input
#[derive(Debug, Clone)]
pub struct DeviceInputConfig {
    /// Device identifier
    pub device_id: DeviceId,
    /// Desired output format
    pub format: Option<AudioFormat>,
}

impl DeviceInputConfig {
    /// Creates a new device input configuration.
    #[must_use]
    pub fn new(device_id: DeviceId) -> Self {
        Self {
            device_id,
            format: None,
        }
    }
    /// Sets the desired format.
    #[must_use]
    pub fn with_format(mut self, format: AudioFormat) -> Self {
        self.format = Some(format);
        self
    }
}

impl Default for DeviceInputConfig {
    fn default() -> Self {
        Self::new(DeviceId::default_input())
    }
}

/// Audio file input configuration
#[derive(Debug, Clone)]
pub struct FileInput {
    /// Path to the audio file
    pub path: PathBuf,
    /// Whether to loop playback
    pub looping: bool,
    /// Starting positions in seconds
    pub start_position: f64,
}

impl FileInput {
    /// Creates a new file input.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            looping: false,
            start_position: 0.0,
        }
    }

    /// Enables looping
    #[must_use]
    pub const fn with_loop(mut self) -> Self {
        self.looping = true;
        self
    }

    /// Sets the start position
    #[must_use]
    pub const fn with_start_position(mut self, seconds: f64) -> Self {
        self.start_position = seconds;
        self
    }

    /// Returns the file extension
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }

    /// Returnt he file format on basis of extension
    pub fn format(&self) -> Option<AudioFileFormat> {
        self.extension().and_then(AudioFileFormat::from_extension)
    }
}

/// Supported audio file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioFileFormat {
    /// Waveform Audio file format
    Wav,
    /// MPEG Audio Layer 3
    Mp3,
    /// Free lossless audio codec
    Flac,
    /// Ogg vorbis
    Ogg,
}

impl AudioFileFormat {
    /// Retrun the format for a given file extension
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wav" | "wave" => Some(Self::Wav),
            "mp3" => Some(Self::Mp3),
            "flac" => Some(Self::Flac),
            "ogg" | "oga" => Some(Self::Ogg),
            _ => None,
        }
    }

    /// Returns the MIME type for the format
    #[must_use]
    pub const fn mime_type(self) -> &'static str {
        match self {
            Self::Wav => "audio/wav",
            Self::Mp3 => "audio/mpeg",
            Self::Flac => "audio/flac",
            Self::Ogg => "audio/ogg",
        }
    }

    /// Returns the default file extension for this format
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
        }
    }
}

impl fmt::Display for AudioFileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wav => write!(f, "WAV"),
            Self::Mp3 => write!(f, "MP3"),
            Self::Flac => write!(f, "FLAC"),
            Self::Ogg => write!(f, "OGG"),
        }
    }
}

/// Network stream input configuration
#[derive(Debug, Clone)]
pub struct NetworkInput {
    /// Stream url
    pub url: StreamUrl,
    /// Buffer size in milliseconds
    pub buffer_ms: u32,
    /// Reconnect on failure
    pub auto_reconnect: bool,
}
impl NetworkInput {
    /// Creates a new network input
    #[must_use]
    pub fn new(url: StreamUrl) -> Self {
        Self {
            url,
            buffer_ms: 1000,
            auto_reconnect: true,
        }
    }

    /// Sets the buffer size
    #[must_use]
    pub const fn with_buffer_ms(mut self, ms: u32) -> Self {
        self.buffer_ms = ms;
        self
    }
    /// Disables auto reconnect
    #[must_use]
    pub const fn without_reconnect(mut self) -> Self {
        self.auto_reconnect = false;
        self
    }
    /// Returns the protocol
    #[must_use]
    pub fn protocol(&self) -> NetworkProtocol {
        self.url.protocol()
    }
}

/// Signal generator that is used for testing
#[derive(Debug, Clone, Copy)]
pub enum SignalGenerator {
    /// Generates Silence
    Silence,
    /// Generates a sine wave at the given frequency.
    Sine {
        /// Frequency in Hz
        frequency_hz: f32,
    },
    /// Generates white noise
    WhiteNoise,
    /// Generates a square wave
    Square {
        /// Frequency in hz
        frequency_hz: f32,
    },
}

impl fmt::Display for SignalGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Silence => write!(f, "Silence"),
            Self::Sine { frequency_hz } => write!(f, "Sine {frequency_hz}Hz"),
            Self::WhiteNoise => write!(f, "White Noise"),
            Self::Square { frequency_hz } => write!(f, "Square {frequency_hz}Hz"),
        }
    }
}
