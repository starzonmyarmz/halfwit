use std::f64::consts::PI;

/// The PolyBLEP correction reaches one sample back, so the output trails the
/// input by one sample. Dry is delayed to match.
pub const LATENCY: u32 = 1;

/// Band-limited input level (-18 dBFS) at which Dynamics gives full level.
const DYNAMICS_FULL: f64 = 0.125;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Feel {
    Raw,
    Smooth,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Output {
    /// Guitar on the main output, bass layer on the aux output.
    Split,
    /// Guitar and bass layer together on the main output.
    Mix,
    /// Only the bass layer on the main output.
    Bass,
}

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// Bass layer output level, 0 to 1.
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
    /// 0 to 1: how far the bass layer follows the playing level, from a
    /// constant level at 0 to fully proportional at 1.
    pub dynamics: f64,
    /// The highest note, in Hz, that the bass plays under. Notes above it
    /// close the gate. Infinite turns this off.
    pub note_max_hz: f64,
    pub output: Output,
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
            dynamics: 0.5,
            note_max_hz: f64::INFINITY,
            output: Output::Split,
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
    note_rel_sm: f64,
    dyn_rel: f64,
    dyn_sm_a: f64,
    max_period: f64,
    lpf_a: f64,
    gate_thresh: f64,
    gate_rel: f64,
    /// Note gate thresholds: closes above `note_close`, opens below
    /// `note_open`, a quarter semitone either side of halfway between the
    /// set note and the one above.
    note_close: f64,
    note_open: f64,

    det1: f64,
    det2: f64,
    det_x1: f64,
    det_y: f64,
    det_env: f64,
    det_armed: bool,
    det_since: f64,
    det_period: f64,
    band_hp_a: f64,
    band_lp_a: f64,
    band_hp: [f64; 4],
    band_lp: [f64; 4],
    band_env: f64,
    armed: bool,
    since_rising: f64,
    period_sm: f64,
    div: bool,
    phase: f64,
    lp1: f64,
    lp2: f64,
    gate_env: f64,
    gate_sm: f64,
    dyn_env: f64,
    dyn_sm: f64,
    dc_x1: f64,
    dc_y1: f64,
    note_ok: bool,
    /// det_y's previous cycle length, to tell a steady note from noise.
    det_last: f64,
    /// Votes against `note_ok` in a row, from the band and from det_y.
    note_votes: [u8; 2],

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
            note_rel_sm: 1.0 - (-1.0 / (sr * 0.003)).exp(),
            dyn_rel: (-1.0 / (sr * 0.05)).exp(),
            dyn_sm_a: 1.0 - (-1.0 / (sr * 0.005)).exp(),
            max_period: (sr / 20.0).floor(),
            lpf_a: 0.0,
            gate_thresh: 0.0,
            gate_rel: 0.0,
            note_close: f64::INFINITY,
            note_open: f64::INFINITY,
            det1: 0.0,
            det2: 0.0,
            det_x1: 0.0,
            det_y: 0.0,
            det_env: 0.0,
            det_armed: false,
            det_since: 0.0,
            det_period: 0.0,
            band_hp_a: 0.0,
            band_lp_a: 0.0,
            band_hp: [0.0; 4],
            band_lp: [0.0; 4],
            band_env: 0.0,
            armed: false,
            since_rising: 0.0,
            period_sm: 0.0,
            div: false,
            phase: 0.0,
            lp1: 0.0,
            lp2: 0.0,
            gate_env: 0.0,
            gate_sm: 0.0,
            dyn_env: 0.0,
            dyn_sm: 0.0,
            dc_x1: 0.0,
            dc_y1: 0.0,
            note_ok: true,
            det_last: 0.0,
            note_votes: [0; 2],
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
        self.det_armed = false;
        self.det_since = 0.0;
        self.det_period = self.sample_rate / 110.0;
        self.band_hp = [0.0; 4];
        self.band_lp = [0.0; 4];
        self.band_env = 0.0;
        self.armed = false;
        self.since_rising = 0.0;
        self.period_sm = self.sample_rate / 110.0;
        self.retune_band();
        self.div = false;
        self.phase = 0.0;
        self.lp1 = 0.0;
        self.lp2 = 0.0;
        self.gate_env = 0.0;
        self.gate_sm = 0.0;
        self.dyn_env = 0.0;
        self.dyn_sm = 0.0;
        self.dc_x1 = 0.0;
        self.dc_y1 = 0.0;
        self.note_ok = true;
        self.det_last = 0.0;
        self.note_votes = [0; 2];
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
        let boundary = settings.note_max_hz * 2f64.powf(0.5 / 12.0);
        self.note_close = boundary * 2f64.powf(0.25 / 12.0);
        self.note_open = boundary * 2f64.powf(-0.25 / 12.0);
        if settings.note_max_hz.is_infinite() {
            self.note_ok = true;
            self.note_votes = [0; 2];
        }
        self.settings = settings;
    }

    /// Processes the main buffers in place and writes the bass layer to
    /// `bass`, which must be as long as `left`.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], bass: &mut [f32]) {
        for ((l, r), b) in left.iter_mut().zip(right.iter_mut()).zip(bass.iter_mut()) {
            let (out_l, out_r, out_b) = self.tick(*l as f64, *r as f64);
            *l = out_l as f32;
            *r = out_r as f32;
            *b = out_b as f32;
        }
    }

    fn tick(&mut self, in_l: f64, in_r: f64) -> (f64, f64, f64) {
        let s = self.settings;
        let mono = (in_l + in_r) * 0.5;

        // Zero crossings are counted on band-limited copies, with hysteresis
        // scaled to their envelopes, so string noise doesn't add crossings.
        self.det1 = self.det1 * self.det_a + mono * (1.0 - self.det_a);
        self.det2 = self.det2 * self.det_a + self.det1 * (1.0 - self.det_a);
        self.det_y = self.det_hp * (self.det_y + self.det2 - self.det_x1);
        self.det_x1 = self.det2;
        self.det_env = self.det_y.abs().max(self.det_env * self.det_rel);
        let det_hyst = self.det_env * 0.2;
        if self.det_y < -det_hyst {
            self.det_armed = true;
        }
        self.det_since = (self.det_since + 1.0).min(self.max_period);
        if self.det_armed && self.det_y > det_hyst {
            self.det_armed = false;
            // det_y follows a jump a few cycles before the divider's band
            // does, so it switches Gate: Note sooner. It can read low notes
            // an octave high, never low, so one reading under the note
            // opens the gate. Readings over it only close the gate when
            // they're well over an octave above the band, and steady, so
            // a pick attack's noise doesn't cut the note it starts.
            let since = self.det_since.max(8.0);
            let hz = self.sample_rate / since;
            let steady = (since / self.det_last - 1.0).abs() < 0.06;
            self.det_last = since;
            if hz < self.note_open {
                self.vote_note(1, true);
            } else if steady && hz > self.note_close && hz > 2.1 * self.sample_rate / self.period_sm {
                self.vote_note(1, false);
            }
            self.det_period = self.det_period * 0.7 + self.det_since.max(8.0) * 0.3;
            self.det_since = 0.0;
            self.retune_band();
        }

        // On low strings the second harmonic can outweigh the fundamental,
        // doubling det_y's crossings. The divider counts a copy band-passed
        // around its own pitch instead, which can't settle more than an
        // octave below det_y's, or hum would capture it.
        let mut x = self.det_y;
        for i in 0..2 {
            let y = self.band_hp_a * (self.band_hp[i + 2] + x - self.band_hp[i]);
            self.band_hp[i] = x;
            self.band_hp[i + 2] = y;
            x = y;
        }
        let prev_band = self.band_lp[3];
        for y in &mut self.band_lp {
            *y = *y * self.band_lp_a + x * (1.0 - self.band_lp_a);
            x = *y;
        }
        self.band_env = x.abs().max(self.band_env * self.det_rel);
        let hyst = self.band_env * 0.2;

        let mut rising = false;
        if x < -hyst {
            self.armed = true;
        }
        if self.armed && x > hyst {
            rising = true;
            self.armed = false;
        }

        self.since_rising = (self.since_rising + 1.0).min(self.max_period);
        if rising {
            let hz = self.sample_rate / self.since_rising.max(8.0);
            let ok = if self.note_ok { hz <= self.note_close } else { hz < self.note_open };
            self.vote_note(0, ok);
            self.period_sm = self.period_sm * 0.7 + self.since_rising.max(8.0) * 0.3;
            self.since_rising = 0.0;
            self.retune_band();
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
                    let span = x - prev_band;
                    let at = if span > 0.0 { ((hyst - prev_band) / span).clamp(0.0, 1.0) } else { 1.0 };
                    let step = 2.0 * square;
                    self.held_bass += step * (1.0 - at) * (1.0 - at) * 0.5;
                    square - step * at * at * 0.5
                } else {
                    square
                }
            }
        };

        // The gate and dynamics read the detector's band-limited signal, so
        // pick scrapes and fret noise barely register.
        let am = self.det_y.abs();
        self.gate_env = if am > self.gate_env { am } else { self.gate_env * self.gate_rel };
        let gate = if self.gate_env > self.gate_thresh && self.note_ok { 1.0 } else { 0.0 };
        // Gate: Note cuts a note that's still ringing, so it fades faster.
        let coef = if gate > self.gate_sm {
            self.gate_att
        } else if self.note_ok {
            self.gate_rel_sm
        } else {
            self.note_rel_sm
        };
        self.gate_sm += (gate - self.gate_sm) * coef;

        self.dyn_env = am.max(self.dyn_env * self.dyn_rel);
        let follow = (self.dyn_env / DYNAMICS_FULL).min(1.0).powf(s.dynamics);
        self.dyn_sm += (follow - self.dyn_sm) * self.dyn_sm_a;

        let gain = self.gate_sm * self.dyn_sm * s.level * if s.stomp { 1.0 } else { 0.0 };

        let bass_out = self.shape(self.held_bass * self.held_gain, s) * s.blend;
        let [dry_l, dry_r] = self.held_dry;
        let result = match s.output {
            Output::Split => (dry_l, dry_r, bass_out),
            Output::Mix => (dry_l + bass_out, dry_r + bass_out, bass_out),
            Output::Bass => (bass_out, bass_out, bass_out),
        };

        self.held_bass = bass;
        self.held_gain = gain;
        self.held_dry = [in_l, in_r];
        result
    }

    /// Each cycle, the band (`source` 0) and det_y (1) vote on whether the
    /// note is under Gate: Note. Two in a row from either against the
    /// current state switch it, so one stray crossing can't chop the bass,
    /// except that one from det_y opens it.
    fn vote_note(&mut self, source: usize, ok: bool) {
        if ok == self.note_ok {
            self.note_votes[source] = 0;
        } else {
            self.note_votes[source] += 1;
            let needed = if source == 1 && ok { 1 } else { 2 };
            if self.note_votes[source] >= needed {
                self.note_ok = ok;
                self.note_votes = [0; 2];
            }
        }
    }

    fn retune_band(&mut self) {
        let lowest = self.sample_rate / (2.0 * self.det_period);
        let pitch = (self.sample_rate / self.period_sm).max(lowest);
        let nyquist = self.sample_rate * 0.45;
        self.band_hp_a = (-2.0 * PI * (lowest * 0.5).min(nyquist) / self.sample_rate).exp();
        self.band_lp_a = (-2.0 * PI * pitch.min(nyquist) / self.sample_rate).exp();
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
