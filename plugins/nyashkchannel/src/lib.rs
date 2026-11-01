//! NyashkChannel: channel strip with input stage, filters, expander / gate, compressor, 4-band EQ and output stage (VST3).

use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use nyashk_common::dsp::*;
use nyashk_common::eq::*;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub mod dsp;
mod editor;

pub struct Meters {
    vals: [AtomicU32; 8],
    pub fs: AtomicU32,
}
pub const M_IN_L: usize = 0;
pub const M_IN_R: usize = 1;
pub const M_OUT_L: usize = 2;
pub const M_OUT_R: usize = 3;
pub const M_GR_C: usize = 4;
pub const M_GR_X: usize = 5;
impl Meters {
    fn new() -> Self {
        let m = (-90f32).to_bits();
        Self {
            vals: [AtomicU32::new(m), AtomicU32::new(m), AtomicU32::new(m), AtomicU32::new(m), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0)],
            fs: AtomicU32::new(48000f32.to_bits()),
        }
    }
    pub fn get(&self, i: usize) -> f32 {
        f32::from_bits(self.vals[i].load(Ordering::Relaxed))
    }
    fn set(&self, i: usize, v: f32) {
        self.vals[i].store(v.to_bits(), Ordering::Relaxed);
    }
}

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Voice {
    #[name = "BROWN"]
    Brown,
    #[name = "BLACK"]
    Black,
}

pub fn fmt_hz(v: f32) -> String {
    if v >= 1000.0 {
        format!("{:.2}k", v / 1000.0)
    } else {
        format!("{:.0}", v)
    }
}
pub fn parse_hz(s: &str) -> Option<f32> {
    let t = s.trim().to_lowercase().replace("hz", "");
    let t = t.trim();
    if let Some(k) = t.strip_suffix('k') {
        k.trim().parse::<f32>().ok().map(|v| v * 1000.0)
    } else {
        t.parse().ok()
    }
}
fn freq(name: &str, def: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(name, def, FloatRange::Skewed { min, max, factor: FloatRange::skew_factor(-2.0) })
        .with_smoother(SmoothingStyle::Logarithmic(25.0))
        .with_value_to_string(Arc::new(fmt_hz))
        .with_string_to_value(Arc::new(parse_hz))
}
fn db(name: &str, min: f32, max: f32, def: f32) -> FloatParam {
    FloatParam::new(name, def, FloatRange::Linear { min, max })
        .with_step_size(0.1)
        .with_smoother(SmoothingStyle::Linear(25.0))
        .with_value_to_string(Arc::new(|v| format!("{:+.1}", v)))
}
fn qp(name: &str) -> FloatParam {
    FloatParam::new(name, 1.0, FloatRange::Skewed { min: 0.3, max: 8.0, factor: FloatRange::skew_factor(-1.5) })
        .with_step_size(0.01)
        .with_smoother(SmoothingStyle::Linear(25.0))
        .with_value_to_string(Arc::new(|v| format!("{:.2}", v)))
}
fn ms(name: &str, def: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(name, def, FloatRange::Skewed { min, max, factor: FloatRange::skew_factor(-2.0) })
        .with_value_to_string(Arc::new(|v| if v >= 1000.0 { format!("{:.2} s", v / 1000.0) } else { format!("{:.0} ms", v) }))
        .with_string_to_value(Arc::new(|s| {
            let t = s.trim().to_lowercase();
            if let Some(v) = t.strip_suffix("ms") {
                v.trim().parse().ok()
            } else if let Some(v) = t.strip_suffix('s') {
                v.trim().parse::<f32>().ok().map(|x| x * 1000.0)
            } else {
                t.parse().ok()
            }
        }))
}

#[derive(Params)]
pub struct ChannelParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "power"]
    pub power: BoolParam,
    #[id = "phase"]
    pub phase: BoolParam,
    #[id = "pad"]
    pub pad: BoolParam,
    #[id = "line"]
    pub line: FloatParam,
    #[id = "mic"]
    pub mic: FloatParam,
    #[id = "analog"]
    pub analog: BoolParam,

    #[id = "hpon"]
    pub hp_on: BoolParam,
    #[id = "hpf"]
    pub hp_f: FloatParam,
    #[id = "lpon"]
    pub lp_on: BoolParam,
    #[id = "lpf"]
    pub lp_f: FloatParam,
    #[id = "split"]
    pub split: BoolParam,

    #[id = "xon"]
    pub x_on: BoolParam,
    #[id = "xthr"]
    pub x_thr: FloatParam,
    #[id = "xrange"]
    pub x_range: FloatParam,
    #[id = "xrel"]
    pub x_rel: FloatParam,
    #[id = "xgate"]
    pub x_gate: BoolParam,
    #[id = "xfast"]
    pub x_fast: BoolParam,

    #[id = "con"]
    pub c_on: BoolParam,
    #[id = "cratio"]
    pub c_ratio: FloatParam,
    #[id = "cthr"]
    pub c_thr: FloatParam,
    #[id = "crel"]
    pub c_rel: FloatParam,
    #[id = "cfast"]
    pub c_fast: BoolParam,
    #[id = "cauto"]
    pub c_auto: BoolParam,
    #[id = "dynpost"]
    pub dyn_post: BoolParam,

    #[id = "eqon"]
    pub eq_on: BoolParam,
    #[id = "voice"]
    pub voice: EnumParam<Voice>,
    #[id = "lfg"]
    pub lf_g: FloatParam,
    #[id = "lff"]
    pub lf_f: FloatParam,
    #[id = "lfb"]
    pub lf_bell: BoolParam,
    #[id = "lmg"]
    pub lmf_g: FloatParam,
    #[id = "lmf"]
    pub lmf_f: FloatParam,
    #[id = "lmq"]
    pub lmf_q: FloatParam,
    #[id = "hmg"]
    pub hmf_g: FloatParam,
    #[id = "hmf"]
    pub hmf_f: FloatParam,
    #[id = "hmq"]
    pub hmf_q: FloatParam,
    #[id = "hfg"]
    pub hf_g: FloatParam,
    #[id = "hff"]
    pub hf_f: FloatParam,
    #[id = "hfb"]
    pub hf_bell: BoolParam,

    #[id = "out"]
    pub out: FloatParam,
    #[id = "width"]
    pub width: FloatParam,
    #[id = "mono"]
    pub mono: BoolParam,
}

impl Default for ChannelParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(1000, 600),
            power: BoolParam::new("Power", true).make_bypass(),
            phase: BoolParam::new("Phase", false),
            pad: BoolParam::new("Pad -20 dB", false),
            line: db("Line", -20.0, 20.0, 0.0),
            mic: db("Mic", 0.0, 50.0, 0.0),
            analog: BoolParam::new("Analog", false),
            hp_on: BoolParam::new("HP On", false),
            hp_f: freq("HP Freq", 30.0, 16.0, 350.0),
            lp_on: BoolParam::new("LP On", false),
            lp_f: freq("LP Freq", 18000.0, 3000.0, 22000.0),
            split: BoolParam::new("Filters To Sidechain", false),
            x_on: BoolParam::new("Expander On", false),
            x_thr: db("Expander Threshold", -60.0, 0.0, -45.0),
            x_range: FloatParam::new("Expander Range", 20.0, FloatRange::Linear { min: 0.0, max: 40.0 }).with_step_size(0.1).with_unit(" dB"),
            x_rel: ms("Expander Release", 300.0, 50.0, 4000.0),
            x_gate: BoolParam::new("Gate", false),
            x_fast: BoolParam::new("Expander Fast Attack", false),
            c_on: BoolParam::new("Compressor On", false),
            c_ratio: FloatParam::new("Ratio", 3.0, FloatRange::Skewed { min: 1.0, max: 20.0, factor: FloatRange::skew_factor(-1.5) })
                .with_value_to_string(Arc::new(|v| if v >= 19.5 { "INF".to_string() } else { format!("{:.1}:1", v) }))
                .with_string_to_value(Arc::new(|s| {
                    let t = s.trim().to_lowercase();
                    if t == "inf" { Some(20.0) } else { t.trim_end_matches(":1").trim().parse().ok() }
                })),
            c_thr: db("Compressor Threshold", -40.0, 10.0, -12.0),
            c_rel: ms("Compressor Release", 400.0, 50.0, 4000.0),
            c_fast: BoolParam::new("Compressor Fast Attack", false),
            c_auto: BoolParam::new("Auto Makeup", true),
            dyn_post: BoolParam::new("Dynamics After EQ", false),
            eq_on: BoolParam::new("EQ In", true),
            voice: EnumParam::new("EQ Type", Voice::Brown),
            lf_g: db("LF Gain", -16.0, 16.0, 0.0),
            lf_f: freq("LF Freq", 100.0, 30.0, 450.0),
            lf_bell: BoolParam::new("LF Bell", false),
            lmf_g: db("LMF Gain", -16.0, 16.0, 0.0),
            lmf_f: freq("LMF Freq", 600.0, 200.0, 2500.0),
            lmf_q: qp("LMF Q"),
            hmf_g: db("HMF Gain", -16.0, 16.0, 0.0),
            hmf_f: freq("HMF Freq", 3000.0, 600.0, 7000.0),
            hmf_q: qp("HMF Q"),
            hf_g: db("HF Gain", -16.0, 16.0, 0.0),
            hf_f: freq("HF Freq", 10000.0, 1500.0, 16000.0),
            hf_bell: BoolParam::new("HF Bell", false),
            out: FloatParam::new("Output", 0.0, FloatRange::Skewed { min: -60.0, max: 12.0, factor: FloatRange::skew_factor(1.2) })
                .with_step_size(0.1)
                .with_smoother(SmoothingStyle::Linear(25.0))
                .with_value_to_string(Arc::new(|v| if v <= -59.9 { "-inf".to_string() } else { format!("{:+.1}", v) }))
                .with_string_to_value(Arc::new(|s| if s.trim().eq_ignore_ascii_case("-inf") { Some(-60.0) } else { s.trim().parse().ok() })),
            width: FloatParam::new("Width", 100.0, FloatRange::Linear { min: 0.0, max: 200.0 }).with_unit(" %").with_step_size(1.0).with_smoother(SmoothingStyle::Linear(25.0)),
            mono: BoolParam::new("Mono", false),
        }
    }
}

impl ChannelParams {
    pub fn eq_cfg(&self) -> EqCfg {
        EqCfg {
            hp_on: false,
            hp_f: 30.0,
            lp_on: false,
            lp_f: 18000.0,
            lf: Band { freq: self.lf_f.value(), gain_db: self.lf_g.value(), q: 1.0, bell: self.lf_bell.value() },
            lmf: Band { freq: self.lmf_f.value(), gain_db: self.lmf_g.value(), q: self.lmf_q.value(), bell: true },
            hmf: Band { freq: self.hmf_f.value(), gain_db: self.hmf_g.value(), q: self.hmf_q.value(), bell: true },
            hf: Band { freq: self.hf_f.value(), gain_db: self.hf_g.value(), q: 1.0, bell: self.hf_bell.value() },
            // BROWN = proportional-Q bells and broad shelves, BLACK = constant-Q
            voicing: match self.voice.value() {
                Voice::Brown => Voicing::Brown,
                Voice::Black => Voicing::Black,
            },
        }
    }
}

pub struct NyashkChannel {
    params: Arc<ChannelParams>,
    meters: Arc<Meters>,
    strip: Option<dsp::Strip>,
}

impl Default for NyashkChannel {
    fn default() -> Self {
        Self { params: Arc::new(ChannelParams::default()), meters: Arc::new(Meters::new()), strip: None }
    }
}

impl Plugin for NyashkChannel {
    const NAME: &'static str = "NyashkChannel";
    const VENDOR: &'static str = "Nyashk";
    const URL: &'static str = "https://github.com/xslimakx";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout { main_input_channels: NonZeroU32::new(2), main_output_channels: NonZeroU32::new(2), ..AudioIOLayout::const_default() },
        AudioIOLayout { main_input_channels: NonZeroU32::new(1), main_output_channels: NonZeroU32::new(1), ..AudioIOLayout::const_default() },
    ];
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _e: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.meters.clone())
    }

    fn initialize(&mut self, _l: &AudioIOLayout, config: &BufferConfig, _c: &mut impl InitContext<Self>) -> bool {
        self.strip = Some(dsp::Strip::new(config.sample_rate));
        self.meters.fs.store(config.sample_rate.to_bits(), Ordering::Relaxed);
        true
    }

    fn reset(&mut self) {
        if let Some(s) = self.strip.as_mut() {
            s.reset();
        }
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _c: &mut impl ProcessContext<Self>) -> ProcessStatus {
        let Some(strip) = self.strip.as_mut() else { return ProcessStatus::Normal };
        let p = &self.params;
        let mut dp = dsp::Params::default();
        let (mut ipk, mut opk) = ([0.0f32; 2], [0.0f32; 2]);
        let (mut grc, mut grx) = (0.0f32, 0.0f32);
        for (k, mut frame) in buffer.iter_samples().enumerate() {
            if k % 16 == 0 {
                dp = dsp::Params {
                    power: p.power.value(),
                    phase: p.phase.value(),
                    pad: p.pad.value(),
                    line_db: p.line.smoothed.next_step(16),
                    mic_db: p.mic.smoothed.next_step(16),
                    analog: p.analog.value(),
                    hp_on: p.hp_on.value(),
                    hp_f: p.hp_f.smoothed.next_step(16),
                    lp_on: p.lp_on.value(),
                    lp_f: p.lp_f.smoothed.next_step(16),
                    split: p.split.value(),
                    x_on: p.x_on.value(),
                    x_thr: p.x_thr.smoothed.next_step(16),
                    x_range: p.x_range.value(),
                    x_rel_ms: p.x_rel.value(),
                    x_gate: p.x_gate.value(),
                    x_fast: p.x_fast.value(),
                    c_on: p.c_on.value(),
                    c_ratio: p.c_ratio.value(),
                    c_thr: p.c_thr.smoothed.next_step(16),
                    c_rel_ms: p.c_rel.value(),
                    c_fast: p.c_fast.value(),
                    c_auto: p.c_auto.value(),
                    dyn_post: p.dyn_post.value(),
                    eq_on: p.eq_on.value(),
                    eq: {
                        let mut c = p.eq_cfg();
                        c.lf.freq = p.lf_f.smoothed.next_step(16);
                        c.lf.gain_db = p.lf_g.smoothed.next_step(16);
                        c.lmf.freq = p.lmf_f.smoothed.next_step(16);
                        c.lmf.gain_db = p.lmf_g.smoothed.next_step(16);
                        c.lmf.q = p.lmf_q.smoothed.next_step(16);
                        c.hmf.freq = p.hmf_f.smoothed.next_step(16);
                        c.hmf.gain_db = p.hmf_g.smoothed.next_step(16);
                        c.hmf.q = p.hmf_q.smoothed.next_step(16);
                        c.hf.freq = p.hf_f.smoothed.next_step(16);
                        c.hf.gain_db = p.hf_g.smoothed.next_step(16);
                        c
                    },
                    out_db: p.out.smoothed.next_step(16),
                    width: p.width.smoothed.next_step(16) / 100.0,
                    mono: p.mono.value(),
                };
                if dp.out_db <= -59.9 {
                    dp.out_db = -120.0;
                }
            }
            let n = frame.len().min(2);
            let mut x = [0.0f32; 2];
            for (i, s) in frame.iter_mut().enumerate().take(n) {
                x[i] = *s;
            }
            strip.process(&dp, &mut x, n);
            for i in 0..n {
                ipk[i] = ipk[i].max(strip.in_peak[i]);
                opk[i] = opk[i].max(strip.out_peak[i]);
            }
            grc = grc.max(strip.gr_comp);
            grx = grx.max(strip.gr_exp);
            for (i, s) in frame.iter_mut().enumerate().take(n) {
                *s = x[i];
            }
        }
        let d = |v: f32| lin_to_db(v.max(1e-5));
        let m = &self.meters;
        m.set(M_IN_L, d(ipk[0]));
        m.set(M_IN_R, d(ipk[1].max(if ipk[1] == 0.0 { ipk[0] } else { 0.0 })));
        m.set(M_OUT_L, d(opk[0]));
        m.set(M_OUT_R, d(opk[1].max(if opk[1] == 0.0 { opk[0] } else { 0.0 })));
        m.set(M_GR_C, grc);
        m.set(M_GR_X, grx);
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for NyashkChannel {
    const VST3_CLASS_ID: [u8; 16] = *b"NyashkChanVST3!!";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics, Vst3SubCategory::Eq];
}

nih_export_vst3!(NyashkChannel);
