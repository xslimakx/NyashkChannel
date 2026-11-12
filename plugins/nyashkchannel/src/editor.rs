//! NyashkChannel GUI.
use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, epaint::PathStroke, Align2, Pos2, Rect, Stroke, Vec2, Color32},
};
use nyashk_common::eq::EqCoefs;
use nyashk_common::ui::*;
use std::sync::Arc;
use std::time::Duration;

use crate::*;

const BASE_W: f32 = 1000.0;
const BASE_H: f32 = 600.0;

struct Gui {
    spec: nyashk_common::analyzer::Spectrum,
    lv: [f32; 4],
    hold: [f32; 4],
    gr: [f32; 2],
    /// graph view: false = EQ + filters, true = dynamics curve
    dyn_view: bool,
}

pub fn create(params: Arc<ChannelParams>, meters: Arc<Meters>) -> Option<Box<dyn Editor>> {
    let editor_state = params.editor_state.clone();
    let size_state = editor_state.clone();
    create_egui_editor(
        editor_state,
        Gui { spec: nyashk_common::analyzer::Spectrum::new(4096), lv: [-90.0; 4], hold: [-90.0; 4], gr: [0.0; 2], dyn_view: false },
        |ctx, _| apply_theme(ctx),
        move |ctx, setter, st| {
            ctx.request_repaint_after(Duration::from_millis(33));
            let p = &*params;
            let fs = f32::from_bits(meters.fs.load(std::sync::atomic::Ordering::Relaxed)).max(8000.0);
            st.spec.update(&meters.tap);
            let cur = [
                meters.get(M_IN_L).max(meters.get(M_IN_R)),
                meters.get(M_OUT_L).max(meters.get(M_OUT_R)),
            ];
            for i in 0..2 {
                st.lv[i] = if cur[i] > st.lv[i] { cur[i] } else { st.lv[i] - 1.2 };
                st.hold[i] = st.hold[i].max(st.lv[i]) - 0.12;
            }
            for (i, m) in [M_GR_X, M_GR_C].into_iter().enumerate() {
                let t = meters.get(m);
                st.gr[i] = if t > st.gr[i] { t } else { st.gr[i] - 0.6 }.max(0.0);
            }

            egui::CentralPanel::default().frame(egui::Frame::NONE.fill(BG)).show(ctx, |ui| {
                let full = ui.max_rect();
                let o = full.min;
                let rect = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(Pos2::new(o.x + x, o.y + y), Vec2::new(w, h));
                header(ui, o, BASE_W, "Channel", "CHANNEL STRIP");
                let on = p.power.value();
                if pill_tip(ui, rect(BASE_W - 14.0 - 76.0, 22.0, 76.0, 24.0), "POWER", on, "Switch the whole strip on or off (smooth, click-free).") {
                    set_b(setter, &p.power, !on);
                }
                let painter = ui.painter().clone();
                let r = 21.0;
                let ch = 80.0;
                let cw = 66.0;
                let title = |t: &str, x0: f32, y0: f32, w: f32| {
                    painter.text(Pos2::new(o.x + x0 + w * 0.5, o.y + y0 + 13.0), Align2::CENTER_CENTER, t, mono(10.0), WHITE);
                    painter.hline(o.x + x0 + 4.0..=o.x + x0 + w - 4.0, o.y + y0 + 23.0, Stroke::new(1.0, LINE));
                };
                let tg = |ui: &mut egui::Ui, x: f32, y: f32, w: f32, t: &str, v: bool, b: &BoolParam, tip: &str| {
                    if pill_tip(ui, rect(x, y, w, 22.0), t, v, tip) {
                        set_b(setter, b, !v);
                    }
                };

                // ================= left column
                let lx = 14.0;
                let lw = 300.0;
                // input
                panel(ui, rect(lx, 66.0, lw, 100.0));
                title("INPUT", lx, 66.0, lw);
                tg(ui, lx + 10.0, 98.0, 44.0, "Ø", p.phase.value(), &p.phase, "Invert the polarity.");
                tg(ui, lx + 10.0, 124.0, 44.0, "PAD", p.pad.value(), &p.pad, "-20 dB pad for hot signals.");
                knob_r(ui, setter, &p.line, "LINE", rect(lx + 64.0, 92.0, cw, 72.0), "Line trim, +/- 20 dB.", true, r - 3.0);
                knob_r(ui, setter, &p.mic, "MIC", rect(lx + 64.0 + cw, 92.0, cw, 72.0), "Mic preamp gain, 0 to 50 dB.", false, r - 3.0);
                tg(ui, lx + 64.0 + cw * 2.0 + 6.0, 98.0, 66.0, "ANALOG", p.analog.value(), &p.analog, "Adds gentle transformer-style saturation to the input stage.");

                // filters
                panel(ui, rect(lx, 172.0, lw, 130.0));
                title("FILTERS", lx, 172.0, lw);
                tg(ui, lx + 10.0, 202.0, 44.0, "HP", p.hp_on.value(), &p.hp_on, "High-pass filter (18 dB/oct).");
                if knob_r(ui, setter, &p.hp_f, "HP FREQ", rect(lx + 60.0, 196.0, cw, 72.0), "High-pass frequency.", false, r - 3.0).total.is_some() {
                    st.dyn_view = false;
                }
                tg(ui, lx + 140.0, 202.0, 44.0, "LP", p.lp_on.value(), &p.lp_on, "Low-pass filter (12 dB/oct).");
                if knob_r(ui, setter, &p.lp_f, "LP FREQ", rect(lx + 190.0, 196.0, cw, 72.0), "Low-pass frequency.", false, r - 3.0).total.is_some() {
                    st.dyn_view = false;
                }
                tg(ui, lx + 10.0, 277.0, 100.0, "SPLIT", p.split.value(), &p.split, "Filters act on the dynamics sidechain only, not on the audio.");

                // expander / gate
                panel(ui, rect(lx, 306.0, lw, 112.0));
                title("EXPANDER / GATE", lx, 306.0, lw);
                tg(ui, lx + 10.0, 336.0, 44.0, "ON", p.x_on.value(), &p.x_on, "Switch the expander on.");
                tg(ui, lx + 10.0, 362.0, 44.0, "GATE", p.x_gate.value(), &p.x_gate, "Hard gate instead of a soft expander.");
                tg(ui, lx + 10.0, 388.0, 44.0, "FAST", p.x_fast.value(), &p.x_fast, "Fast attack.");
                if knob_ac(ui, setter, &p.x_thr, "THRESH", rect(lx + 60.0, 332.0, cw, ch), "Level below which the signal is reduced.", false, r, C_GATE).total.is_some() {
                    st.dyn_view = true;
                }
                if knob_ac(ui, setter, &p.x_range, "RANGE", rect(lx + 60.0 + cw, 332.0, cw, ch), "Maximum reduction in dB.", false, r, C_GATE).total.is_some() {
                    st.dyn_view = true;
                }
                knob_ac(ui, setter, &p.x_rel, "RELEASE", rect(lx + 60.0 + cw * 2.0, 332.0, cw, ch), "Release time.", false, r, C_GATE);

                // compressor
                panel(ui, rect(lx, 424.0, lw, 162.0));
                title("COMPRESSOR", lx, 424.0, lw);
                tg(ui, lx + 10.0, 454.0, 44.0, "ON", p.c_on.value(), &p.c_on, "Switch the compressor on.");
                tg(ui, lx + 10.0, 480.0, 44.0, "FAST", p.c_fast.value(), &p.c_fast, "Fast attack.");
                tg(ui, lx + 10.0, 506.0, 44.0, "AUTO", p.c_auto.value(), &p.c_auto, "Automatic makeup gain.");
                if knob_ac(ui, setter, &p.c_thr, "THRESH", rect(lx + 60.0, 450.0, cw, ch), "Level above which compression starts.", false, r, C_DYN).total.is_some() {
                    st.dyn_view = true;
                }
                if knob_ac(ui, setter, &p.c_ratio, "RATIO", rect(lx + 60.0 + cw, 450.0, cw, ch), "Compression ratio.", false, r, C_DYN).total.is_some() {
                    st.dyn_view = true;
                }
                knob_ac(ui, setter, &p.c_rel, "RELEASE", rect(lx + 60.0 + cw * 2.0, 450.0, cw, ch), "Release time.", false, r, C_DYN);
                let dp = p.dyn_post.value();
                tg(ui, lx + 10.0, 536.0, lw - 20.0, if dp { "DYNAMICS AFTER EQ" } else { "DYNAMICS BEFORE EQ" }, dp, &p.dyn_post, "Choose whether the dynamics come before or after the EQ.");

                // ================= centre: EQ
                let cx = 328.0;
                let cwid = 440.0;
                panel(ui, rect(cx, 66.0, cwid, 236.0));
                let g = rect(cx + 34.0, 82.0, cwid - 50.0, 192.0);
                painter.rect_filled(g, 4.0, Color32::from_rgb(0x07, 0x07, 0x07));
                let (f_lo, f_hi, d_r) = (20.0f32, 20000.0f32, 18.0f32);
                let fxf = |f: f32| g.left() + g.width() * (f / f_lo).ln() / (f_hi / f_lo).ln();
                let dyf = |d: f32| g.center().y - g.height() * 0.5 * (d / d_r).clamp(-1.0, 1.0);
                if !st.dyn_view {
                    for d in [-12.0f32, -6.0, 0.0, 6.0, 12.0] {
                        painter.hline(g.left()..=g.right(), dyf(d), Stroke::new(1.0, if d == 0.0 { MID } else { LINE }));
                        painter.text(Pos2::new(g.left() - 5.0, dyf(d)), Align2::RIGHT_CENTER, format!("{:+.0}", d), mono(9.0), DIM);
                    }
                    for f in [50.0f32, 100.0, 500.0, 1000.0, 5000.0, 10000.0] {
                        painter.vline(fxf(f), g.top()..=g.bottom(), Stroke::new(1.0, LINE));
                        painter.text(Pos2::new(fxf(f), g.bottom() - 8.0), Align2::CENTER_CENTER, if f >= 1000.0 { format!("{}k", f / 1000.0) } else { format!("{}", f) }, mono(9.0), DIM);
                    }
                    // live output spectrum behind the curve (tilted so pink noise looks flat)
                    let spy = |d: f32| g.bottom() - g.height() * ((d + 90.0) / 90.0).clamp(-0.05, 1.05);
                    let sp = st.spec.points(fs, f_lo, f_hi.min(fs * 0.49), 180, 4.5, fxf, spy);
                    nyashk_common::analyzer::draw_spectrum(&painter, g, &sp, Color32::from_gray(26), Color32::from_gray(72));
                    for d in [-12.0f32, -6.0, 6.0, 12.0] {
                        painter.with_clip_rect(g).hline(g.left()..=g.right(), dyf(d), Stroke::new(1.0, Color32::from_white_alpha(6)));
                    }
                    for f in [50.0f32, 100.0, 500.0, 1000.0, 5000.0, 10000.0] {
                        painter.with_clip_rect(g).vline(fxf(f), g.top()..=g.bottom(), Stroke::new(1.0, Color32::from_white_alpha(5)));
                        painter.text(Pos2::new(fxf(f), g.bottom() - 8.0), Align2::CENTER_CENTER, if f >= 1000.0 { format!("{}k", f / 1000.0) } else { format!("{}", f) }, mono(9.0), DIM);
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
                                gclip.add(egui::Shape::line(w.to_vec(), PathStroke::new(1.5, MID)));
                            }
                        } else {
                            nyashk_common::ui::fill_under(&gclip, &fp, dyf(0.0), Color32::from_white_alpha(10));
                            gclip.add(egui::Shape::line(fp, PathStroke::new(1.2, MID)));
                        }
                        for (on_f, f, name) in [(hp_on, p.hp_f.value(), "HP"), (lp_on, p.lp_f.value(), "LP")] {
                            if on_f {
                                let x = fxf(f.clamp(f_lo, f_hi));
                                let y = dyf(filt(f)).min(g.bottom() - 30.0);
                                gclip.circle_filled(Pos2::new(x, y), 3.5, WHITE);
                                let label = if f >= 1000.0 { format!("{name} {:.1}k", f / 1000.0) } else { format!("{name} {:.0}", f) };
                                let al = if name == "HP" { Align2::LEFT_CENTER } else { Align2::RIGHT_CENTER };
                                let dx = if name == "HP" { 7.0 } else { -7.0 };
                                gclip.text(Pos2::new(x + dx, y - 12.0), al, label, mono(9.0), WHITE);
                            }
                        }
                        if split {
                            gclip.text(Pos2::new(g.right() - 8.0, g.bottom() - 22.0), Align2::RIGHT_CENTER, "FILTERS: SIDECHAIN ONLY", mono(9.0), DIM);
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
                    gclip.add(egui::Shape::line(pts, PathStroke::new(2.2, if eq_on || hp_on || lp_on { WHITE } else { DIM })));
                } else {
                    // dynamics: output level against input level, -60..0 dB, over the spectrum area
                    painter.rect_filled(g, 4.0, Color32::from_rgb(0x07, 0x07, 0x07));
                    let sq = g.height();
                    let d = Rect::from_center_size(g.center() + Vec2::new(10.0, 4.0), Vec2::new(sq * 1.55, sq - 44.0));
                    let (lo, hi) = (-60.0f32, 0.0f32);
                    let dx = |v: f32| d.left() + d.width() * ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
                    let dyy = |v: f32| d.bottom() - d.height() * ((v - lo) / (hi - lo)).clamp(-0.02, 1.02);
                    for v in [-48.0f32, -36.0, -24.0, -12.0] {
                        gclip.vline(dx(v), d.top()..=d.bottom(), Stroke::new(1.0, LINE));
                        gclip.hline(d.left()..=d.right(), dyy(v), Stroke::new(1.0, LINE));
                        gclip.text(Pos2::new(dx(v), d.bottom() + 8.0), Align2::CENTER_CENTER, format!("{v:.0}"), mono(8.5), DIM);
                        gclip.text(Pos2::new(d.left() - 4.0, dyy(v)), Align2::RIGHT_CENTER, format!("{v:.0}"), mono(8.5), DIM);
                    }
                    gclip.rect_stroke(d, 0.0, Stroke::new(1.0, LINE), egui::StrokeKind::Inside);
                    gclip.line_segment([Pos2::new(dx(lo), dyy(lo)), Pos2::new(dx(hi), dyy(hi))], Stroke::new(1.0, MID));
                    let (x_on, c_on) = (p.x_on.value() && on, p.c_on.value() && on);
                    let tf = |v: f32| crate::dsp::dyn_transfer_db(v, x_on, p.x_gate.value(), p.x_thr.value(), p.x_range.value(), c_on, p.c_thr.value(), p.c_ratio.value(), p.c_auto.value());
                    if x_on {
                        let x = dx(p.x_thr.value());
                        gclip.vline(x, d.top()..=d.bottom(), Stroke::new(1.0, C_GATE.gamma_multiply(0.6)));
                        gclip.text(Pos2::new(x + 4.0, d.bottom() - 10.0), Align2::LEFT_CENTER, if p.x_gate.value() { "GATE" } else { "EXP" }, mono(9.0), C_GATE);
                    }
                    if c_on {
                        let x = dx(p.c_thr.value());
                        gclip.vline(x, d.top()..=d.bottom(), Stroke::new(1.0, C_DYN.gamma_multiply(0.6)));
                        gclip.text(Pos2::new(x - 4.0, d.bottom() - 10.0), Align2::RIGHT_CENTER, "COMP", mono(9.0), C_DYN);
                    }
                    let n = 160;
                    let pts: Vec<Pos2> = (0..=n).map(|i| { let v = lo + (hi - lo) * i as f32 / n as f32; Pos2::new(dx(v), dyy(tf(v))) }).collect();
                    gclip.add(egui::Shape::line(pts, PathStroke::new(2.2, WHITE)));
                    // live level on the curve
                    let lvl = cur[0];
                    if lvl > lo {
                        let pnt = Pos2::new(dx(lvl), dyy(tf(lvl)));
                        gclip.circle_filled(pnt, 6.0, Color32::from_white_alpha(40));
                        gclip.circle_filled(pnt, 3.5, WHITE);
                    }
                    gclip.text(Pos2::new(d.right(), d.bottom() + 8.0), Align2::RIGHT_CENTER, "IN dB", mono(8.5), DIM);
                    gclip.text(Pos2::new(d.left() - 4.0, d.top() + 2.0), Align2::RIGHT_CENTER, "OUT", mono(8.5), DIM);
                }
                // view tabs
                if pill_tip(ui, rect(cx + 40.0, 86.0, 40.0, 20.0), "EQ", !st.dyn_view, "EQ curve with the HP / LP filters.") {
                    st.dyn_view = false;
                }
                if pill_tip(ui, rect(cx + 84.0, 86.0, 44.0, 20.0), "DYN", st.dyn_view, "Expander / gate and compressor curve: output level against input level, with the live level.") {
                    st.dyn_view = true;
                }
                tg(ui, cx + cwid - 150.0, 84.0, 60.0, "EQ IN", p.eq_on.value(), &p.eq_on, "Switch the EQ section in or out.");
                enum_combo(ui, setter, &p.voice, "ch_voice", rect(cx + cwid - 84.0, 84.0, 70.0, 22.0));

                panel(ui, rect(cx, 308.0, cwid, 278.0));
                let bw = cwid / 4.0;
                let bands: [(&str, f32); 4] = [("LF", 0.0), ("LMF", 1.0), ("HMF", 2.0), ("HF", 3.0)];
                for (name, i) in bands {
                    let x0 = cx + bw * i;
                    title(name, x0, 308.0, bw);
                    let cell = |row: f32| rect(x0 + (bw - cw) * 0.5, 336.0 + row * 82.0, cw, ch);
                    match name {
                        "LF" => {
                            knob_ac(ui, setter, &p.lf_g, "GAIN", cell(0.0), "Low band gain.", true, r, C_LF);
                            knob_ac(ui, setter, &p.lf_f, "FREQ", cell(1.0), "Low band frequency.", false, r, C_LF);
                            tg(ui, x0 + (bw - 60.0) * 0.5, 336.0 + 2.0 * 82.0 + 14.0, 60.0, "BELL", p.lf_bell.value(), &p.lf_bell, "Bell instead of shelf.");
                        }
                        "LMF" => {
                            knob_ac(ui, setter, &p.lmf_g, "GAIN", cell(0.0), "Low-mid gain.", true, r, C_LMF);
                            knob_ac(ui, setter, &p.lmf_f, "FREQ", cell(1.0), "Low-mid frequency.", false, r, C_LMF);
                            knob_ac(ui, setter, &p.lmf_q, "Q", cell(2.0), "Low-mid bandwidth.", false, r, C_LMF);
                        }
                        "HMF" => {
                            knob_ac(ui, setter, &p.hmf_g, "GAIN", cell(0.0), "High-mid gain.", true, r, C_HMF);
                            knob_ac(ui, setter, &p.hmf_f, "FREQ", cell(1.0), "High-mid frequency.", false, r, C_HMF);
                            knob_ac(ui, setter, &p.hmf_q, "Q", cell(2.0), "High-mid bandwidth.", false, r, C_HMF);
                        }
                        _ => {
                            knob_ac(ui, setter, &p.hf_g, "GAIN", cell(0.0), "High band gain.", true, r, C_HF);
                            knob_ac(ui, setter, &p.hf_f, "FREQ", cell(1.0), "High band frequency.", false, r, C_HF);
                            tg(ui, x0 + (bw - 60.0) * 0.5, 336.0 + 2.0 * 82.0 + 14.0, 60.0, "BELL", p.hf_bell.value(), &p.hf_bell, "Bell instead of shelf.");
                        }
                    }
                }

                // ================= right: meters + output
                let rx = 782.0;
                let rw = BASE_W - rx - 14.0;
                panel(ui, rect(rx, 66.0, rw, 520.0));
                title("METERS", rx, 66.0, rw);
                let bt = o.y + 104.0;
                let bh = 220.0;
                let mut bx = o.x + rx + 22.0;
                for (i, name) in ["IN", "OUT"].into_iter().enumerate() {
                    let rr = Rect::from_min_size(Pos2::new(bx, bt), Vec2::new(14.0, bh));
                    vbar(ui, rr, st.lv[i], st.hold[i], -48.0, 0.0);
                    painter.text(Pos2::new(rr.center().x, rr.bottom() + 11.0), Align2::CENTER_CENTER, name, mono(9.5), DIM);
                    bx += 36.0;
                }
                bx += 8.0;
                for (i, name) in ["EXP", "COMP"].into_iter().enumerate() {
                    let rr = Rect::from_min_size(Pos2::new(bx, bt), Vec2::new(14.0, bh));
                    vbar(ui, rr, st.gr[i], st.gr[i], 0.0, 20.0);
                    painter.text(Pos2::new(rr.center().x, rr.bottom() + 11.0), Align2::CENTER_CENTER, name, mono(9.5), DIM);
                    painter.text(Pos2::new(rr.center().x, rr.bottom() + 24.0), Align2::CENTER_CENTER, if st.gr[i] < 0.05 { "0.0".to_string() } else { format!("-{:.1}", st.gr[i]) }, mono(9.0), WHITE);
                    bx += 40.0;
                }
                title("OUTPUT", rx, 356.0, rw);
                knob_r(ui, setter, &p.out, "OUTPUT", rect(rx + 8.0, 384.0, cw, ch), "Output fader in dB.", true, r);
                knob_r(ui, setter, &p.width, "WIDTH", rect(rx + 8.0 + cw + 6.0, 384.0, cw, ch), "Stereo width: 0 = mono, 100 = unchanged, 200 = double.", false, r);
                tg(ui, rx + 14.0, 480.0, rw - 28.0, "MONO", p.mono.value(), &p.mono, "Sum the output to mono.");

                resize_grip(ui, ctx, &size_state, full, BASE_W, BASE_H, 760.0);
            });
        },
    )
}
