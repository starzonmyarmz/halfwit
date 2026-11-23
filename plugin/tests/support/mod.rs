//! Offline test helpers: read input WAVs, and compare renders against
//! stored "golden" WAVs. Run with BLESS=1 to rewrite the golden files after
//! an intended change to the sound, then listen to them before committing.

#![allow(dead_code)]

use std::path::PathBuf;

pub fn path(dir: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join(dir).join(name)
}

/// Reads a mono or interleaved WAV as f32, with its sample rate.
pub fn read_wav(path: &PathBuf) -> (Vec<f32>, u32) {
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

/// Writes mono or interleaved f32 samples as a 32-bit float WAV.
pub fn write_wav(path: &PathBuf, samples: &[f32], channels: u16, sample_rate: u32) {
    let spec = hound::WavSpec { channels, sample_rate, bits_per_sample: 32, sample_format: hound::SampleFormat::Float };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for &s in samples {
        writer.write_sample(s).unwrap();
    }
    writer.finalize().unwrap();
}

/// Compares `output` with tests/golden/<name>.wav, or rewrites it when BLESS
/// is set.
pub fn check_golden(name: &str, output: &[f32], sample_rate: u32) {
    compare_golden(name, output, 1, sample_rate);
}

/// [`check_golden`] for a stereo render, stored as a stereo WAV.
pub fn check_golden_stereo(name: &str, left: &[f32], right: &[f32], sample_rate: u32) {
    let interleaved: Vec<f32> = left.iter().zip(right).flat_map(|(&l, &r)| [l, r]).collect();
    compare_golden(name, &interleaved, 2, sample_rate);
}

fn compare_golden(name: &str, output: &[f32], channels: u16, sample_rate: u32) {
    let path = path("golden", &format!("{name}.wav"));
    if std::env::var_os("BLESS").is_some() {
        write_wav(&path, output, channels, sample_rate);
        return;
    }
    let (expected, _) = read_wav(&path);
    assert_eq!(expected.len(), output.len(), "{name}: length changed");
    let (at, diff) = output
        .iter()
        .zip(&expected)
        .map(|(a, b)| (a - b).abs())
        .enumerate()
        .fold((0, 0.0f32), |worst, (i, d)| if d > worst.1 { (i, d) } else { worst });
    assert!(diff < 1e-4, "{name}: differs from golden by {diff} at sample {at}; rerun with BLESS=1 if intended");
}

/// A test tone with a few harmonics, roughly a plucked string's spectrum.
pub fn tone(freq: f64, seconds: f64, sample_rate: f32) -> Vec<f32> {
    use std::f64::consts::PI;
    let sr = sample_rate as f64;
    (0..(seconds * sr) as usize)
        .map(|i| {
            let t = i as f64 / sr;
            let x =
                (2.0 * PI * freq * t).sin() + 0.5 * (4.0 * PI * freq * t).sin() + 0.25 * (6.0 * PI * freq * t).sin();
            (0.3 * x) as f32
        })
        .collect()
}
