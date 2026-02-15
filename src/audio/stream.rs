use crate::audio::device::AudioDevice;
use crate::buffer::{RingBuffer, RingBufferReader, RingBufferWriter};
use crate::error::{AudioEngineError, Result};
use crate::types::{AudioFormat, ChannelCount, Sample, SampleRate};
use cpal::Stream;
use cpal::traits::{DeviceTrait, StreamTrait};

/// Hanlde to a running audio stream
pub struct StreamHandle {
    stream: Stream,
    format: AudioFormat,
}

impl StreamHandle {
    pub fn play(&self) -> Result<()> {
        self.stream
            .play()
            .map_err(|e| AudioEngineError::DeviceAccess {
                message: format!("Failed to start stream: {e}"),
            })
    }

    pub fn pause(&self) -> Result<()> {
        self.stream
            .pause()
            .map_err(|e| AudioEngineError::DeviceAccess {
                message: format!("Failed to pause stream: {e}"),
            })
    }

    #[must_use]
    pub const fn format(&self) -> AudioFormat {
        self.format
    }
}

/// Input callback
fn input_callback(data: &[f32], writer: &mut RingBufferWriter<Sample>) {
    for &sample in data {
        let _ = writer.push(Sample::new(sample));
    }
}

fn output_callback(data: &mut [f32], reader: &mut RingBufferReader<Sample>) {
    for sample in data.iter_mut() {
        *sample = reader.pop().map_or(0.0, |s| s.value());
    }
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub sample_rate: SampleRate,
    pub channels: ChannelCount,
    pub buffer_frames: usize,
}

impl StreamConfig {
    #[must_use]
    pub const fn new(
        sample_rate: SampleRate,
        channels: ChannelCount,
        buffer_frames: usize,
    ) -> Self {
        Self {
            sample_rate,
            channels,
            buffer_frames,
        }
    }

    #[must_use]
    pub const fn to_audio_format(&self) -> AudioFormat {
        AudioFormat::new(self.sample_rate, self.channels, crate::types::BitDepth::F32)
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            sample_rate: SampleRate::Hz48000,
            channels: ChannelCount::Stereo,
            buffer_frames: 512,
        }
    }
}

//// Audio Output stream
pub struct AudioOutputStream {
    handle: StreamHandle,
    writer: RingBufferWriter<Sample>,
}

impl AudioOutputStream {
    pub fn new(device: &AudioDevice, format: AudioFormat, buffer_frames: usize) -> Result<Self> {
        let config =
            device
                .best_config(&format)
                .ok_or_else(|| AudioEngineError::FormatMismatch {
                    expected: format.to_string(),
                    actual: "No compatible configuration".to_string(),
                })?;

        let buffer_size = buffer_frames * format.channels.count_usize() * 4;

        let (writer, mut reader) = RingBuffer::<Sample>::new(buffer_size);

        let err_callback = |err| {
            log::error!("Output stream error: {err}");
        };

        let stream = device
            .cpal_device()
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    output_callback(data, &mut reader);
                },
                err_callback,
                None,
            )
            .map_err(|e| AudioEngineError::DeviceAccess {
                message: format!("Failed to build output stream: {e}"),
            })?;

        Ok(Self {
            handle: StreamHandle { stream, format },
            writer,
        })
    }

    pub fn start(&self) -> Result<()> {
        self.handle.play()
    }

    pub fn pause(&self) -> Result<()> {
        self.handle.pause()
    }

    #[must_use]
    pub fn writer(&mut self) -> &mut RingBufferWriter<Sample> {
        &mut self.writer
    }

    pub fn write(&mut self, buffer: &[Sample]) -> usize {
        self.writer.push_slice(buffer)
    }

    #[must_use]
    pub fn available(&self) -> usize {
        self.writer.slots()
    }
}

pub struct AudioInputStream {
    handle: StreamHandle,
    reader: RingBufferReader<Sample>,
}

impl AudioInputStream {
    pub fn new(device: &AudioDevice, format: AudioFormat, buffer_frames: usize) -> Result<Self> {
        let config =
            device
                .best_config(&format)
                .ok_or_else(|| AudioEngineError::FormatMismatch {
                    expected: format.to_string(),
                    actual: "no compatible configuration".to_string(),
                })?;

        let buffer_size = buffer_frames * format.channels.count_usize();
        let (mut writer, reader) = RingBuffer::<Sample>::new(buffer_size);

        let err_callback = |err| {
            log::error!("Input stream error: {err}");
        };

        let stream = device
            .cpal_device()
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    input_callback(data, &mut writer);
                },
                err_callback,
                None,
            )
            .map_err(|e| AudioEngineError::DeviceAccess {
                message: format!("Failed to build input stream: {e}"),
            })?;

        Ok(Self {
            handle: StreamHandle { stream, format },
            reader,
        })
    }

    pub fn start(&self) -> Result<()> {
        self.handle.play()
    }

    pub fn pause(&self) -> Result<()> {
        self.handle.pause()
    }

    #[must_use]
    pub const fn format(&self) -> AudioFormat {
        self.handle.format()
    }

    #[must_use]
    pub fn reader(&mut self) -> &mut RingBufferReader<Sample> {
        &mut self.reader
    }

    pub fn read(&mut self, buffer: &mut [Sample]) -> usize {
        self.reader.pop_slice(buffer)
    }

    #[must_use]
    pub fn available(&self) -> usize {
        self.reader.slots()
    }
}
