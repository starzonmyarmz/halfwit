mod bridge;
mod editor;
pub mod engine;
pub mod params;

use engine::{Engine, LATENCY};
use nice_plug::prelude::*;
use params::HalfwitParams;
use std::sync::Arc;

static PAGE: editor::Page = editor::Page {
    title: "Halfwit",
    size: (488, 640),
    html: include_str!("../../ui/index.html"),
    background: (24, 23, 22, 255),
    dev_url_var: "HALFWIT_UI_URL",
};

pub struct Halfwit {
    params: Arc<HalfwitParams>,
    engine: Engine,
    /// Takes the bass layer when the host gives no aux output.
    scratch: Vec<f32>,
}

impl Default for Halfwit {
    fn default() -> Self {
        Self { params: Arc::new(HalfwitParams::default()), engine: Engine::new(48_000.0), scratch: Vec::new() }
    }
}

impl Plugin for Halfwit {
    const NAME: &'static str = "Halfwit";
    const VENDOR: &'static str = "Dan Marino";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_output_ports: &[new_nonzero_u32(2)],
            names: PortNames {
                layout: Some("Guitar + bass"),
                main_input: Some("Guitar"),
                main_output: Some("Guitar"),
                aux_inputs: &[],
                aux_outputs: &["Bass"],
            },
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    type Editor = editor::WebEditor;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        Some(editor::WebEditor::new(&PAGE, self.params.clone()))
    }

    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl ActivateContext<Self>,
    ) -> bool {
        context.set_latency_samples(LATENCY);
        self.scratch.resize(buffer_config.max_buffer_size as usize, 0.0);
        if self.engine.sample_rate() != buffer_config.sample_rate {
            self.engine = Engine::new(buffer_config.sample_rate);
        }
        true
    }

    fn reset(&mut self) {
        self.engine.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set(self.params.settings());
        let [left, right] = buffer.as_slice() else {
            return ProcessStatus::Normal;
        };
        match aux.outputs.first_mut().map(|b| b.as_slice()) {
            Some([bass_l, bass_r]) => {
                self.engine.process(left, right, bass_l);
                bass_r.copy_from_slice(bass_l);
            }
            _ => {
                let bass = &mut self.scratch[..left.len()];
                self.engine.process(left, right, bass);
            }
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for Halfwit {
    const CLAP_ID: &'static str = "com.danmarino.halfwit";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Octave-divider bass layer for guitar");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::AudioEffect, ClapFeature::PitchShifter, ClapFeature::Distortion, ClapFeature::Stereo];
}

impl Vst3Plugin for Halfwit {
    const VST3_CLASS_ID: [u8; 16] = *b"HalfwitDanMarino";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::PitchShift];
}

nice_export_clap!(Halfwit);
nice_export_vst3!(Halfwit);
