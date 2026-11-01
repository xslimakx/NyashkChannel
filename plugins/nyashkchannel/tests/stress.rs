use nyashkchannel::dsp::*;

fn extreme() -> Params {
    let mut p = Params::default();
    p.mic_db = 50.0;
    p.line_db = 20.0;
    p.analog = true;
    p.hp_on = true;
    p.hp_f = 350.0;
    p.lp_on = true;
    p.lp_f = 3000.0;
    p.x_on = true;
    p.x_gate = true;
    p.x_thr = 0.0;
    p.x_range = 40.0;
    p.c_on = true;
    p.c_ratio = 20.0;
    p.c_thr = -40.0;
    p.c_fast = true;
    p.eq.lf.gain_db = 16.0;
    p.eq.lmf.gain_db = -16.0;
    p.eq.lmf.q = 8.0;
    p.eq.hmf.gain_db = 16.0;
    p.eq.hmf.q = 8.0;
    p.eq.hf.gain_db = 16.0;
    p.width = 2.0;
    p.out_db = 12.0;
    p
}

#[test]
fn every_sample_rate_and_extreme_setting_stays_finite() {
    for &fs in &[8000.0f32, 22050.0, 44100.0, 48000.0, 96000.0, 192000.0] {
        for (name, p) in [("default", Params::default()), ("extreme", extreme())] {
            for split in [false, true] {
                for post in [false, true] {
                    let mut p = Params { split, dyn_post: post, ..p };
                    p.power = true;
                    let mut s = Strip::new(fs);
                    let mut rng = 3u32;
                    for n in 0..(fs as usize) {
                        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                        let w = (rng >> 8) as f32 / 8388608.0 - 1.0;
                        let v = match (n / 4000) % 4 {
                            0 => w,
                            1 => 0.0,
                            2 => if (n / 20) % 2 == 0 { 1.0 } else { -1.0 },
                            _ => 1.0,
                        };
                        let mut x = [v, -v * 0.7];
                        s.process(&p, &mut x, 2);
                        assert!(x[0].is_finite() && x[1].is_finite(), "{name} fs {fs} n {n}");
                        assert!(x[0].abs() < 1e4 && x[1].abs() < 1e4, "{name} fs {fs} n {n}: {x:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn mono_channel_works() {
    let mut s = Strip::new(48000.0);
    let p = extreme();
    for n in 0..48000 {
        let mut x = [(n as f32 * 0.05).sin() * 0.5, 0.0];
        s.process(&p, &mut x, 1);
        assert!(x[0].is_finite());
    }
}
