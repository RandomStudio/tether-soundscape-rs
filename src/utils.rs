use std::time::Duration;

use log::debug;
use rand::Rng;

use crate::{loader::AudioClipOnDisk, playback::ClipWithSink, remote_control::PanWithRange};

pub fn get_clip_index_with_name<'a>(
    clips: &'a [ClipWithSink],
    name: &str,
) -> Option<(usize, &'a ClipWithSink)> {
    clips
        .iter()
        .enumerate()
        .find(|(_index, c)| c.name() == name)
        .map(|(index, c)| (index, c))
}

pub fn get_clip_index_with_id(clips: &[ClipWithSink], id: usize) -> Option<(usize, &ClipWithSink)> {
    clips
        .iter()
        .enumerate()
        .find(|(_index, c)| c.id() == id)
        .map(|(index, c)| (index, c))
}

pub fn get_clip_index_with_id_mut(
    clips: &mut [ClipWithSink],
    id: usize,
) -> Option<(usize, &mut ClipWithSink)> {
    clips
        .iter_mut()
        .enumerate()
        .find(|(_index, c)| c.id() == id)
        .map(|(index, c)| (index, c))
}

pub fn get_highest_id(clips: &[ClipWithSink]) -> usize {
    let mut highest_so_far = 0;
    for el in clips {
        if el.id() >= highest_so_far {
            highest_so_far = el.id() + 1;
        }
    }
    highest_so_far
}

pub fn pick_random_clip(clip_names: Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    let index: usize = rng.gen_range(0..clip_names.len());
    clip_names[index].clone()
}

pub fn optional_ms_to_duration(ms: Option<u64>) -> Option<Duration> {
    match ms {
        None => None,
        Some(ms) => Some(Duration::from_millis(ms)),
    }
}

// pub fn get_duration_range(clips: &[AudioClipOnDisk]) -> [u32; 2] {
//     let mut longest: u32 = 0;
//     let mut shortest: Option<u32> = None;

//     for c in clips {
//         if c.frames_count() > longest {
//             longest = c.frames_count()
//         }
//         match shortest {
//             Some(shortest_sofar) => {
//                 if c.frames_count() < shortest_sofar {
//                     shortest = Some(c.frames_count())
//                 }
//             }
//             None => shortest = Some(c.frames_count()),
//         }
//     }
//     [shortest.unwrap_or(0), longest]
// }

// /// Given a list of currently playing clips and a list of clips we *want* to play,
// /// filter the list to return only the names which need to be *added* (i.e. are
// /// in the latter list, but not the former)
// pub fn clips_to_add(
//     currently_playing: &[ClipWithSink],
//     to_play: &[String],
// ) -> impl Iterator<Item = &String> {
//     to_play.iter().filter(|candidate| {
//         currently_playing
//             .iter()
//             .find(|playing| playing.name().eq_ignore_ascii_case(&candidate))
//             .is_none()
//     })
// }

// /// Given a list of currently playing clips and a list of clips we *want* to play/continue,
// /// filter the list to return only the IDs for the clips which need to be *removed*
// /// (i.e. are in the former list, but not the latter)
// pub fn clips_to_remove(
//     currently_playing: &[ClipWithSink],
//     should_be_playing: &[String],
// ) -> Vec<usize> {
//     let mut ids: Vec<usize> = Vec::new();
//     for c in currently_playing {
//         if !should_be_playing
//             .iter()
//             .any(|name| name.eq_ignore_ascii_case(&c.name()))
//         {
//             ids.push(c.id());
//         }
//     }
//     ids
// }
