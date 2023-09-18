use crate::utils::map_range;

/// Calculates a final set of per-channel volume levels, given a "position" and a "spread" value,
/// as well as the number of output channels available.
///
/// `position` is a value in the range [0; output_channel_count - 1]. So, in a 4 channel setup, position 3.0 would be "full right".
/// `spread` is a multiple of the "width" of a channel. So, `0.0` means that the signal will be as focussed as possible, i.e. "1 channel width".
pub fn simple_panning_channel_volumes(
    position: f32,
    spread: f32,
    output_channel_count: u16,
) -> Vec<f32> {
    let mut result: Vec<f32> = Vec::new();
    for i in 0..output_channel_count {
        let distance = (position - i as f32).abs();
        let this_channel_volume = f32::max(map_range(distance, 0. ..(1. + spread), 1.0..0.), 0.);
        result.push(this_channel_volume);
    }
    result
}

#[cfg(test)]
#[test]
fn zero_distance_is_max_volume() {
    let output_channel_count: u16 = 2;

    assert_eq!(
        simple_panning_channel_volumes(0., 0., output_channel_count),
        vec![1.0, 0.],
    );
}

#[test]
fn halfway_is_half_volume() {
    let output_channel_count: u16 = 2;

    assert_eq!(
        simple_panning_channel_volumes(0.5, 0., output_channel_count),
        vec![0.5, 0.5]
    );
}

#[test]
fn whole_channel_spread_stereo() {
    let output_channel_count: u16 = 2;

    assert_eq!(
        simple_panning_channel_volumes(1.0, 1.0, output_channel_count),
        vec![0.5, 1.0]
    );
}

#[test]
fn zero_spread_quad_right() {
    let output_channel_count: u16 = 4;

    assert_eq!(
        simple_panning_channel_volumes(3.0, 0., output_channel_count),
        vec![0., 0., 0., 1.0]
    );
}

#[test]
fn whole_channel_spread_quad() {
    let output_channel_count: u16 = 4;

    assert_eq!(
        simple_panning_channel_volumes(1.0, 1.0, output_channel_count),
        vec![0.5, 1.0, 0.5, 0.]
    );
}

#[test]
fn centred_quad_zero_spread() {
    let output_channel_count: u16 = 4;

    assert_eq!(
        simple_panning_channel_volumes(1.5, 0., output_channel_count),
        vec![0., 0.5, 0.5, 0.]
    );
}

#[test]
fn centred_quad_whole_spread() {
    let output_channel_count: u16 = 4;

    assert_eq!(
        simple_panning_channel_volumes(1.5, 1.0, output_channel_count),
        vec![0.25, 0.75, 0.75, 0.25]
    );
}

#[test]
fn centred_eight_double_spread() {
    let output_channel_count: u16 = 8;

    assert_eq!(
        simple_panning_channel_volumes(3.0, 2.0, output_channel_count),
        vec![0., 0.3333333, 0.6666666, 1.0, 0.6666666, 0.3333333, 0., 0.]
    );
}
