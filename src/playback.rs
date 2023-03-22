use std::{fs::File, io::BufReader};

use audrey::Reader;
use nannou_audio::Buffer;
use rtrb::Producer;

use crate::settings::UPDATE_INTERVAL;
pub struct BufferedClip {
    id: usize,
    reader: audrey::read::BufFileReader,
    frames_played: usize,
    last_update_sent: std::time::SystemTime,
}

impl BufferedClip {
    pub fn new(id: usize, reader: Reader<BufReader<File>>) -> Self {
        BufferedClip {
            id,
            reader,
            frames_played: 0,
            last_update_sent: std::time::SystemTime::now(),
        }
    }
}

/// ID of the clip, followed by frames played (count)
pub type ProgressUpdate = (usize, usize);

/// ID of the clip
pub type CompleteUpdate = usize;

pub enum PlaybackState {
    Ready(),
    Playing(usize),
    Complete(),
}

pub struct Audio {
    sounds: Vec<BufferedClip>,
    tx_progress: Producer<ProgressUpdate>,
    tx_complete: Producer<CompleteUpdate>,
}

impl Audio {
    pub fn new(
        tx_progress: Producer<ProgressUpdate>,
        tx_complete: Producer<CompleteUpdate>,
    ) -> Self {
        Audio {
            sounds: Vec::new(),
            tx_progress,
            tx_complete,
        }
    }
    pub fn add_sound(&mut self, new_clip: BufferedClip) {
        self.sounds.push(new_clip);
    }
    pub fn remove_sound(&mut self, id: usize) {
        if let Some(to_remove) = self
            .sounds
            .iter()
            .enumerate()
            .find(|(_index, s)| s.id == id)
        {
            let (index, _s) = to_remove;
            self.sounds.remove(index);
        }
    }
}

pub fn render_audio(audio: &mut Audio, buffer: &mut Buffer) {
    let mut have_ended = vec![];
    let len_frames = buffer.len_frames();

    // Sum all of the sounds onto the buffer.
    for (i, sound) in audio.sounds.iter_mut().enumerate() {
        let mut frame_count = 0;
        let file_frames = sound.reader.frames::<[f32; 2]>().filter_map(Result::ok);
        for (frame, file_frame) in buffer.frames_mut().zip(file_frames) {
            for (sample, file_sample) in frame.iter_mut().zip(&file_frame) {
                *sample += *file_sample;
            }
            frame_count += 1;
        }

        // If the sound yielded less samples than are in the buffer, it must have ended.
        if frame_count < len_frames {
            if !audio.tx_complete.is_full() {
                have_ended.push(i);
                audio.tx_complete.push(sound.id).unwrap();
            }
        } else {
            sound.frames_played += frame_count;

            if sound.last_update_sent.elapsed().unwrap() > UPDATE_INTERVAL
                && !audio.tx_progress.is_full()
            {
                sound.last_update_sent = std::time::SystemTime::now();
                audio
                    .tx_progress
                    .push((sound.id, sound.frames_played))
                    .unwrap();
            }
        }
    }

    // Remove all sounds that have ended.
    for i in have_ended.into_iter().rev() {
        audio.sounds.remove(i);
    }
}
