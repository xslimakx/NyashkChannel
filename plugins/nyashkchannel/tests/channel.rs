use nyashkchannel::dsp::*;

const FS: f32 = 48000.0;

fn run(p: &Params, f: f32, amp: f32, secs: usize) -> Vec<f32> {
    let mut s = Strip::new(FS);
    let mut o = vec![];
    for n in 0..(FS as usize * secs) {
        let v = amp * (2.0 * std::f32::consts::PI * f * n as f32 / FS).sin();
        let mut x = [v, v];
        s.process(p, &mut x, 2);
        assert!(x[0].is_finite() && x[1].is_finite());
        o.push(x[0]);
    }
    o
}
fn rms(v: &[f32]) -> f32 {
    (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt()
}

#[test]
fn defaults_transparent() {
    let o = run(&Params::default(), 1000.0, 0.5, 1);
    let r = rms(&o[24000..]) / (0.5 / 2f32.sqrt());
    assert!((r - 1.0).abs() < 0.01, "{r}");
}

#[test]
fn comp_curve() {
    assert_eq!(comp_gain_db(-30.0, -10.0, 4.0, 0.0), 0.0);
    assert!((comp_gain_db(0.0, -10.0, 4.0, 0.0) + 7.5).abs() < 1e-4);
    assert!(comp_gain_db(-10.0, -10.0, 4.0, 6.0) < 0.0);
}

#[test]
fn expander_curve() {
    assert_eq!(expander_gain_db(-10.0, -40.0, 2.0, 20.0), 0.0);
    assert!((expander_gain_db(-50.0, -40.0, 2.0, 20.0) + 10.0).abs() < 1e-4);
    assert_eq!(expander_gain_db(-90.0, -40.0, 2.0, 20.0), -20.0);
}

#[test]
fn compressor_reduces() {
    let mut p = Params::default();
    p.c_on = true;
    p.c_thr = -20.0;
    p.c_ratio = 4.0;
    let o = run(&p, 1000.0, 0.7, 2);
    let r = rms(&o[FS as usize..]);
    assert!(r < 0.7 / 2f32.sqrt() * 0.8, "{r}");
}

#[test]
fn gate_closes_quiet() {
    let mut p = Params::default();
    p.x_on = true;
    p.x_gate = true;
    p.x_thr = -30.0;
    p.x_range = 40.0;
    let o = run(&p, 1000.0, 0.005, 2);
    assert!(rms(&o[FS as usize..]) < 0.005 / 2f32.sqrt() * 0.1);
}

#[test]
fn eq_boost_and_mono_width() {
    let mut p = Params::default();
    p.eq.hmf.gain_db = 10.0;
    p.eq.hmf.freq = 3000.0;
    let o = run(&p, 3000.0, 0.1, 1);
    assert!(rms(&o[24000..]) > 0.1 / 2f32.sqrt() * 2.5);
    let mut p = Params::default();
    p.mono = true;
    let mut s = Strip::new(FS);
    let mut last = [0.0; 2];
    for n in 0..48000 {
        let mut x = [(n as f32 * 0.1).sin() * 0.5, -(n as f32 * 0.1).sin() * 0.5];
        s.process(&p, &mut x, 2);
        last = x;
    }
    assert!((last[0] - last[1]).abs() < 1e-5);
}

#[test]
fn off_is_bypass() {
    let mut p = Params::default();
    p.power = false;
    p.out_db = -20.0;
    let o = run(&p, 1000.0, 0.5, 1);
    let r = rms(&o[24000..]) / (0.5 / 2f32.sqrt());
    assert!((r - 1.0).abs() < 0.01, "{r}");
}
