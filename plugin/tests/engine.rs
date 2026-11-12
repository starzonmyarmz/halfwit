use halfwit::engine::{Engine, Feel, Output, Settings, LATENCY};
use std::f64::consts::PI;
use std::path::PathBuf;

const SR: f32 = 48_000.0;

fn dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join(name)
}

fn read_wav(path: &PathBuf) -> (Vec<f32>, u32) {
    let mut reader = hound::WavReader::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let spec = reader.spec();
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(Result::unwrap).collect(),
        hound::SampleFormat::Int => {
            let scale = (1u32 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| s.unwrap() as f32 / scale).collect()
        }
    };
    (samples, spec.sample_rate)
}

fn write_wav(path: &PathBuf, samples: &[f32], sample_rate: u32) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for &s in samples {
        writer.write_sample(s).unwrap();
    }
    writer.finalize().unwrap();
}

/// Runs mono input through the engine in host-sized blocks and returns the
/// left main output and the bass output.
fn render_both(input: &[f32], sample_rate: f32, settings: Settings) -> (Vec<f32>, Vec<f32>) {
    let mut engine = Engine::new(sample_rate);
    engine.set(settings);
    let mut left = input.to_vec();
    let mut right = input.to_vec();
    let mut bass = vec![0.0; input.len()];
    for ((l, r), b) in left.chunks_mut(512).zip(right.chunks_mut(512)).zip(bass.chunks_mut(512)) {
        engine.process(l, r, b);
    }
    (left, bass)
}

fn render(input: &[f32], sample_rate: f32, settings: Settings) -> Vec<f32> {
    render_both(input, sample_rate, settings).0
}

fn bass_only(feel: Feel) -> Settings {
    Settings { blend: 1.0, output: Output::Bass, feel, ..Settings::default() }
}

fn tone(freq: f64, seconds: f64, sample_rate: f32) -> Vec<f32> {
    let sr = sample_rate as f64;
    (0..(seconds * sr) as usize)
        .map(|i| {
            let t = i as f64 / sr;
            let x = (2.0 * PI * freq * t).sin() + 0.5 * (4.0 * PI * freq * t).sin() + 0.25 * (6.0 * PI * freq * t).sin();
            (0.3 * x) as f32
        })
        .collect()
}

/// Rising zero crossings per second, skipping the first half second.
fn frequency(signal: &[f32], sample_rate: f32) -> f64 {
    let settled = &signal[(sample_rate / 2.0) as usize..];
    let crossings = settled.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count();
    crossings as f64 / (settled.len() as f64 / sample_rate as f64)
}

fn peak(signal: &[f32]) -> f32 {
    signal.iter().fold(0.0, |m, s| m.max(s.abs()))
}

/// Compares against the stored render. Set HALFWIT_BLESS=1 to rewrite it
/// after an intended change, then listen to the new file before committing.
fn check_golden(name: &str, feel: Feel) {
    let (input, sample_rate) = read_wav(&dir("di").join("riff.wav"));
    let output = render(&input, sample_rate as f32, bass_only(feel));
    let path = dir("golden").join(format!("{name}.wav"));
    if std::env::var_os("HALFWIT_BLESS").is_some() {
        write_wav(&path, &output, sample_rate);
        return;
    }
    let (expected, _) = read_wav(&path);
    assert_eq!(expected.len(), output.len(), "{name}: length changed");
    let (at, diff) = output
        .iter()
        .zip(&expected)
        .map(|(a, b)| (a - b).abs())
        .enumerate()
        .fold((0, 0.0f32), |best, (i, d)| if d > best.1 { (i, d) } else { best });
    assert!(diff < 1e-4, "{name}: differs from golden by {diff} at sample {at}; rerun with HALFWIT_BLESS=1 if intended");
}

#[test]
fn riff_raw_matches_golden() {
    check_golden("riff-raw", Feel::Raw);
}

#[test]
fn riff_smooth_matches_golden() {
    check_golden("riff-smooth", Feel::Smooth);
}

#[test]
fn tracks_an_octave_down_across_the_neck() {
    for (note, freq) in [("low E", 82.41), ("A", 110.0), ("open G", 196.0), ("high E", 329.63)] {
        for feel in [Feel::Raw, Feel::Smooth] {
            let out = render(&tone(freq, 2.0, SR), SR, bass_only(feel));
            let got = frequency(&out, SR);
            let want = freq / 2.0;
            assert!((got - want).abs() < want * 0.03, "{note} {feel:?}: expected {want:.1} Hz, got {got:.1} Hz");
        }
    }
}

#[test]
fn gate_silences_the_bass_after_the_note() {
    let mut input = tone(110.0, 1.0, SR);
    input.extend(std::iter::repeat_n(0.0, 2 * SR as usize));
    let out = render(&input, SR, bass_only(Feel::Raw));
    assert!(peak(&out[..SR as usize]) > 0.1, "no bass while playing");
    let tail = &out[out.len() - (SR as usize) / 2..];
    assert!(peak(tail) < 1e-4, "bass still sounding {} after the note", peak(tail));
}

#[test]
fn stomp_off_passes_dry_through_with_the_reported_latency() {
    let input = tone(110.0, 0.5, SR);
    let out = render(&input, SR, Settings { stomp: false, ..Settings::default() });
    let latency = LATENCY as usize;
    assert!(out[..latency].iter().all(|&s| s == 0.0));
    assert_eq!(&out[latency..], &input[..input.len() - latency]);
}

#[test]
fn split_sends_guitar_to_main_and_bass_to_aux() {
    let input = tone(110.0, 1.0, SR);
    let settings = Settings { blend: 1.0, ..Settings::default() };
    assert_eq!(settings.output, Output::Split, "split should be the default");
    let (main, bass) = render_both(&input, SR, settings);
    let latency = LATENCY as usize;
    assert_eq!(&main[latency..], &input[..input.len() - latency]);
    assert_eq!(bass, render(&input, SR, bass_only(Feel::Raw)));
}

#[test]
fn mix_adds_the_bass_to_the_guitar() {
    let input = tone(110.0, 1.0, SR);
    let (main, bass) = render_both(&input, SR, Settings { output: Output::Mix, ..Settings::default() });
    let latency = LATENCY as usize;
    for i in latency..input.len() {
        assert!((main[i] - (input[i - latency] + bass[i])).abs() < 1e-6, "sample {i}");
    }
}

#[test]
fn stays_finite_at_every_sample_rate() {
    for sample_rate in [22_050.0, 44_100.0, 48_000.0, 96_000.0, 192_000.0] {
        let settings = Settings { tone_hz: 2000.0, drive: 2.0, level: 1.0, blend: 1.0, ..Settings::default() };
        for feel in [Feel::Raw, Feel::Smooth] {
            let out = render(&tone(82.41, 1.0, sample_rate), sample_rate, Settings { feel, ..settings });
            assert!(out.iter().all(|s| s.is_finite()), "{sample_rate} Hz {feel:?}: non-finite output");
            assert!(peak(&out) < 4.0, "{sample_rate} Hz {feel:?}: peak {}", peak(&out));
        }
    }
}

/// A pick scrape: 30 ms of differentiated white noise peaking at -10 dBFS.
fn scrape(sample_rate: f32) -> Vec<f32> {
    let mut seed = 1u32;
    let mut white = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        seed as f32 / u32::MAX as f32 * 2.0 - 1.0
    };
    let mut prev = 0.0;
    let mut out: Vec<f32> = (0..(0.03 * sample_rate) as usize)
        .map(|_| {
            let w = white();
            let hf = w - prev;
            prev = w;
            hf
        })
        .collect();
    let scale = 0.3 / peak(&out);
    out.iter_mut().for_each(|s| *s *= scale);
    out
}

#[test]
fn a_pick_scrape_barely_sounds_the_bass() {
    let note = tone(110.0, 0.5, SR);
    let mut input = vec![0.0; SR as usize / 2];
    input.extend(scrape(SR));
    input.extend(vec![0.0; SR as usize / 2]);
    let settings = Settings { dynamics: 0.0, ..bass_only(Feel::Raw) };
    let from_scrape = peak(&render(&input, SR, settings));
    let from_note = peak(&render(&note, SR, settings));
    assert!(from_scrape < from_note * 0.05, "scrape gave {from_scrape}, a note gives {from_note}");
}

#[test]
fn dynamics_makes_the_bass_follow_the_playing_level() {
    let loud: Vec<f32> = tone(110.0, 1.0, SR).iter().map(|s| s * 0.2).collect();
    let quiet: Vec<f32> = loud.iter().map(|s| s * 0.1).collect();
    let ratio = |dynamics| {
        let settings = Settings { dynamics, ..bass_only(Feel::Raw) };
        let settled = SR as usize / 2;
        peak(&render(&quiet, SR, settings)[settled..]) / peak(&render(&loud, SR, settings)[settled..])
    };
    assert!((ratio(0.0) - 1.0).abs() < 0.05, "Dynamics at 0 should keep a constant level, got {}", ratio(0.0));
    assert!(ratio(1.0) < 0.15, "Dynamics at 1 should follow a 20 dB drop, got {}", ratio(1.0));
}
