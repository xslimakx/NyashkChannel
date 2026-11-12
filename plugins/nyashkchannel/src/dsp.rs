//! Channel strip: input stage, filters, expander / gate, compressor, 4-band EQ, output stage.
use nyashk_common::dsp::*;
use nyashk_common::eq::*;

#[derive(Clone, Copy)]
pub struct Params {
    pub power: bool,
    pub phase: bool,
    pub pad: bool,
    pub line_db: f32,
    pub mic_db: f32,
    pub analog: bool,
    pub hp_on: bool,
    pub hp_f: f32,
    pub lp_on: bool,
    pub lp_f: f32,
    pub split: bool, // filters feed the dynamics sidechain only
    pub x_on: bool,
    pub x_thr: f32,
    pub x_range: f32,
    pub x_rel_ms: f32,
    pub x_gate: bool,
    pub x_fast: bool,
    pub c_on: bool,
    pub c_ratio: f32,
    pub c_thr: f32,
    pub c_rel_ms: f32,
    pub c_fast: bool,
    pub c_auto: bool,
    pub dyn_post: bool,
    pub eq_on: bool,
    pub eq: EqCfg,
    pub out_db: f32,
    pub width: f32, // 0..2
    pub mono: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            power: true,
            phase: false,
            pad: false,
            line_db: 0.0,
            mic_db: 0.0,
            analog: false,
            hp_on: false,
            hp_f: 30.0,
            lp_on: false,
            lp_f: 18000.0,
            split: false,
            x_on: false,
            x_thr: -50.0,
            x_range: 20.0,
            x_rel_ms: 300.0,
            x_gate: false,
            x_fast: false,
            c_on: false,
            c_ratio: 3.0,
            c_thr: -10.0,
            c_rel_ms: 400.0,
            c_fast: false,
            c_auto: false,
            dyn_post: false,
            eq_on: true,
            eq: EqCfg::default(),
            out_db: 0.0,
            width: 1.0,
            mono: false,
        }
    }
}

/// Magnitude (dB) of the HP (18 dB/oct) and LP (12 dB/oct) filters at `f`, for the display.
pub fn filter_response_db(fs: f32, hp_on: bool, hp_f: f32, lp_on: bool, lp_f: f32, f: f32) -> f32 {
    let mut d = 0.0;
    if hp_on {
        let hpf = hp_f.max(10.0);
        d += Coef::highpass1(fs, hpf).response_db(fs, f) + Coef::highpass(fs, hpf, 1.0).response_db(fs, f);
    }
    if lp_on {
        d += Coef::lowpass(fs, lp_f, 0.7071).response_db(fs, f);
    }
    d
}

/// Static curve of the expander / gate and the compressor: output level (dB) for a steady input level.
#[allow(clippy::too_many_arguments)]
pub fn dyn_transfer_db(in_db: f32, x_on: bool, x_gate: bool, x_thr: f32, x_range: f32, c_on: bool, c_thr: f32, c_ratio: f32, c_auto: bool) -> f32 {
    let mut out = in_db;
    if x_on {
        out += expander_gain_db(in_db, x_thr, if x_gate { 12.0 } else { 2.0 }, x_range);
    }
    if c_on {
        out += comp_gain_db(in_db, c_thr, c_ratio, 4.0);
        if c_auto {
            out += auto_makeup_db(c_thr, c_ratio);
        }
    }
    out
}

/// Gain computer of the compressor (soft knee). Returns gain in dB (<= 0).
pub fn comp_gain_db(in_db: f32, thr: f32, ratio: f32, knee: f32) -> f32 {
    let over = in_db - thr;
    let slope = 1.0 / ratio.max(1.0) - 1.0;
    if 2.0 * over < -knee {
        0.0
    } else if 2.0 * over.abs() <= knee {
        slope * (over + knee / 2.0).powi(2) / (2.0 * knee)
    } else {
        slope * over
    }
}

/// Downward expander / gate gain computer. Returns gain in dB (<= 0), limited by `range`.
pub fn expander_gain_db(in_db: f32, thr: f32, ratio: f32, range: f32) -> f32 {
    if in_db >= thr {
        0.0
    } else {
        (-(thr - in_db) * (ratio - 1.0)).max(-range)
    }
}

pub fn auto_makeup_db(thr: f32, ratio: f32) -> f32 {
    (-thr * (1.0 - 1.0 / ratio.max(1.0)) * 0.5).max(0.0)
}

pub struct Strip {
    fs: f32,
    eq_st: [EqState; 2],
    eq_c: EqCoefs,
    hp: [Chain<2>; 2],
    lp: [Bq; 2],
    sc_hp: [Chain<2>; 2],
    sc_lp: [Bq; 2],
    filt: [Coef; 4], // hp1, hp2(q=1), unused slot, lp
    os: [Os2; 2],
    tick: u32,
    x_gain: f32, // current gate gain (dB)
    x_hold: u32,
    c_gain: f32, // current compressor gain reduction (dB, <= 0)
    sm_in: Smooth,
    sm_out: Smooth,
    sm_pow: Smooth,
    sm_w: Smooth,
    pub in_peak: [f32; 2],
    pub out_peak: [f32; 2],
    pub gr_comp: f32,
    pub gr_exp: f32,
}

impl Strip {
    pub fn new(fs: f32) -> Self {
        Self {
            fs,
            eq_st: Default::default(),
            eq_c: EqCoefs::new(fs, &EqCfg::default()),
            hp: Default::default(),
            lp: Default::default(),
            sc_hp: [Chain::new(), Chain::new()],
            sc_lp: Default::default(),
            filt: [Coef::THRU; 4],
            os: [Os2::new(fs), Os2::new(fs)],
            tick: 0,
            x_gain: 0.0,
            x_hold: 0,
            c_gain: 0.0,
            sm_in: Smooth::new(1.0, fs, 12.0),
            sm_out: Smooth::new(1.0, fs, 12.0),
            sm_pow: Smooth::new(1.0, fs, 8.0),
            sm_w: Smooth::new(1.0, fs, 12.0),
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            gr_comp: 0.0,
            gr_exp: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.eq_st = Default::default();
        self.hp = Default::default();
        self.lp = Default::default();
        self.sc_hp = [Chain::new(), Chain::new()];
        self.sc_lp = Default::default();
        for o in self.os.iter_mut() {
            o.reset();
        }
        self.x_gain = 0.0;
        self.c_gain = 0.0;
    }

    fn update(&mut self, p: &Params) {
        let fs = self.fs;
        self.eq_c = EqCoefs::new(fs, &p.eq);
        let hpf = p.hp_f.max(10.0);
        self.filt = [Coef::highpass1(fs, hpf), Coef::highpass(fs, hpf, 1.0), Coef::THRU, Coef::lowpass(fs, p.lp_f, 0.7071)];
    }

    pub fn process(&mut self, p: &Params, x: &mut [f32; 2], n: usize) {
        if self.tick % 16 == 0 {
            self.update(p);
        }
        self.tick = self.tick.wrapping_add(1);
        let fs = self.fs;
        let in_g = self.sm_in.tick(db_to_lin(p.line_db + p.mic_db - if p.pad { 20.0 } else { 0.0 }) * if p.phase { -1.0 } else { 1.0 });
        let out_g = self.sm_out.tick(db_to_lin(p.out_db));
        let pw = self.sm_pow.tick(if p.power { 1.0 } else { 0.0 });
        let width = self.sm_w.tick(if p.mono { 0.0 } else { p.width });
        let dry = *x;

        // ---- input stage
        let mut ipk = [0.0f32; 2];
        for i in 0..n {
            ipk[i] = x[i].abs();
            let mut v = x[i] * in_g;
            if p.analog {
                let kk = 1.0 + p.mic_db / 25.0;
                v = self.os[i].process(v, |a| sat_asym(a * kk, 0.08) / kk);
            }
            x[i] = v;
        }
        self.in_peak = ipk;

        // ---- filters
        let f = self.filt;
        if !p.split {
            for i in 0..n {
                let mut v = x[i];
                if p.hp_on {
                    v = self.hp[i].tick(&[f[0], f[1]], v); // 18 dB/oct: 1st order + biquad
                }
                if p.lp_on {
                    v = self.lp[i].tick(&f[3], v);
                }
                x[i] = v;
            }
        }

        // ---- EQ / dynamics order
        if !p.dyn_post {
            self.dynamics(p, x, n);
            self.eq(p, x, n);
        } else {
            self.eq(p, x, n);
            self.dynamics(p, x, n);
        }

        // ---- output stage: width (mid/side), fader
        if n == 2 {
            let (m, s) = ((x[0] + x[1]) * 0.5, (x[0] - x[1]) * 0.5 * width);
            x[0] = m + s;
            x[1] = m - s;
        }
        let mut opk = [0.0f32; 2];
        for i in 0..n {
            x[i] *= out_g;
            let o = dry[i] + (x[i] - dry[i]) * pw;
            x[i] = o;
            opk[i] = o.abs();
        }
        let _ = fs;
        self.out_peak = opk;
    }

    fn eq(&mut self, p: &Params, x: &mut [f32; 2], n: usize) {
        if !p.eq_on {
            return;
        }
        for i in 0..n {
            x[i] = self.eq_st[i].tick(&self.eq_c, x[i]);
        }
    }

    fn dynamics(&mut self, p: &Params, x: &mut [f32; 2], n: usize) {
        if !p.x_on && !p.c_on {
            self.gr_comp = 0.0;
            self.gr_exp = 0.0;
            self.x_gain *= 0.999;
            self.c_gain *= 0.999;
            return;
        }
        let fs = self.fs;
        // sidechain level: linked peak, optionally filtered when SPLIT is on
        let mut lvl = 0.0f32;
        for i in 0..n {
            let mut v = x[i];
            if p.split {
                if p.hp_on {
                    v = self.sc_hp[i].tick(&[self.filt[0], self.filt[1]], v);
                }
                if p.lp_on {
                    v = self.sc_lp[i].tick(&self.filt[3], v);
                }
            }
            lvl = lvl.max(v.abs());
        }
        let in_db = lin_to_db(lvl.max(1e-6));
        let mut gain_db = 0.0f32;

        if p.x_on {
            let ratio = if p.x_gate { 12.0 } else { 2.0 };
            let target = expander_gain_db(in_db, p.x_thr, ratio, p.x_range);
            let (att, rel) = (time_coef(if p.x_fast { 0.1 } else { 1.0 }, fs), time_coef(p.x_rel_ms, fs));
            if in_db >= p.x_thr {
                self.x_hold = (0.012 * fs) as u32;
            }
            let t = if self.x_hold > 0 {
                self.x_hold -= 1;
                0.0
            } else {
                target
            };
            let c = if t > self.x_gain { att } else { rel };
            self.x_gain = t + (self.x_gain - t) * c;
            gain_db += self.x_gain;
            self.gr_exp = -self.x_gain;
        } else {
            self.gr_exp = 0.0;
        }

        if p.c_on {
            // compressor detects after the expander so the gate can not pump it
            let cdb = in_db + gain_db;
            let target = comp_gain_db(cdb, p.c_thr, p.c_ratio, 4.0);
            let (att, rel) = (time_coef(if p.c_fast { 0.3 } else { 6.0 }, fs), time_coef(p.c_rel_ms, fs));
            let c = if target < self.c_gain { att } else { rel };
            self.c_gain = target + (self.c_gain - target) * c;
            gain_db += self.c_gain;
            if p.c_auto {
                gain_db += auto_makeup_db(p.c_thr, p.c_ratio);
            }
            self.gr_comp = -self.c_gain;
        } else {
            self.gr_comp = 0.0;
        }
        let g = db_to_lin(gain_db);
        for i in 0..n {
            x[i] *= g;
        }
    }
}
