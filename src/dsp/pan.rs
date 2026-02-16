//! Pan effect

use crate::dsp::params::{ParamId, ParamValue, ParameterInfo, SmoothParam};
use crate::dsp::traits::{Effect, EffectId};
use crate::types::{ChannelCount, Pan, Sample, SampleRate};

pub mod params {
    use super::ParamId;
    /// Pan position (-1.0 = left, 0.0 = center, 1.0 = right)
    pub const PAN: ParamId = ParamId::new(0);
}

#[derive(Debug)]
pub struct PanEffect {
    id: EffectId,
    enabled: bool,
    pan: SmoothParam,
    sample_rate: SampleRate,
    param_info: Vec<ParameterInfo>,
}

impl PanEffect {
    #[must_use]
    pub fn new(id: EffectId) -> Self {
        Self::with_pan(id, Pan::CENTER)
    }

    #[must_use]
    pub fn with_pan(id: EffectId, pan: Pan) -> Self {
        let param_info = vec![
            ParameterInfo::new(params::PAN, "Pan")
                .with_short_name("Pan")
                .with_range(-1.0, 1.0)
                .with_default(0.0)
                .with_precision(2),
        ];

        Self {
            id,
            enabled: true,
            pan: SmoothParam::new(pan.values()),
            sample_rate: SampleRate::Hz48000,
            param_info,
        }
    }

    pub fn set_pan(&mut self, pan: Pan) {
        let samples = self.sample_rate.samples_for_milliseconds(10);
        self.pan.set_target(pan.values(), samples);
    }

    pub fn pan(&self) -> Pan {
        Pan::new(self.pan.current())
    }
}

impl Effect for PanEffect {
    fn id(&self) -> EffectId {
        self.id
    }

    fn name(&self) -> &str {
        "Pan"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn reset(&mut self) {
        self.pan.set_immediate(self.pan.target());
    }

    fn initialize(&mut self, sample_rate: SampleRate, _: ChannelCount) {
        self.sample_rate = sample_rate;
    }

    fn process(&mut self, samples: &mut [Sample], channels: ChannelCount) {
        if !self.enabled {
            return;
        }

        let channel_count = channels.count_usize();
        for frame in samples.chunks_exact_mut(channel_count) {
            let pan = Pan::new(self.pan.next());
            let (left_gain, right_gain) = pan.gains();

            if let [left, right] = frame {
                *left = Sample::new(left.value() * left_gain.as_linear());
                *right = Sample::new(right.value() * right_gain.as_linear());
            }
        }
    }

    fn parameters(&self) -> &[ParameterInfo] {
        &self.param_info
    }

    fn get_parameter(&self, id: ParamId) -> Option<ParamValue> {
        match id {
            params::PAN => Some(ParamValue::Float(self.pan.current())),
            _ => None,
        }
    }

    fn set_parameter(&mut self, id: ParamId, value: ParamValue) -> bool {
        match id {
            params::PAN => {
                self.set_pan(Pan::new(value.as_float()));
                true
            }
            _ => false,
        }
    }
}
