//! Every parameter's display text must fit the value boxes (9 monospace characters).
use nih_plug::prelude::*;

#[test]
fn value_texts_fit_their_boxes() {
    let p = nyashkchannel::ChannelParams::default();
    let mut worst = Vec::new();
    for (id, ptr, _) in p.param_map() {
        for i in 0..=400 {
            let n = i as f32 / 400.0;
            let s = unsafe { ptr.normalized_value_to_string(n, true) };
            let len = s.chars().count();
            if len > 9 {
                worst.push(format!("{id}: '{s}' ({len})"));
                break;
            }
        }
    }
    for w in &worst {
        println!("too long: {w}");
    }
    assert!(worst.is_empty(), "{} parameters print longer than the box", worst.len());
}
