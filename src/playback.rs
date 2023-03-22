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

pub enum PlaybackState {
    Ready(),
    Playing(usize),
    Complete(),
}

pub struct Audio {
    sounds: Vec<BufferedClip>,
    producer: Producer<ClipUpdate>,
}

impl Audio {
    pub fn new(producer: Producer<ClipUpdate>) -> Self {
        Audio {
            sounds: Vec::new(),
            producer,
        }
    }
    pub fn add_sound(&mut self, new_clip: BufferedClip) {
        self.sounds.push(new_clip);
    }
}

/// ID of the clip, followed by "state"
pub type ClipUpdate = (usize, PlaybackState);

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
            if !audio.producer.is_full() {
                have_ended.push(i);
                audio
                    .producer
                    .push((sound.id, PlaybackState::Complete()))
                    .unwrap();
            }
        } else {
            sound.frames_played += frame_count;

            if sound.last_update_sent.elapsed().unwrap() > UPDATE_INTERVAL
                && !audio.producer.is_full()
            {
                sound.last_update_sent = std::time::SystemTime::now();
                audio
                    .producer
                    .push((sound.id, PlaybackState::Playing(sound.frames_played)))
                    .unwrap();
            }
        }
    }

    // Remove all sounds that have ended.
    for i in have_ended.into_iter().rev() {
        audio.sounds.remove(i);
    }
}
