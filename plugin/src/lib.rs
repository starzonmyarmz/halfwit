pub mod engine;
pub mod params;

use engine::{Engine, LATENCY};
use nice_plug::prelude::*;
use params::HalfwitParams;
use std::sync::Arc;

pub struct Halfwit {
    params: Arc<HalfwitParams>,
    engine: Engine,
}

impl Default for Halfwit {
    fn default() -> Self {
        Self { params: Arc::new(HalfwitParams::default()), engine: Engine::new(48_000.0) }
    }
}

impl Plugin for Halfwit {
    const NAME: &'static str = "Halfwit";
    const VENDOR: &'static str = "Dan Marino";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type Editor = ();
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl ActivateContext<Self>,
    ) -> bool {
        context.set_latency_samples(LATENCY);
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
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set(self.params.settings());
        if let [left, right] = buffer.as_slice() {
            self.engine.process(left, right);
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
