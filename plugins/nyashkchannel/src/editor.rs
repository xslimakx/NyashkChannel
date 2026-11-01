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
    lv: [f32; 4],
    hold: [f32; 4],
    gr: [f32; 2],
}

pub fn create(params: Arc<ChannelParams>, meters: Arc<Meters>) -> Option<Box<dyn Editor>> {
    let editor_state = params.editor_state.clone();
    let size_state = editor_state.clone();
    create_egui_editor(
        editor_state,
        Gui { lv: [-90.0; 4], hold: [-90.0; 4], gr: [0.0; 2] },
        |ctx, _| apply_theme(ctx),
        move |ctx, setter, st| {
            ctx.request_repaint_after(Duration::from_millis(33));
            let p = &*params;
            let fs = f32::from_bits(meters.fs.load(std::sync::atomic::Ordering::Relaxed)).max(8000.0);
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
                knob_r(ui, setter, &p.hp_f, "HP FREQ", rect(lx + 60.0, 196.0, cw, 72.0), "High-pass frequency.", false, r - 3.0);
                tg(ui, lx + 140.0, 202.0, 44.0, "LP", p.lp_on.value(), &p.lp_on, "Low-pass filter (12 dB/oct).");
                knob_r(ui, setter, &p.lp_f, "LP FREQ", rect(lx + 190.0, 196.0, cw, 72.0), "Low-pass frequency.", false, r - 3.0);
                tg(ui, lx + 10.0, 272.0, 100.0, "SPLIT", p.split.value(), &p.split, "Filters act on the dynamics sidechain only, not on the audio.");

                // expander / gate
                panel(ui, rect(lx, 306.0, lw, 112.0));
                title("EXPANDER / GATE", lx, 306.0, lw);
                tg(ui, lx + 10.0, 336.0, 44.0, "ON", p.x_on.value(), &p.x_on, "Switch the expander on.");
                tg(ui, lx + 10.0, 362.0, 44.0, "GATE", p.x_gate.value(), &p.x_gate, "Hard gate instead of a soft expander.");
                tg(ui, lx + 10.0, 388.0, 44.0, "FAST", p.x_fast.value(), &p.x_fast, "Fast attack.");
                knob_r(ui, setter, &p.x_thr, "THRESH", rect(lx + 60.0, 332.0, cw, ch), "Level below which the signal is reduced.", false, r);
                knob_r(ui, setter, &p.x_range, "RANGE", rect(lx + 60.0 + cw, 332.0, cw, ch), "Maximum reduction in dB.", false, r);
                knob_r(ui, setter, &p.x_rel, "RELEASE", rect(lx + 60.0 + cw * 2.0, 332.0, cw, ch), "Release time.", false, r);

                // compressor
                panel(ui, rect(lx, 424.0, lw, 162.0));
                title("COMPRESSOR", lx, 424.0, lw);
                tg(ui, lx + 10.0, 454.0, 44.0, "ON", p.c_on.value(), &p.c_on, "Switch the compressor on.");
                tg(ui, lx + 10.0, 480.0, 44.0, "FAST", p.c_fast.value(), &p.c_fast, "Fast attack.");
                tg(ui, lx + 10.0, 506.0, 44.0, "AUTO", p.c_auto.value(), &p.c_auto, "Automatic makeup gain.");
                knob_r(ui, setter, &p.c_thr, "THRESH", rect(lx + 60.0, 450.0, cw, ch), "Level above which compression starts.", false, r);
                knob_r(ui, setter, &p.c_ratio, "RATIO", rect(lx + 60.0 + cw, 450.0, cw, ch), "Compression ratio.", false, r);
                knob_r(ui, setter, &p.c_rel, "RELEASE", rect(lx + 60.0 + cw * 2.0, 450.0, cw, ch), "Release time.", false, r);
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
                for d in [-12.0f32, -6.0, 0.0, 6.0, 12.0] {
                    painter.hline(g.left()..=g.right(), dyf(d), Stroke::new(1.0, if d == 0.0 { MID } else { LINE }));
                    painter.text(Pos2::new(g.left() - 5.0, dyf(d)), Align2::RIGHT_CENTER, format!("{:+.0}", d), mono(9.0), DIM);
                }
                for f in [50.0f32, 100.0, 500.0, 1000.0, 5000.0, 10000.0] {
                    painter.vline(fxf(f), g.top()..=g.bottom(), Stroke::new(1.0, LINE));
                    painter.text(Pos2::new(fxf(f), g.bottom() - 8.0), Align2::CENTER_CENTER, if f >= 1000.0 { format!("{}k", f / 1000.0) } else { format!("{}", f) }, mono(9.0), DIM);
                }
                let eq_on = p.eq_on.value() && on;
                let cf = EqCoefs::new(fs, &p.eq_cfg());
                let pts: Vec<Pos2> = (0..=240)
                    .map(|i| {
                        let f = f_lo * (f_hi / f_lo).powf(i as f32 / 240.0);
                        Pos2::new(fxf(f), dyf(if eq_on { cf.response_db(fs, f) } else { 0.0 }))
                    })
                    .collect();
                painter.with_clip_rect(g).add(egui::Shape::line(pts, PathStroke::new(2.2, if eq_on { WHITE } else { DIM })));
                painter.text(Pos2::new(g.left() + 8.0, g.top() + 10.0), Align2::LEFT_CENTER, "EQ", mono(9.0), DIM);
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
                            knob_r(ui, setter, &p.lf_g, "GAIN", cell(0.0), "Low band gain.", true, r);
                            knob_r(ui, setter, &p.lf_f, "FREQ", cell(1.0), "Low band frequency.", false, r);
                            tg(ui, x0 + (bw - 60.0) * 0.5, 336.0 + 2.0 * 82.0 + 14.0, 60.0, "BELL", p.lf_bell.value(), &p.lf_bell, "Bell instead of shelf.");
                        }
                        "LMF" => {
                            knob_r(ui, setter, &p.lmf_g, "GAIN", cell(0.0), "Low-mid gain.", true, r);
                            knob_r(ui, setter, &p.lmf_f, "FREQ", cell(1.0), "Low-mid frequency.", false, r);
                            knob_r(ui, setter, &p.lmf_q, "Q", cell(2.0), "Low-mid bandwidth.", false, r);
                        }
                        "HMF" => {
                            knob_r(ui, setter, &p.hmf_g, "GAIN", cell(0.0), "High-mid gain.", true, r);
                            knob_r(ui, setter, &p.hmf_f, "FREQ", cell(1.0), "High-mid frequency.", false, r);
                            knob_r(ui, setter, &p.hmf_q, "Q", cell(2.0), "High-mid bandwidth.", false, r);
                        }
                        _ => {
                            knob_r(ui, setter, &p.hf_g, "GAIN", cell(0.0), "High band gain.", true, r);
                            knob_r(ui, setter, &p.hf_f, "FREQ", cell(1.0), "High band frequency.", false, r);
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
                    painter.text(Pos2::new(rr.center().x, rr.bottom() + 24.0), Align2::CENTER_CENTER, format!("-{:.1}", st.gr[i]), mono(9.0), WHITE);
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
