use crate::types::{ChannelCount, Sample, SampleRate};
use std::fmt;

use super::params::{ParamId, ParamValue, ParameterInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(u32);

impl EffectId {
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl From<u32> for EffectId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl fmt::Display for EffectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Effect#{}", self.0)
    }
}

pub trait Effect: Send + 'static {
    fn id(&self) -> EffectId;
    fn name(&self) -> &str;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn reset(&mut self);
    fn initialize(&mut self, sample_rate: SampleRate, channels: ChannelCount);
    fn process(&mut self, samples: &mut [Sample], channels: ChannelCount);
    fn parameters(&self) -> &[ParameterInfo];
    fn get_parameter(&self, id: ParamId) -> Option<ParamValue>;
    fn set_parameter(&mut self, id: ParamId, value: ParamValue) -> bool;
    fn latency_samples(&self) -> u32 {
        0
    }
    fn tail_samples(&self) -> u32 {
        0
    }
}

pub trait SmoothableEffect: Effect {
    fn set_parameter_smooth(&mut self, id: ParamId, value: ParamValue, samples: u32);
    fn update_smoothing(&mut self);
}
#[derive(Debug, Clone, Copy)]
pub struct ProcessContext {
    pub sample_rate: SampleRate,
    pub channels: ChannelCount,
    pub frames: usize,
    pub position_samples: u64,
    pub tempo_bpm: Option<f32>,
}

impl ProcessContext {
    #[must_use]
    pub const fn new(sample_rate: SampleRate, channels: ChannelCount, frames: usize) -> Self {
        Self {
            sample_rate,
            channels,
            frames,
            position_samples: 0,
            tempo_bpm: None,
        }
    }
}
