//! NyashkChannel GUI: modern analog console strip (pre-rendered in tools/render_ui.py, see hw.rs).
use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, epaint::PathStroke, Align2, Color32, Pos2, Rect, Stroke, Vec2},
};
use nyashk_common::eq::EqCoefs;
use nyashk_common::ui::{resize_grip, set_b, set_e};
use std::sync::Arc;
use std::time::Duration;

use crate::hw::{self, Hw};
use crate::hw_layout as lay;
use crate::*;

const BASE_W: f32 = lay::W;
const BASE_H: f32 = lay::H;

struct Gui {
    hw: Option<Hw>,
    spec: nyashk_common::analyzer::Spectrum,
    lv: [f32; 4],
    hold: [f32; 4],
    gr: [f32; 2],
    /// graph view: false = EQ + filters, true = dynamics curve
    dyn_view: bool,
}

fn k(id: &str) -> &'static hw::KnobL {
    hw::find(lay::KNOBS, id, |x| x.id)
}
fn b(id: &str) -> &'static hw::BtnL {
    hw::find(lay::BUTTONS, id, |x| x.id)
}

const C_RED: Color32 = Color32::from_rgb(255, 80, 64);
const C_GRN: Color32 = Color32::from_rgb(60, 220, 110);
const C_BLU: Color32 = Color32::from_rgb(80, 150, 255);
const C_BRN: Color32 = Color32::from_rgb(230, 150, 70);
const C_WHT: Color32 = Color32::from_rgb(235, 235, 230);

pub fn create(params: Arc<ChannelParams>, meters: Arc<Meters>) -> Option<Box<dyn Editor>> {
    let editor_state = params.editor_state.clone();
    let size_state = editor_state.clone();
    create_egui_editor(
        editor_state,
        Gui { hw: None, spec: nyashk_common::analyzer::Spectrum::new(4096), lv: [-90.0; 4], hold: [-90.0; 4], gr: [0.0; 2], dyn_view: false },
        |ctx, st| {
            st.hw = None;
            let mut v = egui::Visuals::dark();
            v.override_text_color = Some(Color32::from_rgb(232, 232, 228));
            ctx.set_visuals(v);
        },
        move |ctx, setter, st| {
            ctx.request_repaint_after(Duration::from_millis(33));
            if st.hw.is_none() {
                st.hw = Some(Hw::load(ctx, lay::ASSETS));
            }
            let p = &*params;
            let fs = f32::from_bits(meters.fs.load(std::sync::atomic::Ordering::Relaxed)).max(8000.0);
            st.spec.update(&meters.tap);
            let cur = [meters.get(M_IN_L).max(meters.get(M_IN_R)), meters.get(M_OUT_L).max(meters.get(M_OUT_R))];
            for i in 0..2 {
                st.lv[i] = if cur[i] > st.lv[i] { cur[i] } else { st.lv[i] - 1.2 };
                st.hold[i] = st.hold[i].max(st.lv[i]) - 0.12;
            }
            for (i, m) in [M_GR_X, M_GR_C].into_iter().enumerate() {
                let t = meters.get(m);
                st.gr[i] = if t > st.gr[i] { t } else { st.gr[i] - 0.6 }.max(0.0);
            }
            let accent = hw::c32(lay::ACCENT);
            egui::CentralPanel::default().frame(egui::Frame::NONE.fill(Color32::from_rgb(14, 14, 15))).show(ctx, |ui| {
                let full = ui.max_rect();
                st.hw.as_mut().unwrap().o = full.min;
                let hwr = st.hw.as_ref().unwrap();
                let painter = ui.painter().clone();
                hwr.background(&painter, BASE_W, BASE_H);
                let on = p.power.value();
                let tg = |ui: &mut egui::Ui, id: &str, bp: &BoolParam, tip: &str| {
                    let v = bp.value();
                    if hw::button(ui, hwr, b(id), v, tip) {
                        set_b(setter, bp, !v);
                    }
                };
                tg(ui, "power", &p.power, "Switch the whole strip on or off (smooth, click-free).");
                // ---- input / filters
                tg(ui, "phase", &p.phase, "Invert the polarity.");
                tg(ui, "pad", &p.pad, "-20 dB pad for hot signals.");
                tg(ui, "analog", &p.analog, "Adds gentle transformer-style saturation to the input stage.");
                hw::knob(ui, hwr, setter, &p.line, k("line"), C_WHT, "LINE: line trim, +/- 20 dB.");
                hw::knob(ui, hwr, setter, &p.mic, k("mic"), C_RED, "MIC: mic preamp gain, 0 to 50 dB.");
                tg(ui, "hp_on", &p.hp_on, "High-pass filter (18 dB/oct).");
                tg(ui, "lp_on", &p.lp_on, "Low-pass filter (12 dB/oct).");
                if hw::knob(ui, hwr, setter, &p.hp_f, k("hp_f"), C_WHT, "High-pass frequency.") != 0.0 {
                    st.dyn_view = false;
                }
                if hw::knob(ui, hwr, setter, &p.lp_f, k("lp_f"), C_WHT, "Low-pass frequency.") != 0.0 {
                    st.dyn_view = false;
                }
                tg(ui, "split", &p.split, "SPLIT: the filters act on the dynamics sidechain only, not on the audio.");
                // ---- dynamics
                tg(ui, "x_on", &p.x_on, "Switch the expander on.");
                tg(ui, "x_gate", &p.x_gate, "Hard gate instead of a soft expander.");
                tg(ui, "x_fast", &p.x_fast, "Fast attack.");
                if hw::knob(ui, hwr, setter, &p.x_thr, k("x_thr"), C_GRN, "Level below which the signal is reduced.") != 0.0 {
                    st.dyn_view = true;
                }
                if hw::knob(ui, hwr, setter, &p.x_range, k("x_range"), C_GRN, "Maximum reduction in dB.") != 0.0 {
                    st.dyn_view = true;
                }
                hw::knob(ui, hwr, setter, &p.x_rel, k("x_rel"), C_GRN, "Release time.");
                tg(ui, "c_on", &p.c_on, "Switch the compressor on.");
                tg(ui, "c_fast", &p.c_fast, "Fast attack.");
                tg(ui, "c_auto", &p.c_auto, "Automatic makeup gain.");
                if hw::knob(ui, hwr, setter, &p.c_thr, k("c_thr"), C_WHT, "Level above which compression starts.") != 0.0 {
                    st.dyn_view = true;
                }
                if hw::knob(ui, hwr, setter, &p.c_ratio, k("c_ratio"), C_WHT, "Compression ratio.") != 0.0 {
                    st.dyn_view = true;
                }
                hw::knob(ui, hwr, setter, &p.c_rel, k("c_rel"), C_WHT, "Release time.");
                tg(ui, "dyn_post", &p.dyn_post, "Lit: the dynamics come after the EQ. Off: before the EQ.");

                // ---- graph
                let scr = hw::find(lay::SCREENS, "graph", |x| x.id);
                let g = hwr.rect(scr.rect).shrink(6.0);
                let (f_lo, f_hi, d_r) = (20.0f32, 20000.0f32, 18.0f32);
                let fxf = |f: f32| g.left() + g.width() * (f / f_lo).ln() / (f_hi / f_lo).ln();
                let dyf = |d: f32| g.center().y - g.height() * 0.5 * (d / d_r).clamp(-1.0, 1.0);
                if !st.dyn_view {
                    for d in [-12.0f32, -6.0, 0.0, 6.0, 12.0] {
                        painter.hline(g.left()..=g.right(), dyf(d), Stroke::new(1.0_f32, if d == 0.0 { Color32::from_rgb(120, 122, 126) } else { Color32::from_rgb(30, 32, 36) }));
                        painter.text(Pos2::new(g.left() + 4.0, dyf(d) - 6.0), Align2::LEFT_CENTER, if d.abs() < 0.5 { "0".to_string() } else { format!("{:+.0}", d) }, egui::FontId::monospace(9.0), Color32::from_rgb(110, 112, 116));
                    }
                    for f in [50.0f32, 100.0, 500.0, 1000.0, 5000.0, 10000.0] {
                        painter.vline(fxf(f), g.top()..=g.bottom(), Stroke::new(1.0_f32, Color32::from_rgb(30, 32, 36)));
                        painter.text(Pos2::new(fxf(f), g.bottom() - 8.0), Align2::CENTER_CENTER, if f >= 1000.0 { format!("{}k", f / 1000.0) } else { format!("{}", f) }, egui::FontId::monospace(9.0), Color32::from_rgb(110, 112, 116));
                    }
                    // live output spectrum behind the curve (tilted so pink noise looks flat)
                    let spy = |d: f32| g.bottom() - g.height() * ((d + 90.0) / 90.0).clamp(-0.05, 1.05);
                    let sp = st.spec.points(fs, f_lo, f_hi.min(fs * 0.49), 180, 4.5, fxf, spy);
                    nyashk_common::analyzer::draw_spectrum(&painter, g, &sp, Color32::from_gray(26), Color32::from_gray(72));
                    for d in [-12.0f32, -6.0, 6.0, 12.0] {
                        painter.with_clip_rect(g).hline(g.left()..=g.right(), dyf(d), Stroke::new(1.0_f32, Color32::from_white_alpha(6)));
                    }
                    for f in [50.0f32, 100.0, 500.0, 1000.0, 5000.0, 10000.0] {
                        painter.with_clip_rect(g).vline(fxf(f), g.top()..=g.bottom(), Stroke::new(1.0_f32, Color32::from_white_alpha(5)));
                        painter.text(Pos2::new(fxf(f), g.bottom() - 8.0), Align2::CENTER_CENTER, if f >= 1000.0 { format!("{}k", f / 1000.0) } else { format!("{}", f) }, egui::FontId::monospace(9.0), Color32::from_rgb(110, 112, 116));
                    }
                }
                let gclip = painter.with_clip_rect(g);
                if !st.dyn_view {
                    let eq_on = p.eq_on.value() && on;
                    let cf = EqCoefs::new(fs, &p.eq_cfg());
                    let (hp_on, lp_on, split) = (p.hp_on.value() && on, p.lp_on.value() && on, p.split.value());
                    let filt = |f: f32| crate::dsp::filter_response_db(fs, hp_on, p.hp_f.value(), lp_on, p.lp_f.value(), f);
                    let n = 240;
                    let fr = |i: usize| f_lo * (f_hi / f_lo).powf(i as f32 / n as f32);
                    // filters on their own (dashed when they only feed the dynamics sidechain)
                    if hp_on || lp_on {
                        let fp: Vec<Pos2> = (0..=n).map(|i| Pos2::new(fxf(fr(i)), dyf(filt(fr(i))))).collect();
                        if split {
                            for w in fp.chunks(6).step_by(2) {
                                gclip.add(egui::Shape::line(w.to_vec(), PathStroke::new(1.5_f32, Color32::from_rgb(120, 122, 126))));
                            }
                        } else {
                            nyashk_common::ui::fill_under(&gclip, &fp, dyf(0.0), Color32::from_white_alpha(10));
                            gclip.add(egui::Shape::line(fp, PathStroke::new(1.2_f32, Color32::from_rgb(120, 122, 126))));
                        }
                        for (on_f, f, name) in [(hp_on, p.hp_f.value(), "HP"), (lp_on, p.lp_f.value(), "LP")] {
                            if on_f {
                                let x = fxf(f.clamp(f_lo, f_hi));
                                let y = dyf(filt(f)).min(g.bottom() - 30.0);
                                gclip.circle_filled(Pos2::new(x, y), 3.5, accent);
                                let label = if f >= 1000.0 { format!("{name} {:.1}k", f / 1000.0) } else { format!("{name} {:.0}", f) };
                                let al = if name == "HP" { Align2::LEFT_CENTER } else { Align2::RIGHT_CENTER };
                                let dx = if name == "HP" { 7.0 } else { -7.0 };
                                gclip.text(Pos2::new(x + dx, y - 12.0), al, label, egui::FontId::monospace(9.0), accent);
                            }
                        }
                        if split {
                            gclip.text(Pos2::new(g.right() - 8.0, g.bottom() - 22.0), Align2::RIGHT_CENTER, "FILTERS: SIDECHAIN ONLY", egui::FontId::monospace(9.0), Color32::from_rgb(110, 112, 116));
                        }
                    }
                    // total: EQ (+ filters when they are in the audio path)
                    let pts: Vec<Pos2> = (0..=n)
                        .map(|i| {
                            let f = fr(i);
                            let mut d = if eq_on { cf.response_db(fs, f) } else { 0.0 };
                            if !split {
                                d += filt(f);
                            }
                            Pos2::new(fxf(f), dyf(d))
                        })
                        .collect();
                    gclip.add(egui::Shape::line(pts, PathStroke::new(2.2_f32, if eq_on || hp_on || lp_on { accent } else { Color32::from_rgb(110, 112, 116) })));
                } else {
                    // dynamics: output level against input level, -60..0 dB, over the spectrum area
                    
                    let sq = g.height();
                    let d = Rect::from_center_size(g.center() + Vec2::new(10.0, 4.0), Vec2::new(sq * 1.55, sq - 44.0));
                    let (lo, hi) = (-60.0f32, 0.0f32);
                    let dx = |v: f32| d.left() + d.width() * ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
                    let dyy = |v: f32| d.bottom() - d.height() * ((v - lo) / (hi - lo)).clamp(-0.02, 1.02);
                    for v in [-48.0f32, -36.0, -24.0, -12.0] {
                        gclip.vline(dx(v), d.top()..=d.bottom(), Stroke::new(1.0_f32, Color32::from_rgb(30, 32, 36)));
                        gclip.hline(d.left()..=d.right(), dyy(v), Stroke::new(1.0_f32, Color32::from_rgb(30, 32, 36)));
                        gclip.text(Pos2::new(dx(v), d.bottom() + 8.0), Align2::CENTER_CENTER, format!("{v:.0}"), egui::FontId::monospace(8.5), Color32::from_rgb(110, 112, 116));
                        gclip.text(Pos2::new(d.left() - 4.0, dyy(v)), Align2::RIGHT_CENTER, format!("{v:.0}"), egui::FontId::monospace(8.5), Color32::from_rgb(110, 112, 116));
                    }
                    gclip.rect_stroke(d, 0.0, Stroke::new(1.0_f32, Color32::from_rgb(30, 32, 36)), egui::StrokeKind::Inside);
                    gclip.line_segment([Pos2::new(dx(lo), dyy(lo)), Pos2::new(dx(hi), dyy(hi))], Stroke::new(1.0_f32, Color32::from_rgb(120, 122, 126)));
                    let (x_on, c_on) = (p.x_on.value() && on, p.c_on.value() && on);
                    let tf = |v: f32| crate::dsp::dyn_transfer_db(v, x_on, p.x_gate.value(), p.x_thr.value(), p.x_range.value(), c_on, p.c_thr.value(), p.c_ratio.value(), p.c_auto.value());
                    if x_on {
                        let x = dx(p.x_thr.value());
                        gclip.vline(x, d.top()..=d.bottom(), Stroke::new(1.0_f32, C_GRN.gamma_multiply(0.6)));
                        gclip.text(Pos2::new(x + 4.0, d.bottom() - 10.0), Align2::LEFT_CENTER, if p.x_gate.value() { "GATE" } else { "EXP" }, egui::FontId::monospace(9.0), C_GRN);
                    }
                    if c_on {
                        let x = dx(p.c_thr.value());
                        gclip.vline(x, d.top()..=d.bottom(), Stroke::new(1.0_f32, accent.gamma_multiply(0.6)));
                        gclip.text(Pos2::new(x - 4.0, d.bottom() - 10.0), Align2::RIGHT_CENTER, "COMP", egui::FontId::monospace(9.0), accent);
                    }
                    let n = 160;
                    let pts: Vec<Pos2> = (0..=n).map(|i| { let v = lo + (hi - lo) * i as f32 / n as f32; Pos2::new(dx(v), dyy(tf(v))) }).collect();
                    gclip.add(egui::Shape::line(pts, PathStroke::new(2.2_f32, accent)));
                    // live level on the curve
                    let lvl = cur[0];
                    if lvl > lo {
                        let pnt = Pos2::new(dx(lvl), dyy(tf(lvl)));
                        gclip.circle_filled(pnt, 6.0, Color32::from_white_alpha(40));
                        gclip.circle_filled(pnt, 3.5, accent);
                    }
                    gclip.text(Pos2::new(d.right(), d.bottom() + 8.0), Align2::RIGHT_CENTER, "IN dB", egui::FontId::monospace(8.5), Color32::from_rgb(110, 112, 116));
                    gclip.text(Pos2::new(d.left() - 4.0, d.top() + 2.0), Align2::RIGHT_CENTER, "OUT", egui::FontId::monospace(8.5), Color32::from_rgb(110, 112, 116));
                }
                hw::glass(&painter, hwr, scr);
                if hw::button(ui, hwr, b("v_eq"), !st.dyn_view, "EQ curve with the HP / LP filters.") {
                    st.dyn_view = false;
                }
                if hw::button(ui, hwr, b("v_dyn"), st.dyn_view, "Expander / gate and compressor curve with the live level.") {
                    st.dyn_view = true;
                }
                tg(ui, "eq_on", &p.eq_on, "Switch the EQ section in or out.");
                for (id, v, tip) in [("brown", Voice::Brown, "BROWN: the softer, broader EQ voicing."), ("black", Voice::Black, "BLACK: the sharper, more surgical EQ voicing.")] {
                    if hw::button(ui, hwr, b(id), p.voice.value() == v, tip) {
                        set_e(setter, &p.voice, v);
                    }
                }
                // ---- EQ bands
                hw::knob(ui, hwr, setter, &p.lf_g, k("lf_g"), C_BRN, "Low band gain.");
                hw::knob(ui, hwr, setter, &p.lf_f, k("lf_f"), C_BRN, "Low band frequency.");
                tg(ui, "lf_bell", &p.lf_bell, "Bell instead of shelf.");
                hw::knob(ui, hwr, setter, &p.lmf_g, k("lmf_g"), C_BLU, "Low-mid gain.");
                hw::knob(ui, hwr, setter, &p.lmf_f, k("lmf_f"), C_BLU, "Low-mid frequency.");
                hw::knob(ui, hwr, setter, &p.lmf_q, k("lmf_q"), C_BLU, "Low-mid bandwidth.");
                hw::knob(ui, hwr, setter, &p.hmf_g, k("hmf_g"), C_GRN, "High-mid gain.");
                hw::knob(ui, hwr, setter, &p.hmf_f, k("hmf_f"), C_GRN, "High-mid frequency.");
                hw::knob(ui, hwr, setter, &p.hmf_q, k("hmf_q"), C_GRN, "High-mid bandwidth.");
                hw::knob(ui, hwr, setter, &p.hf_g, k("hf_g"), C_RED, "High band gain.");
                hw::knob(ui, hwr, setter, &p.hf_f, k("hf_f"), C_RED, "High band frequency.");
                tg(ui, "hf_bell", &p.hf_bell, "Bell instead of shelf.");
                // ---- meters + output
                let lv = |d: f32| ((d + 48.0) / 48.0).clamp(0.0, 1.0);
                hw::ladder(&painter, hwr, hw::find(lay::LADDERS, "in", |x| x.id), lv(st.lv[0]), false);
                hw::ladder(&painter, hwr, hw::find(lay::LADDERS, "out", |x| x.id), lv(st.lv[1]), false);
                hw::ladder_hold(&painter, hwr, hw::find(lay::LADDERS, "in", |x| x.id), lv(st.hold[0]));
                hw::ladder_hold(&painter, hwr, hw::find(lay::LADDERS, "out", |x| x.id), lv(st.hold[1]));
                hw::ladder(&painter, hwr, hw::find(lay::LADDERS, "exp", |x| x.id), (st.gr[0] / 20.0).clamp(0.0, 1.0), true);
                hw::ladder(&painter, hwr, hw::find(lay::LADDERS, "comp", |x| x.id), (st.gr[1] / 20.0).clamp(0.0, 1.0), true);
                for (i, id) in ["gr_exp", "gr_comp"].into_iter().enumerate() {
                    let t = if st.gr[i] < 0.05 { "0.0".to_string() } else { format!("-{:.1}", st.gr[i]) };
                    hw::text(&painter, hwr, hw::find(lay::TEXTS, id, |x| x.id), &t);
                }
                hw::knob(ui, hwr, setter, &p.out, k("out"), C_WHT, "Output fader in dB.");
                hw::knob(ui, hwr, setter, &p.width, k("width"), C_WHT, "Stereo width: 0 = mono, 100 = unchanged, 200 = double.");
                tg(ui, "mono", &p.mono, "Sum the output to mono.");
                let _ = (Rect::NOTHING, Vec2::ZERO, Align2::CENTER_CENTER);
                resize_grip(ui, ctx, &size_state, full, BASE_W, BASE_H, 760.0);
            });
        },
    )
}
