use std::time::Duration;

use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use rtrb::{Consumer, Producer, RingBuffer};

struct Model {
    consumer: Consumer<ClipUpdate>,
    stream: audio::Stream<Audio>,
    clips: Vec<AudioClipMetadata>,
}

#[derive(Debug, PartialEq)]
enum PlaybackState {
    Ready(),
    Playing(usize),
    Complete(),
}

struct AudioClip {
    id: usize,
    reader: audrey::read::BufFileReader,
    frames_played: usize,
}

struct AudioClipMetadata {
    id: usize,
    length: usize,
    state: PlaybackState,
}

/// ID of the clip, followed by "state"
type ClipUpdate = (usize, PlaybackState);

struct Audio {
    sounds: Vec<AudioClip>,
    producer: Producer<ClipUpdate>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    let (producer, consumer) = RingBuffer::new(2);
    let audio_model = Audio {
        sounds: Vec::new(),
        producer,
    };
    // Initialise the state that we want to live on the audio thread.
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        .sample_rate(96000)
        .build()
        .unwrap();

    stream.play().unwrap();

    Model {
        stream,
        clips: Vec::new(),
        consumer,
    }
}

fn audio(audio: &mut Audio, buffer: &mut Buffer) {
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
            while audio.producer.is_full() {
                std::thread::sleep(Duration::from_millis(1));
            }
            have_ended.push(i);
            audio
                .producer
                .push((sound.id, PlaybackState::Complete()))
                .unwrap();
        } else {
            sound.frames_played += frame_count;
            if audio.producer.is_full() {
                // Ignore
            } else {
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

fn calculate_length(path: &std::path::Path) -> usize {
    let mut reader = audrey::open(path).unwrap();
    let mut count = 0;
    reader.frames::<[f32; 2]>().for_each(|_f| count += 1);
    count
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    if key == Key::Space {
        let assets = app.assets_path().expect("could not find assets directory");
        let path = assets.join("sounds").join("frog.wav");
        if let Ok(reader) = audrey::open(&path) {
            println!("Opened sound file OK: {:?}", reader.description());
            let length = calculate_length(&path);
            println!("Got length {}", length);
            let id = model.clips.len();
            let new_clip = AudioClip {
                id,
                reader,
                frames_played: 0,
            };
            model.clips.push(AudioClipMetadata {
                id,
                length,
                state: PlaybackState::Ready(),
            });
            model
                .stream
                .send(move |audio| {
                    audio.sounds.push(new_clip);
                })
                .ok();
        } else {
            panic!("Failed to load sound");
        }
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if let Ok(receive) = model.consumer.pop() {
        let (id, state) = receive;
        match state {
            PlaybackState::Complete() => {
                if let Some(to_remove) = model
                    .clips
                    .iter()
                    .enumerate()
                    .find(|(_i, clip_meta)| clip_meta.id == id)
                {
                    let (i, _info) = to_remove;
                    println!("Removing clip at index {}", i);
                    model.clips.remove(i);
                } else {
                    panic!("No match for clip id {}", id);
                }
            }
            PlaybackState::Playing(frames_played) => {
                println!("Got Playing state: {}", frames_played);
                if let Some(to_update) = model
                    .clips
                    .iter()
                    .enumerate()
                    .find(|(_i, clip_meta)| clip_meta.id == id)
                {
                    let (index, _c) = to_update;
                    model.clips[index].state = PlaybackState::Playing(frames_played);
                }
            }
            PlaybackState::Ready() => {}
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(DARKSLATEGREY);
    draw.text(&format!("playing {} sounds", model.clips.len()));

    let start_y = -15.;
    for (i, c) in model.clips.iter().enumerate() {
        if let PlaybackState::Playing(progress) = c.state {
            draw.text(&format!("clip ID#{} : {} / {}", c.id, progress, c.length))
                .left_justify()
                .y(start_y - (i * 15).to_f32().unwrap());
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
