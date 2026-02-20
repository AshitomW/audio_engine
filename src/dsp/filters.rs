//! Biquad filter implementation
use std::f32::consts::PI;
use std::iter::Filter;

use crate::dsp::params::{ParamId, ParamValue, ParameterInfo, SmoothParam};
use crate::dsp::traits::{Effect, EffectId};
use crate::types::{ChannelCount, Sample, SampleRate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    Peak,
    LowShelf,
    HighShelf,
}

pub mod params {
    use super::ParamId;
    pub const FREQUENCY: ParamId = ParamId::new(0);
    pub const Q: ParamId = ParamId::new(1);
    pub const GAIN_DB: ParamId = ParamId::new(2);
}

#[derive(Debug, Clone, Copy, Default)]
struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn process(&mut self, input: f32, coeffs: &BiquadCoeffs) -> f32 {
        let output = coeffs.b0 * input + coeffs.b1 * self.x1 + coeffs.b2 * self.x2
            - coeffs.a1 * self.y1
            - coeffs.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

#[derive(Debug)]
pub struct BiquadFilter {
    id: EffectId,
    enabled: bool,
    filter_type: FilterType,
    frequency: SmoothParam,
    q: SmoothParam,
    gain_db: SmoothParam,
    sample_rate: SampleRate,
    coeffs: BiquadCoeffs,
    states: [BiquadState; 8],
    param_info: Vec<ParameterInfo>,
    coeffs_dirty: bool,
}

impl BiquadFilter {
    #[must_use]
    pub fn new(id: EffectId, filter_type: FilterType) -> Self {
        Self::with_params(id, filter_type, 1000.0, 0.707, 0.0)
    }

    #[must_use]
    pub fn with_params(
        id: EffectId,
        filter_type: FilterType,
        frequency: f32,
        q: f32,
        gain_db: f32,
    ) -> Self {
        let param_info = vec![
            ParameterInfo::new(params::FREQUENCY, "Frequency")
                .with_short_name("frequency")
                .with_range(20.0, 20000.0)
                .with_default(1000.0)
                .with_unit("Hz")
                .with_precision(0),
            ParameterInfo::new(params::Q, "Q")
                .with_short_name("Q")
                .with_range(0.1, 20.0)
                .with_default(0.707)
                .with_precision(2),
            ParameterInfo::new(params::GAIN_DB, "Gain")
                .with_short_name("Gain")
                .with_range(-24.0, 24.0)
                .with_default(0.0)
                .with_unit("dB")
                .with_precision(1),
        ];

        let mut filter = Self {
            id,
            enabled: true,
            filter_type,
            frequency: SmoothParam::new(frequency),
            q: SmoothParam::new(q),
            gain_db: SmoothParam::new(gain_db),
            sample_rate: SampleRate::Hz48000,
            coeffs: BiquadCoeffs::default(),
            states: [BiquadState::default(); 8],
            param_info,
            coeffs_dirty: true,
        };
        filter.update_coefficients();
        filter
    }

    #[must_use]
    pub fn notch(id: EffectId, frequency: f32, q: f32) -> Self {
        Self::with_params(id, FilterType::Notch, frequency, q, 0.0)
    }

    #[must_use]
    pub fn low_pass(id: EffectId, frequency: f32, q: f32) -> Self {
        Self::with_params(id, FilterType::LowPass, frequency, q, 0.0)
    }

    #[must_use]
    pub fn high_pass(id: EffectId, frequency: f32, q: f32) -> Self {
        Self::with_params(id, FilterType::HighPass, frequency, q, gain_db)
    }
    #[must_use]
    pub fn bandpass(id: EffectId, frequency: f32, q: f32) -> Self {
        Self::with_params(id, FilterType::BandPass, frequency, q, 0.0)
    }

    pub fn peak(id: EffectId, frequency: f32, q: f32, gain_db: f32) -> Self {
        Self::with_params(id, FilterType::Peak, frequency, q, gain_db)
    }

    pub fn low_shelf(id: EffectId, frequency: f32, gain_db: f32) -> Self {
        Self::with_params(id, FilterType::LowShelf, frequency, 0.707, gain_db)
    }

    pub fn high_shelf(id: EffectId, frequency: f32, gain_db: f32) -> Self {
        Self::with_params(id, FilterType::HighShelf, frequency, 0.707, gain_db)
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        let samples = self.sample_rate.samples_for_milliseconds(10);
        self.frequency
            .set_target(frequency.clamp(20.0, 20000.0), samples);
        self.coeffs_dirty = true;
    }

    pub fn set_q(&mut self, q: f32) {
        let samples = self.sample_rate.samples_for_milliseconds(10);
        self.q.set_target(q.clamp(0.1, 20.0), samples);
        self.coeffs_dirty = true;
    }

    pub fn set_gain_db(&mut self, db: f32) {
        let samples = self.sample_rate.samples_for_milliseconds(10);
        self.gain_db.set_target(db.clamp(-24.0, 24.0), samples);
        self.coeffs_dirty = true;
    }

    pub fn update_coefficients(&mut self) {
        let fs = f32::from(u16::try_from(self.sample_rate.as_hz()).unwrap_or(48000));

        let freq = self.frequency.current().clamp(20.0, fs * 0.49);
        let q = self.q.current();
        let gain = self.gain_db.current();

        let omega = 2.0 * PI * freq / fs;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            FilterType::LowPass => {
                let b1 = 1.0 - cos_omega;
                let b0 = b1 / 2.0;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighPass => {
                let b1 = -(1.0 + cos_omega);
                let b0 = (1.0 + cos_omega) / 2.0;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::BandPass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Peak => {
                let a = 10.0_f32.powf(gain / 40.0);
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha / a;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::LowShelf => {
                let a = 10.0_f32.powf(gain / 40.0);
                let sqrt_a = a.sqrt();
                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha);
                let a0 = (a + 1.0) + (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
                let a2 = (a + 1.0) + (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighShelf => {
                let a = 10.0_f32.powf(gain / 40.0);
                let sqrt_a = a.sqrt();
                let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha);
                let a0 = (a + 1.0) - (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega);
                let a2 = (a + 1.0) - (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        let a0_inv = 1.0 / a0;
        self.coeffs = BiquadCoeffs {
            b0: b0 * a0_inv,
            b1: b1 * a0_inv,
            b2: b2 * a0_inv,
            a1: a1 * a0_inv,
            a2: a2 * a0_inv,
        };

        self.coeffs_dirty = false;
    }
}

impl Effect for BiquadFilter {
    fn id(&self) -> EffectId {
        self.id
    }

    fn name(&self) -> &str {
        match self.filter_type {
            FilterType::LowPass => "Low Pass",
            FilterType::HighPass => "High Pass",
            FilterType::BandPass => "Band Pass",
            FilterType::Notch => "Notch",
            FilterType::Peak => "Peak",
            FilterType::LowShelf => "Low Shelf",
            FilterType::HighShelf => "High Shelf",
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }

    fn reset(&mut self) {
        for state in &mut self.states {
            state.reset();
        }

        self.frequency.set_immediate(self.frequency.target());
        self.q.set_immediate(self.q.target());
        self.gain_db.set_immediate(self.gain_db.target());
        self.update_coefficients();
    }

    fn initialize(&mut self, sample_rate: SampleRate, channels: ChannelCount) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
    }

    fn process(&mut self, samples: &mut [Sample], channels: ChannelCount) {
        if !self.enabled {
            return;
        }
        if self.coeffs_dirty
            || self.frequency.is_smoothing()
            || self.q.is_smoothing()
            || self.gain_db.is_smoothing()
        {
            self.frequency.next();
            self.q.next();
            self.gain_db.next();
            self.update_coefficients();
        }

        let channel_count = channels.count_usize();

        for frame in samples.chunks_exact_mut(channel_count) {
            for (ch, sample) in frame.iter_mut().enumerate() {
                let output = self.states[ch].process(sample.value(), &self.coeffs);
                *sample = Sample::new(output);
            }
        }
    }

    fn parameters(&self) -> &[ParameterInfo] {
        &self.param_info
    }

    fn get_parameter(&self, id: ParamId) -> Option<ParamValue> {
        match id {
            params::FREQUENCY => Some(ParamValue::Float(self.frequency.current())),
            params::Q => Some(ParamValue::Float(self.q.current())),
            params::GAIN_DB => Some(ParamValue::Float(self.gain_db.current())),
            _ => None,
        }
    }

    fn set_parameter(&mut self, id: ParamId, value: ParamValue) -> bool {
        match id {
            params::FREQUENCY => {
                self.set_frequency(value.as_float());
                true
            }
            params::Q => {
                self.set_q(value.as_float());
                true
            }
            params::GAIN_DB => {
                self.set_gain_db(value.as_float());
                true
            }
            _ => false,
        }
    }
}
