use nannou::prelude::ToPrimitive;

use crate::{loader::AudioClipOnDisk, CurrentlyPlayingClip};

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
