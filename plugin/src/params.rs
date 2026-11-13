use crate::engine::{Feel, Output, Settings};
use nice_plug::prelude::*;

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
pub enum FeelParam {
    Raw,
    Smooth,
}

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
pub enum OutputParam {
    Split,
    Mix,
    Bass,
}

#[derive(Params)]
pub struct HalfwitParams {
    #[id = "blend"]
    pub blend: FloatParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "dynamics"]
    pub dynamics: FloatParam,
    #[id = "tone"]
    pub tone: FloatParam,
    #[id = "gate"]
    pub gate: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
    #[id = "note"]
    pub note: IntParam,
    #[id = "feel"]
    pub feel: EnumParam<FeelParam>,
    #[id = "output"]
    pub output: EnumParam<OutputParam>,
    #[id = "stomp"]
    pub stomp: BoolParam,
}

/// Gate: Note runs from E2 to E5 in semitones, as MIDI notes, and one past
/// the top is Off.
const NOTE_LOW: i32 = 40;
const NOTE_OFF: i32 = 77;
const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

fn note_to_string(v: i32) -> String {
    if v >= NOTE_OFF {
        "Off".into()
    } else {
        format!("{}{}", NOTE_NAMES[v.rem_euclid(12) as usize], v.div_euclid(12) - 1)
    }
}

fn string_to_note(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("off") {
        return Some(NOTE_OFF);
    }
    let split = s.find(|c: char| c == '-' || c.is_ascii_digit())?;
    let (name, octave) = s.split_at(split);
    let pitch = NOTE_NAMES.iter().position(|n| n.eq_ignore_ascii_case(name))? as i32;
    let midi = (octave.parse::<i32>().ok()? + 1) * 12 + pitch;
    Some(midi.clamp(NOTE_LOW, NOTE_OFF))
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
            dynamics: percent("Dynamics", 0.5, 1.0),
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
            note: IntParam::new("Note", NOTE_OFF, IntRange::Linear { min: NOTE_LOW, max: NOTE_OFF })
                .with_value_to_string(std::sync::Arc::new(note_to_string))
                .with_string_to_value(std::sync::Arc::new(string_to_note)),
            feel: EnumParam::new("Feel", FeelParam::Raw),
            output: EnumParam::new("Output", OutputParam::Split),
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
            dynamics: self.dynamics.value() as f64,
            note_max_hz: match self.note.value() {
                v if v >= NOTE_OFF => f64::INFINITY,
                v => 440.0 * 2f64.powf((v - 69) as f64 / 12.0),
            },
            output: match self.output.value() {
                OutputParam::Split => Output::Split,
                OutputParam::Mix => Output::Mix,
                OutputParam::Bass => Output::Bass,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_names_round_trip() {
        assert_eq!(note_to_string(40), "E2");
        assert_eq!(note_to_string(57), "A3");
        assert_eq!(note_to_string(NOTE_OFF), "Off");
        for v in NOTE_LOW..=NOTE_OFF {
            assert_eq!(string_to_note(&note_to_string(v)), Some(v));
        }
        assert_eq!(string_to_note("a#3"), Some(58));
    }
}
