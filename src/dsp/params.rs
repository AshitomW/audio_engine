use std::fmt;

use crate::types::{Decibels, Gain};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParamId(u32);

impl ParamId {
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl From<u32> for ParamId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl fmt::Display for ParamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Param#{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Decibels(Decibels),
    Gain(Gain),
}

impl ParamValue {
    #[must_use]
    pub fn as_float(&self) -> f32 {
        match self {
            Self::Float(v) => *v,
            Self::Int(v) => *v as f32,
            Self::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Decibels(v) => v.value(),
            Self::Gain(v) => v.as_linear(),
        }
    }

    #[must_use]
    pub fn as_int(&self) -> i32 {
        match self {
            Self::Float(v) => *v as i32,
            Self::Int(v) => *v,
            Self::Bool(v) => i32::from(*v),
            Self::Decibels(v) => v.value() as i32,
            Self::Gain(v) => v.as_linear() as i32,
        }
    }

    #[must_use]
    pub fn as_bool(&self) -> bool {
        match self {
            Self::Float(v) => *v > 0.5,
            Self::Int(v) => *v != 0,
            Self::Bool(v) => *v,
            Self::Decibels(v) => !v.is_silent(),
            Self::Gain(v) => v.as_linear() > 0.0,
        }
    }
}

impl From<f32> for ParamValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<i32> for ParamValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<bool> for ParamValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<Decibels> for ParamValue {
    fn from(value: Decibels) -> Self {
        Self::Decibels(value)
    }
}

impl From<Gain> for ParamValue {
    fn from(value: Gain) -> Self {
        Self::Gain(value)
    }
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub id: ParamId,
    pub name: String,
    pub short_name: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: String,
    pub precision: u8,
}

impl ParameterInfo {
    #[must_use]
    pub fn new(id: ParamId, name: impl Into<String>) -> Self {
        let name = name.into();
        let short_name = name.clone();
        Self {
            id,
            name,
            short_name,
            min: 0.0,
            max: 1.0,
            default: 0.5,
            unit: String::new(),
            precision: 2,
        }
    }

    #[must_use]
    pub fn with_short_name(mut self, short_name: impl Into<String>) -> Self {
        self.short_name = short_name.into();
        self
    }

    #[must_use]
    pub const fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    #[must_use]
    pub const fn with_default(mut self, default: f32) -> Self {
        self.default = default;
        self
    }

    #[must_use]
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    #[must_use]
    pub const fn with_precision(mut self, precision: u8) -> Self {
        self.precision = precision;
        self
    }

    #[must_use]
    pub fn normalize(&self, value: f32) -> f32 {
        if (self.max - self.min).abs() < f32::EPSILON {
            0.0
        } else {
            ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        }
    }

    #[must_use]
    pub fn denormalize(&self, normalized: f32) -> f32 {
        self.min + normalized.clamp(0.0, 1.0) * (self.max - self.min)
    }

    #[must_use]
    pub fn format_value(&self, value: f32) -> String {
        if self.unit.is_empty() {
            format!("{:.prec$}", value, prec = self.precision as usize)
        } else {
            format!(
                "{:.prec$}{}",
                value,
                self.unit,
                prec = self.precision as usize
            )
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SmoothParam {
    current: f32,
    target: f32,
    increment: f32,
    samples_remaining: u32,
}

impl SmoothParam {
    #[must_use]
    pub const fn new(initial: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            increment: 0.0,
            samples_remaining: 0,
        }
    }

    pub fn set_target(&mut self, target: f32, samples: u32) {
        self.target = target;
        if samples == 0 {
            self.current = target;
            self.increment = 0.0;
            self.samples_remaining = 0;
        } else {
            self.increment = (target - self.current) / samples as f32;
            self.samples_remaining = samples;
        }
    }

    pub fn set_immediate(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.increment = 0.0;
        self.samples_remaining = 0;
    }

    #[must_use]
    pub const fn current(&self) -> f32 {
        self.current
    }

    #[must_use]
    pub const fn target(&self) -> f32 {
        self.target
    }

    #[must_use]
    pub const fn is_smoothing(&self) -> bool {
        self.samples_remaining > 0
    }

    #[must_use]
    pub fn next(&mut self) -> f32 {
        if self.samples_remaining > 0 {
            self.current += self.increment;
            self.samples_remaining -= 1;
            if self.samples_remaining == 0 {
                self.current = self.target;
            }
        }
        self.current
    }

    pub fn advance(&mut self, samples: u32) {
        if self.samples_remaining > 0 {
            let advance = samples.min(self.samples_remaining);
            self.current += self.increment * advance as f32;
            self.samples_remaining -= advance;
            if self.samples_remaining == 0 {
                self.current = self.target;
            }
        }
    }
}
