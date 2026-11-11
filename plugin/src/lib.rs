use nice_plug::prelude::*;
use std::sync::Arc;

pub struct Halfwit {
    params: Arc<HalfwitParams>,
}

#[derive(Params)]
struct HalfwitParams {
    #[id = "gain"]
    gain: FloatParam,
}

impl Default for Halfwit {
    fn default() -> Self {
        let gain = FloatParam::new("Gain", util::db_to_gain(0.0), FloatRange::gain_range(-30.0, 30.0))
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db());
        Self { params: Arc::new(HalfwitParams { gain }) }
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

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            for sample in samples {
                *sample *= gain;
            }
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for Halfwit {
    const CLAP_ID: &'static str = "com.danmarino.halfwit";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A gain knob");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Halfwit {
    const VST3_CLASS_ID: [u8; 16] = *b"HalfwitDanMarino";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nice_export_clap!(Halfwit);
nice_export_vst3!(Halfwit);
