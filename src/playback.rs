use std::{fs::File, io::BufReader};

use audrey::Reader;
use nannou::prelude::ToPrimitive;
use nannou_audio::Buffer;
use rtrb::{Consumer, Producer};
use tween::{Linear, Tween, Tweener};

use crate::settings::SAMPLE_RATE;

/// Volume value, duration IN FRAMES
type StoredTweener = Tweener<f32, usize, Box<dyn Tween<f32> + Send + Sync>>;

pub struct BufferedClip {
    id: usize,
    current_volume: f32,
    reader: audrey::read::BufFileReader,
    frames_played: usize,
    phase: PlaybackPhase,
}

/// Start volume, End volume, Duration IN MILLISECONDS
pub type Fade = (f32, f32, usize);
pub enum PlaybackPhase {
    Attack(StoredTweener),
    Sustain(),
    Release(StoredTweener),
}

impl BufferedClip {
    pub fn new(id: usize, fade_in: Option<Fade>, reader: Reader<BufReader<File>>) -> Self {
        let start_volume = match fade_in {
            Some(fade) => {
                let (start, _end, _duration) = fade;
                start
            }
            None => 0.,
        };
        BufferedClip {
            id,
            reader,
            frames_played: 0,
            current_volume: start_volume,
            phase: match fade_in {
                Some((start, end, duration_ms)) => {
                    let tween: Box<dyn Tween<f32> + Send + Sync> = Box::new(Linear);
                    let duration_frames = (duration_ms.to_f32().unwrap() / 1000.
                        * SAMPLE_RATE.to_f32().unwrap())
                    .to_usize()
                    .unwrap();
                    println!(
                        "{} ms to {} @ {}KHz",
                        duration_ms,
                        duration_frames,
                        SAMPLE_RATE / 1000
                    );
                    let stored_tweener = Tweener::new(start, end, duration_frames, tween);
                    PlaybackPhase::Attack(stored_tweener)
                }
                None => {
                    let tween: Box<dyn Tween<f32> + Send + Sync> = Box::new(Linear);
                    let stored_tweener = Tweener::new(0., 1.0, 1000, tween);
                    PlaybackPhase::Attack(stored_tweener)
                }
            },
        }
    }
}

/// audio -> main: ID of the clip, followed by frames played (count)
pub type ProgressUpdate = (usize, usize);

/// audio -> main: ID of the clip
pub type CompleteUpdate = usize;

/// audio <- main: ID of the clip
pub type RequestUpdate = usize;

pub enum PlaybackState {
    Ready(),
    Playing(usize),
    Complete(),
}

pub struct Audio {
    sounds: Vec<BufferedClip>,
    tx_progress: Producer<ProgressUpdate>,
    tx_complete: Producer<CompleteUpdate>,
    rx_request: Consumer<RequestUpdate>,
}

impl Audio {
    pub fn new(
        tx_progress: Producer<ProgressUpdate>,
        tx_complete: Producer<CompleteUpdate>,
        rx_request: Consumer<RequestUpdate>,
    ) -> Self {
        Audio {
            sounds: Vec::new(),
            tx_progress,
            tx_complete,
            rx_request,
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
                *sample += *file_sample * sound.current_volume;
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

            sound.current_volume = match &mut sound.phase {
                PlaybackPhase::Attack(tween) => tween.move_by(frame_count),
                PlaybackPhase::Sustain() => sound.current_volume,
                PlaybackPhase::Release(tween) => tween.move_by(frame_count),
            };

            if let PlaybackPhase::Attack(tween) = &mut sound.phase {
                if tween.is_finished() {
                    println!("sound attack => sustain");
                    sound.phase = PlaybackPhase::Sustain();
                }
            }

            if let Ok(receive_id) = audio.rx_request.pop() {
                if sound.id == receive_id && !audio.tx_progress.is_full() {
                    audio
                        .tx_progress
                        .push((sound.id, sound.frames_played))
                        .expect("failed to send progress");
                }
            }
        }
    }

    // Remove all sounds that have ended.
    for i in have_ended.into_iter().rev() {
        audio.sounds.remove(i);
    }
}
