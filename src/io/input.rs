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
}
