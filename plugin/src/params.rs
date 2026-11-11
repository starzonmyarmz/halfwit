use crate::engine::{Feel, Settings};
use nice_plug::prelude::*;

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
pub enum FeelParam {
    Raw,
    Smooth,
}

#[derive(Params)]
pub struct HalfwitParams {
    #[id = "blend"]
    pub blend: FloatParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "tone"]
    pub tone: FloatParam,
    #[id = "gate"]
    pub gate: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
    #[id = "feel"]
    pub feel: EnumParam<FeelParam>,
    #[id = "dry"]
    pub dry: BoolParam,
    #[id = "stomp"]
    pub stomp: BoolParam,
}

fn percent(name: &str, default: f32, max: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max })
        .with_unit("%")
        .with_value_to_string(formatters::v2s_f32_percentage(0))
        .with_string_to_value(formatters::s2v_f32_percentage())
}

impl Default for HalfwitParams {
    fn default() -> Self {
        Self {
            blend: percent("Blend", 0.4, 1.0),
            level: percent("Level", 0.7, 1.0),
            drive: percent("Drive", 0.0, 2.0),
            tone: FloatParam::new(
                "Tone",
                250.0,
                FloatRange::Skewed { min: 40.0, max: 2000.0, factor: FloatRange::skew_factor(-2.0) },
            )
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            gate: FloatParam::new("Gate", -45.0, FloatRange::Linear { min: -80.0, max: 0.0 })
                .with_step_size(0.1)
                .with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_rounded(1)),
            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Skewed { min: 1.0, max: 500.0, factor: FloatRange::skew_factor(-1.0) },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
            feel: EnumParam::new("Feel", FeelParam::Raw),
            dry: BoolParam::new("Dry", true),
            stomp: BoolParam::new("Stomp", true),
        }
    }
}

impl HalfwitParams {
    pub fn settings(&self) -> Settings {
        Settings {
            blend: self.blend.value() as f64,
            tone_hz: self.tone.value() as f64,
            gate_db: self.gate.value() as f64,
            release_ms: self.release.value() as f64,
            stomp: self.stomp.value(),
            feel: match self.feel.value() {
                FeelParam::Raw => Feel::Raw,
                FeelParam::Smooth => Feel::Smooth,
            },
            level: self.level.value() as f64,
            drive: self.drive.value() as f64,
            dry: self.dry.value(),
        }
    }
}
