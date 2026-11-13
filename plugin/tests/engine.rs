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
    harmonics(freq, &[(1.0, 0.0), (0.5, 0.0), (0.25, 0.0)], seconds, sample_rate)
}

/// A note from (amplitude, phase) pairs for the fundamental and its
/// harmonics in order.
fn harmonics(freq: f64, partials: &[(f64, f64)], seconds: f64, sample_rate: f32) -> Vec<f32> {
    let sr = sample_rate as f64;
    (0..(seconds * sr) as usize)
        .map(|i| {
            let t = i as f64 / sr;
            let x: f64 = partials
                .iter()
                .enumerate()
                .map(|(n, (amp, phase))| amp * (2.0 * PI * freq * (n + 1) as f64 * t + phase).sin())
                .sum();
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
fn tracks_low_notes_whose_second_harmonic_outweighs_the_fundamental() {
    let partials = [
        ("2nd harmonic twice the fundamental", [(0.5, 0.0), (1.0, 0.0), (0.6, 0.0)]),
        ("2nd harmonic as loud, out of phase", [(1.0, 0.0), (1.0, 1.5), (0.5, 0.7)]),
        ("2nd harmonic three times the fundamental", [(0.33, 0.0), (1.0, 0.8), (0.7, 0.3)]),
    ];
    for (note, freq) in [("open D in drop D", 73.42), ("low E", 82.41), ("A", 110.0)] {
        for (shape, partials) in partials {
            let out = render(&harmonics(freq, &partials, 2.0, SR), SR, bass_only(Feel::Raw));
            let got = frequency(&out, SR);
            let want = freq / 2.0;
            assert!((got - want).abs() < want * 0.03, "{note}, {shape}: expected {want:.1} Hz, got {got:.1} Hz");
        }
    }
}

#[test]
fn follows_a_jump_up_the_neck_through_hum() {
    let mut input = tone(73.42, 1.0, SR);
    input.extend(tone(659.26, 1.0, SR));
    for (i, s) in input.iter_mut().enumerate() {
        *s += (0.01 * (2.0 * PI * 60.0 * i as f64 / SR as f64).sin()) as f32;
    }
    let out = render(&input, SR, bass_only(Feel::Raw));
    let got = frequency(&out[SR as usize..], SR);
    let want = 659.26 / 2.0;
    assert!((got - want).abs() < want * 0.03, "expected {want:.1} Hz, got {got:.1} Hz");
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

fn note_hz(midi: i32) -> f64 {
    440.0 * 2f64.powf((midi - 69) as f64 / 12.0)
}

fn under_a3(feel: Feel) -> Settings {
    Settings { note_max_hz: note_hz(57), ..bass_only(feel) }
}

#[test]
fn gate_note_silences_notes_above_it() {
    // E2, A2, D3, G3, A3 play; A#3, B3, E4 don't.
    for (midi, plays) in [(40, true), (45, true), (50, true), (55, true), (57, true), (58, false), (59, false), (64, false)] {
        for feel in [Feel::Raw, Feel::Smooth] {
            let out = render(&tone(note_hz(midi), 1.0, SR), SR, under_a3(feel));
            let level = peak(&out[(SR / 2.0) as usize..]);
            if plays {
                assert!(level > 0.1, "MIDI {midi} {feel:?}: bass should play, peak {level}");
            } else {
                assert!(level < 1e-3, "MIDI {midi} {feel:?}: bass should be silent, peak {level}");
            }
        }
    }
}

/// Milliseconds from `from` until the bass first passes `level`.
fn ms_until_above(out: &[f32], from: usize, level: f32) -> f64 {
    let at = out[from..].iter().position(|s| s.abs() > level).expect("bass never came in");
    at as f64 * 1000.0 / SR as f64
}

/// Milliseconds from `from` until the bass last passes `level`, within `to`.
fn ms_until_below(out: &[f32], from: usize, to: usize, level: f32) -> f64 {
    let at = out[from..to].iter().rposition(|s| s.abs() > level).map_or(0, |i| i + 1);
    at as f64 * 1000.0 / SR as f64
}

/// Leaves out the DC blocker settling after a note is cut, which is mostly
/// below 10 Hz.
fn above_30_hz(signal: &[f32]) -> Vec<f32> {
    let a = (-2.0 * PI * 30.0 / SR as f64).exp();
    let (mut x1, mut y1) = (0.0, 0.0);
    signal
        .iter()
        .map(|&x| {
            y1 = a * (y1 + x as f64 - x1);
            x1 = x as f64;
            y1 as f32
        })
        .collect()
}

#[test]
fn gate_note_switches_within_a_few_cycles() {
    // E4 (silent), E2 (plays), E4 again (silent). The gate opens on each
    // attack before the pitch is known, so the high notes blip briefly.
    let half = SR as usize / 2;
    let mut input = tone(note_hz(64), 0.5, SR);
    input.extend(tone(note_hz(40), 0.5, SR));
    input.extend(tone(note_hz(64), 0.5, SR));
    let out = above_30_hz(&render(&input, SR, under_a3(Feel::Raw)));
    let first = ms_until_below(&out, 0, half, 0.1);
    let opens = ms_until_above(&out, half, 0.1);
    let closes = ms_until_below(&out, 2 * half, 3 * half, 0.1);
    eprintln!("first high note {first:.1} ms, low note opens {opens:.1} ms, high note closes {closes:.1} ms");
    assert!(first < 30.0, "bass sounded {first:.1} ms into the first high note");
    assert!(opens < 30.0, "bass took {opens:.1} ms to come in on the low note");
    assert!(closes < 30.0, "bass took {closes:.1} ms to stop on the high note");
    for (from, to) in [(half / 2, half), (2 * half + half / 2, 3 * half)] {
        assert!(peak(&out[from..to]) < 1e-3, "bass still sounding {} under a high note", peak(&out[from..to]));
    }
}

#[test]
fn gate_note_leaves_a_riff_under_it_alone() {
    // The riff stays at G2 and below, so with Gate: Note on A3 nothing should be
    // cut, pick attacks included.
    let (input, sample_rate) = read_wav(&dir("di").join("riff.wav"));
    let sr = sample_rate as f32;
    let off = render(&input, sr, bass_only(Feel::Raw));
    let on = render(&input, sr, Settings { note_max_hz: note_hz(57), ..bass_only(Feel::Raw) });
    let rms = |s: &[f32]| (s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt();
    let window = (sr * 0.05) as usize;
    for (i, (a, b)) in off.chunks(window).zip(on.chunks(window)).enumerate() {
        if rms(a) > 0.02 {
            assert!(rms(b) > rms(a) * 0.8, "cut at {} ms: {} vs {}", i * 50, rms(b), rms(a));
        }
    }
}
