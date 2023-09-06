pub fn equalise_channel_volumes(output_channel_count: u32) -> Vec<f32> {
    let mut result: Vec<f32> = Vec::new();
    let max_volume = 1.0 / output_channel_count.to_f32().unwrap();
    for _i in 0..output_channel_count {
        result.push(max_volume);
    }
    if result.len() != output_channel_count.to_usize().unwrap() {
        panic!(
            "Per-channel vector should have {} values, got {}",
            output_channel_count,
            result.len()
        );
    }
    result
}

/// Calculates a final set of per-channel volume levels, given a "position" and a "spread" value,
/// as well as the number of output channels available
pub fn simple_panning_channel_volumes(
    position: f32,
    spread: f32,
    output_channel_count: u32,
) -> Vec<f32> {
    let mut result: Vec<f32> = Vec::new();
    for i in 0..output_channel_count {
        let distance = (position - i.to_f32().unwrap()).abs();
        let this_channel_volume = f32::max(map_range(distance, 0., spread, 1.0, 0.), 0.);
        result.push(this_channel_volume);
    }
    result
}

/// Calculate a final set of per-channel volume levels in a "default case", suitable for a given
/// channel count
pub fn default_panning_channel_volumes(output_channel_count: u32) -> Vec<f32> {
    let position = (output_channel_count.to_f32().unwrap() - 1.0) / 2.;
    simple_panning_channel_volumes(position, 1.0, output_channel_count)
}

/// Three possible levels (higher override lower):
/// - Panning provided by Tether Message
/// - Panning provided from clip settings
/// - None provided; use default (equalise channels)
pub fn provided_or_default_panning(
    message_provided_panning: Option<PanWithRange>,
    clip_default_panning: Option<PanWithRange>,
    output_channel_count: u32,
) -> Vec<f32> {
    debug!("Message provided panning: {:?}", message_provided_panning);
    debug!("Clip default panning: {:?}", clip_default_panning);
    match message_provided_panning {
        Some((position, spread)) => {
            debug!("Use message provided panning");
            simple_panning_channel_volumes(position, spread, output_channel_count)
        }
        None => match clip_default_panning {
            Some((position, spread)) => {
                debug!("Use clip default panning");
                simple_panning_channel_volumes(position, spread, output_channel_count)
            }
            None => {
                debug!("No overrides; use equalised channels");
                default_panning_channel_volumes(output_channel_count)
            }
        },
    }
}
