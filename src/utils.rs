use nannou::prelude::ToPrimitive;

pub fn frames_to_millis(frames_count: u32, sample_rate: u32) -> u32 {
    if sample_rate == 0 {
        panic!("Sample rate should be non-zero");
    }
    (frames_count.to_f32().unwrap() / sample_rate.to_f32().unwrap() * 1000.)
        .to_u32()
        .unwrap()
}

pub fn frames_to_seconds(frames_count: u32, sample_rate: u32, precision: Option<u32>) -> f32 {
    if sample_rate == 0 {
        panic!("Sample rate should be non-zero");
    }
    let precision = (10_f32).powi(precision.unwrap_or(1).to_i32().unwrap());
    (frames_count.to_f32().unwrap() / sample_rate.to_f32().unwrap() * precision).trunc() / precision
}

pub fn millis_to_frames(millis: u32, sample_rate: u32) -> u32 {
    if sample_rate == 0 {
        panic!("Sample rate should be non-zero");
    }
    (millis.to_f32().unwrap() / 1000. * sample_rate.to_f32().unwrap())
        .to_u32()
        .unwrap()
}
