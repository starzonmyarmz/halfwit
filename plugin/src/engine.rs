use std::f64::consts::PI;

/// The PolyBLEP correction reaches one sample back, so the output trails the
/// input by one sample. Dry is delayed to match.
pub const LATENCY: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Feel {
    Raw,
    Smooth,
}

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// How much of the bass layer is added, 0 to 1.
    pub blend: f64,
    pub tone_hz: f64,
    pub gate_db: f64,
    pub release_ms: f64,
    pub stomp: bool,
    pub feel: Feel,
    /// Bass layer level into the drive, 0 to 1.
    pub level: f64,
    /// 0 to 2, where 0 bypasses the drive.
    pub drive: f64,
    pub dry: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            blend: 0.4,
            tone_hz: 250.0,
            gate_db: -45.0,
            release_ms: 100.0,
            stomp: true,
            feel: Feel::Raw,
            level: 0.7,
            drive: 0.0,
            dry: true,
        }
    }
}

pub struct Engine {
    sample_rate: f64,
    settings: Settings,

    det_a: f64,
    det_hp: f64,
    det_rel: f64,
    dc_r: f64,
    gate_att: f64,
    gate_rel_sm: f64,
    max_period: f64,
    lpf_a: f64,
    gate_thresh: f64,
    gate_rel: f64,

    det1: f64,
    det2: f64,
    det_x1: f64,
    det_y: f64,
    det_env: f64,
    armed: bool,
    since_rising: f64,
    period_sm: f64,
    div: bool,
    phase: f64,
    lp1: f64,
    lp2: f64,
    gate_env: f64,
    gate_sm: f64,
    dc_x1: f64,
    dc_y1: f64,

    /// The previous sample, held back for the next edge's correction.
    held_bass: f64,
    held_gain: f64,
    held_dry: [f64; 2],
}

impl Engine {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let mut engine = Self {
            sample_rate: sr,
            settings: Settings::default(),
            det_a: (-2.0 * PI * 700.0 / sr).exp(),
            det_hp: (-2.0 * PI * 30.0 / sr).exp(),
            det_rel: (-1.0 / (sr * 0.02)).exp(),
            dc_r: (-2.0 * PI * 10.0 / sr).exp(),
            gate_att: 1.0 - (-1.0 / (sr * 0.002)).exp(),
            gate_rel_sm: 1.0 - (-1.0 / (sr * 0.01)).exp(),
            max_period: (sr / 20.0).floor(),
            lpf_a: 0.0,
            gate_thresh: 0.0,
            gate_rel: 0.0,
            det1: 0.0,
            det2: 0.0,
            det_x1: 0.0,
            det_y: 0.0,
            det_env: 0.0,
            armed: false,
            since_rising: 0.0,
            period_sm: 0.0,
            div: false,
            phase: 0.0,
            lp1: 0.0,
            lp2: 0.0,
            gate_env: 0.0,
            gate_sm: 0.0,
            dc_x1: 0.0,
            dc_y1: 0.0,
            held_bass: 0.0,
            held_gain: 0.0,
            held_dry: [0.0; 2],
        };
        engine.reset();
        engine.set(Settings::default());
        engine
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate as f32
    }

    pub fn reset(&mut self) {
        self.det1 = 0.0;
        self.det2 = 0.0;
        self.det_x1 = 0.0;
        self.det_y = 0.0;
        self.det_env = 0.0;
        self.armed = false;
        self.since_rising = 0.0;
        self.period_sm = self.sample_rate / 110.0;
        self.div = false;
        self.phase = 0.0;
        self.lp1 = 0.0;
        self.lp2 = 0.0;
        self.gate_env = 0.0;
        self.gate_sm = 0.0;
        self.dc_x1 = 0.0;
        self.dc_y1 = 0.0;
        self.held_bass = 0.0;
        self.held_gain = 0.0;
        self.held_dry = [0.0; 2];
    }

    pub fn set(&mut self, settings: Settings) {
        let sr = self.sample_rate;
        // Keeps the low-pass below Nyquist at low sample rates.
        let tone = settings.tone_hz.min(sr * 0.45);
        self.lpf_a = (-2.0 * PI * tone / sr).exp();
        self.gate_thresh = 10f64.powf(settings.gate_db / 20.0);
        self.gate_rel = (-1000.0 / (settings.release_ms.max(1.0) * sr)).exp();
        self.settings = settings;
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let (out_l, out_r) = self.tick(*l as f64, *r as f64);
            *l = out_l as f32;
            *r = out_r as f32;
        }
    }

    fn tick(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let s = self.settings;
        let mono = (in_l + in_r) * 0.5;

        // The detector only sees a band-limited copy, with hysteresis scaled
        // to its envelope, so harmonics and string noise don't add extra
        // zero crossings.
        self.det1 = self.det1 * self.det_a + mono * (1.0 - self.det_a);
        self.det2 = self.det2 * self.det_a + self.det1 * (1.0 - self.det_a);
        let prev_det = self.det_y;
        self.det_y = self.det_hp * (self.det_y + self.det2 - self.det_x1);
        self.det_x1 = self.det2;
        self.det_env = self.det_y.abs().max(self.det_env * self.det_rel);
        let hyst = self.det_env * 0.2;

        let mut rising = false;
        if self.det_y < -hyst {
            self.armed = true;
        }
        if self.armed && self.det_y > hyst {
            rising = true;
            self.armed = false;
        }

        self.since_rising = (self.since_rising + 1.0).min(self.max_period);
        if rising {
            self.period_sm = self.period_sm * 0.7 + self.since_rising.max(8.0) * 0.3;
            self.since_rising = 0.0;
            self.div = !self.div;
            if self.div {
                self.phase = 0.0;
            }
        }

        let bass = match s.feel {
            Feel::Smooth => {
                self.phase += 1.0 / (2.0 * self.period_sm);
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                let x = (2.0 * PI * self.phase).sin() * 2.0;
                x / (1.0 + x.abs()) * 1.5
            }
            Feel::Raw => {
                let square = if self.div { 1.0 } else { -1.0 };
                if rising {
                    // The crossing fell between the previous sample and
                    // this one; spread the step over both (PolyBLEP).
                    let span = self.det_y - prev_det;
                    let at = if span > 0.0 { ((hyst - prev_det) / span).clamp(0.0, 1.0) } else { 1.0 };
                    let step = 2.0 * square;
                    self.held_bass += step * (1.0 - at) * (1.0 - at) * 0.5;
                    square - step * at * at * 0.5
                } else {
                    square
                }
            }
        };

        let am = mono.abs();
        self.gate_env = if am > self.gate_env { am } else { self.gate_env * self.gate_rel };
        let gate = if self.gate_env > self.gate_thresh { 1.0 } else { 0.0 };
        let coef = if gate > self.gate_sm { self.gate_att } else { self.gate_rel_sm };
        self.gate_sm += (gate - self.gate_sm) * coef;
        let gain = self.gate_sm * s.level * if s.stomp { 1.0 } else { 0.0 };

        let out = self.shape(self.held_bass * self.held_gain, s);
        let dry = if s.dry { 1.0 } else { 0.0 };
        let result = (self.held_dry[0] * dry + out * s.blend, self.held_dry[1] * dry + out * s.blend);

        self.held_bass = bass;
        self.held_gain = gain;
        self.held_dry = [in_l, in_r];
        result
    }

    fn shape(&mut self, mut bass: f64, s: Settings) -> f64 {
        let drive = 1.0 + s.drive;
        if drive > 1.0 {
            bass = (bass * drive).tanh() / drive.tanh();
        }
        self.lp1 = bass * (1.0 - self.lpf_a) + self.lp1 * self.lpf_a;
        self.lp2 = self.lp1 * (1.0 - self.lpf_a) + self.lp2 * self.lpf_a;
        let out = self.lp2 - self.dc_x1 + self.dc_r * self.dc_y1;
        self.dc_x1 = self.lp2;
        self.dc_y1 = out;
        out
    }
}
