//! Deeper behaviour checks of the channel strip.
use nyashkchannel::dsp::{comp_gain_db, Params, Strip};
use std::f32::consts::PI;

fn db(x: f32) -> f32 {
    20.0 * x.max(1e-9).log10()
}

/// steady sine; returns output peak (dB) over the second half
fn steady(p: &Params, amp: f32, f: f32) -> f32 {
    let fs = 48000.0;
    let mut s = Strip::new(fs);
    let mut pk = 0.0f32;
    for i in 0..96000 {
        let v = amp * (2.0 * PI * f * i as f32 / fs).sin();
        let mut x = [v, v];
        s.process(p, &mut x, 2);
        if i > 48000 {
            pk = pk.max(x[0].abs());
        }
    }
    db(pk)
}

#[test]
fn compressor_follows_its_static_curve() {
    for ratio in [2.0f32, 4.0, 10.0] {
        let p = Params { c_on: true, c_thr: -20.0, c_ratio: ratio, ..Params::default() };
        for in_db in [-30.0f32, -20.0, -10.0, -4.0] {
            let out = steady(&p, 10f32.powf(in_db / 20.0), 1000.0);
            let want = in_db + comp_gain_db(in_db, -20.0, ratio, 4.0);
            assert!((out - want).abs() < 0.6, "ratio {ratio}, in {in_db}: out {out:.2}, curve {want:.2}");
        }
    }
}

#[test]
fn compressor_attack_and_release_times() {
    let fs = 48000.0;
    let measure = |fast: bool, rel_ms: f32| {
        let p = Params { c_on: true, c_thr: -30.0, c_ratio: 10.0, c_fast: fast, c_rel_ms: rel_ms, ..Params::default() };
        let mut s = Strip::new(fs);
        let mut grs = vec![];
        for i in 0..(fs * 3.0) as usize {
            let t = i as f32 / fs;
            let a = if (0.5..1.5).contains(&t) { 0.5 } else { 0.01 };
            let mut x = [a * (2.0 * PI * 1000.0 * t).sin(); 2];
            s.process(&p, &mut x, 2);
            grs.push((t, s.gr_comp));
        }
        let peak = grs.iter().filter(|g| g.0 < 1.5).map(|g| g.1).fold(0.0f32, f32::max);
        let att = grs.iter().find(|g| g.0 >= 0.5 && g.1 >= 0.63 * peak).unwrap().0 - 0.5;
        let rel = grs.iter().find(|g| g.0 >= 1.5 && g.1 <= 0.37 * peak).map(|g| g.0 - 1.5).unwrap_or(9.0);
        (att * 1000.0, rel * 1000.0)
    };
    for (fast, rel_ms) in [(false, 100.0f32), (true, 100.0), (false, 1000.0)] {
        let (a, r) = measure(fast, rel_ms);
        println!("fast {fast}, release {rel_ms} ms: attack {a:.2} ms, release (to 37 %) {r:.0} ms");
        let want_a = if fast { 0.3 } else { 6.0 };
        assert!(a < want_a * 3.0 + 0.5, "attack {a}");
        assert!(r > rel_ms * 0.5 && r < rel_ms * 2.0, "release {r} for {rel_ms}");
    }
}

#[test]
fn gate_and_expander() {
    // below the threshold: the gate takes off its range, the expander less
    let quiet = 10f32.powf(-60.0 / 20.0);
    let open = steady(&Params { x_on: true, x_gate: true, x_thr: -40.0, x_range: 30.0, ..Params::default() }, 0.1, 1000.0);
    let shut = steady(&Params { x_on: true, x_gate: true, x_thr: -40.0, x_range: 30.0, ..Params::default() }, quiet, 1000.0);
    let exp = steady(&Params { x_on: true, x_gate: false, x_thr: -40.0, x_range: 30.0, ..Params::default() }, quiet, 1000.0);
    println!("gate: above {open:.2} dB (in -20), below {shut:.2} dB (in -60, range 30); expander below {exp:.2} dB");
    assert!((open + 20.0).abs() < 0.3, "gate touches a signal above the threshold");
    assert!((shut + 90.0).abs() < 1.5, "gate range: {shut}");
    assert!(exp > shut + 3.0 && exp < -61.0, "expander: {exp}");
}

#[test]
fn input_stage_pad_phase_width() {
    let a = 0.1;
    let pad = steady(&Params { pad: true, ..Params::default() }, a, 1000.0);
    assert!((pad - (db(a) - 20.0)).abs() < 0.1, "pad {pad}");
    let line = steady(&Params { line_db: 6.0, ..Params::default() }, a, 1000.0);
    assert!((line - (db(a) + 6.0)).abs() < 0.1, "line {line}");
    // phase flips the polarity
    let mut s = Strip::new(48000.0);
    let p = Params { phase: true, ..Params::default() };
    let mut x = [0.0f32; 2];
    for _ in 0..4800 {
        x = [0.3, 0.3];
        s.process(&p, &mut x, 2);
    }
    assert!(x[0] < -0.29, "phase: {}", x[0]);
    // mono: L-only input comes out on both sides at half
    let mut s = Strip::new(48000.0);
    let p = Params { mono: true, ..Params::default() };
    let mut x = [0.0f32; 2];
    for _ in 0..4800 {
        x = [0.4, 0.0];
        s.process(&p, &mut x, 2);
    }
    assert!((x[0] - 0.2).abs() < 0.01 && (x[1] - 0.2).abs() < 0.01, "mono: {x:?}");
}

#[test]
fn split_filters_only_the_sidechain() {
    // SPLIT: the high-pass keys the dynamics but the audio keeps its lows
    let p = Params { split: true, hp_on: true, hp_f: 300.0, ..Params::default() };
    let lo = steady(&p, 0.1, 60.0);
    println!("split + HP 300: 60 Hz {lo:.2} dB (in -20)");
    assert!((lo + 20.0).abs() < 0.3, "split filtered the audio");
    let p2 = Params { split: false, ..p };
    assert!(steady(&p2, 0.1, 60.0) < -30.0, "HP does not cut");
}

/// Largest second difference of the output while one switch is flipped every 0.3 s, relative to
/// the largest one of the same settings without switching (a click is a sharp corner).
fn click_ratio(what: &str) -> f32 {
    let fs = 48000.0;
    let f = 100.0;
    let run = |toggle: bool| {
        let mut s = Strip::new(fs);
        let mut p = Params { hp_f: 150.0, lp_f: 1500.0, c_thr: -30.0, c_ratio: 6.0, x_thr: -10.0, x_range: 20.0, ..Params::default() };
        p.eq.lf.gain_db = 9.0;
        p.eq.lf.freq = 100.0;
        p.eq.lf.bell = true;
        let mut worst = 0.0f32;
        let (mut y1, mut y2) = (0.0f32, 0.0f32);
        for i in 0..(fs * 1.2) as usize {
            if toggle && i % 14400 == 0 && i > 0 {
                match what {
                    "hp" => p.hp_on = !p.hp_on,
                    "lp" => p.lp_on = !p.lp_on,
                    "eq" => p.eq_on = !p.eq_on,
                    "analog" => p.analog = !p.analog,
                    "comp" => p.c_on = !p.c_on,
                    "gate" => p.x_on = !p.x_on,
                    "power" => p.power = !p.power,
                    _ => p.dyn_post = !p.dyn_post,
                }
            }
            let t = i as f32 / fs;
            let v = 0.08 * (2.0 * PI * f * t).sin() + 0.02 * (2.0 * PI * 700.0 * t).sin();
            let mut x = [v, v];
            s.process(&p, &mut x, 2);
            if i > 4800 {
                worst = worst.max((x[0] - 2.0 * y1 + y2).abs());
            }
            y2 = y1;
            y1 = x[0];
        }
        worst
    };
    run(true) / run(false)
}

#[test]
fn switches_do_not_click() {
    let mut bad = vec![];
    for what in ["hp", "lp", "eq", "analog", "comp", "gate", "power", "order"] {
        let r = click_ratio(what);
        println!("{what}: sharpest corner x{r:.2} of the steady signal");
        if r > 2.0 {
            bad.push(format!("{what} x{r:.1}"));
        }
    }
    assert!(bad.is_empty(), "clicks: {bad:?}");
}
