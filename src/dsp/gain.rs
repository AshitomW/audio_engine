//! Gain effect

use crate::dsp::params::{ParamId, ParamValue, ParameterInfo, SmoothParam};
use crate::dsp::traits::{Effect, EffectId};
use crate::types::{ChannelCount, Gain, Sample, SampleRate};

pub mod params {
    use crate::dsp::params::ParamId;
    pub const GAIN_DB: ParamId = ParamId::new(0);
}

#[derive(Debug)]
pub struct GainEffect {
    id: EffectId,
    enabled: bool,
    gain: SmoothParam,
    sample_rate: SampleRate,
    param_info: Vec<ParameterInfo>,
}

impl GainEffect {
    #[must_use]
    pub fn new(id: EffectId) -> Self {
        Self::with_gain(id, Gain::UNITY)
    }

    #[must_use]
    pub fn with_gain(id: EffectId, gain: Gain) -> Self {
        let param_info = vec![
            ParameterInfo::new(params::GAIN_DB, "Gain")
                .with_short_name("Gain")
                .with_range(-80.0, 24.0)
                .with_default(0.0)
                .with_unit("dB")
                .with_precision(1),
        ];

        Self {
            id,
            enabled: true,
            gain: SmoothParam::new(gain.as_linear()),
            sample_rate: SampleRate::Hz48000,
            param_info,
        }
    }

    pub fn set_gain_db(&mut self, db: f32) {
        let gain = Gain::from_db(db);
        let samples = self.sample_rate.samples_for_milliseconds(10);
        self.gain.set_target(gain.as_linear(), samples);
    }

    #[must_use]
    pub fn gain_db(&self) -> f32 {
        Gain::new(self.gain.current()).as_db()
    }
}

impl Effect for GainEffect {
    fn id(&self) -> EffectId {
        self.id
    }

    fn name(&self) -> &str {
        "Gain"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }

    fn reset(&mut self) {
        self.gain.set_immediate(self.gain.target());
    }

    fn initialize(&mut self, sample_rate: SampleRate, _channels: ChannelCount) {
        self.sample_rate = sample_rate;
    }

    fn process(&mut self, samples: &mut [Sample], _channels: ChannelCount) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let gain = self.gain.next();
            *sample = Sample::new(sample.value() * gain);
        }
    }

    fn parameters(&self) -> &[ParameterInfo] {
        &self.param_info
    }

    fn get_parameter(&self, id: ParamId) -> Option<ParamValue> {
        match id {
            params::GAIN_DB => Some(ParamValue::Float(self.gain_db())),
            _ => None,
        }
    }

    fn set_parameter(&mut self, id: ParamId, value: ParamValue) -> bool {
        match id {
            params::GAIN_DB => {
                self.set_gain_db(value.as_float());
                true
            }
            _ => false,
        }
    }
}
