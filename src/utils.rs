use nannou::prelude::{map_range, ToPrimitive};

use crate::{loader::AudioClipOnDisk, tether::SimplePanning, CurrentlyPlayingClip};

// pub fn frames_to_millis(frames_count: u32, sample_rate: u32) -> u32 {
//     if sample_rate == 0 {
//         panic!("Sample rate should be non-zero");
//     }
//     (frames_count.to_f32().unwrap() / sample_rate.to_f32().unwrap() * 1000.)
//         .to_u32()
//         .unwrap()
// }

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

pub fn get_clip_index_with_name<'a>(
    clips: &'a [CurrentlyPlayingClip],
    name: &str,
) -> Option<(usize, &'a CurrentlyPlayingClip)> {
    clips
        .iter()
        .enumerate()
        .find(|(_index, c)| c.name == name)
        .map(|(index, c)| (index, c))
}

pub fn get_clip_index_with_id(
    clips: &[CurrentlyPlayingClip],
    id: usize,
) -> Option<(usize, &CurrentlyPlayingClip)> {
    clips
        .iter()
        .enumerate()
        .find(|(_index, c)| c.id == id)
        .map(|(index, c)| (index, c))
}

pub fn get_clip_index_with_id_mut(
    clips: &mut [CurrentlyPlayingClip],
    id: usize,
) -> Option<(usize, &mut CurrentlyPlayingClip)> {
    clips
        .iter_mut()
        .enumerate()
        .find(|(_index, c)| c.id == id)
        .map(|(index, c)| (index, c))
}

pub fn get_highest_id(clips: &[CurrentlyPlayingClip]) -> usize {
    let mut highest_so_far = 0;
    for el in clips {
        if el.id >= highest_so_far {
            highest_so_far = el.id + 1;
        }
    }
    highest_so_far
}

pub fn get_duration_range(clips: &[AudioClipOnDisk]) -> [u32; 2] {
    let mut longest: u32 = 0;
    let mut shortest: Option<u32> = None;

    for c in clips {
        if c.frames_count() > longest {
            longest = c.frames_count()
        }
        match shortest {
            Some(shortest_sofar) => {
                if c.frames_count() < shortest_sofar {
                    shortest = Some(c.frames_count())
                }
            }
            None => shortest = Some(c.frames_count()),
        }
    }
    [shortest.unwrap_or(0), longest]
}

/// Given a list of currently playing clips and a list of clips we *want* to play,
/// filter the list to return only the names which need to be *added* (i.e. are
/// in the latter list, but not the former)
pub fn clips_to_add(currently_playing: &[CurrentlyPlayingClip], to_play: &[String]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for name in to_play {
        if !currently_playing
            .iter()
            .any(|c| c.name.eq_ignore_ascii_case(name))
        {
            names.push(String::from(name));
        }
    }

    names
}

/// Given a list of currently playing clips and a list of clips we *want* to play/continue,
/// filter the list to return only the IDs for the clips which need to be *removed*
/// (i.e. are in the former list, but not the latter)
pub fn clips_to_remove(
    currently_playing: &[CurrentlyPlayingClip],
    should_be_playing: &[String],
) -> Vec<usize> {
    let mut ids: Vec<usize> = Vec::new();
    for c in currently_playing {
        if !should_be_playing
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&c.name))
        {
            ids.push(c.id);
        }
    }
    ids
}

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

/// If panning was "optionally" provided, calculate the per-channel volumes as given,
/// otherwise return a suitable default
pub fn provided_or_default_panning(
    provided_panning: Option<SimplePanning>,
    output_channel_count: u32,
) -> Vec<f32> {
    match provided_panning {
        Some((position, spread)) => {
            simple_panning_channel_volumes(position, spread, output_channel_count)
        }
        None => default_panning_channel_volumes(output_channel_count),
    }
}
