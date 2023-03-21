use std::path::Path;
use std::time::Duration;

use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use rtrb::{Consumer, Producer, RingBuffer};
use settings::load_sample_bank;

mod settings;

struct Model {
    consumer: Consumer<ClipUpdate>,
    stream: audio::Stream<Audio>,
    clips: Vec<AudioClipMetadata>,
}

enum PlaybackState {
    Ready(),
    Playing(usize),
    Complete(),
}

struct BufferedClip {
    id: usize,
    reader: audrey::read::BufFileReader,
    frames_played: usize,
    last_update_sent: std::time::SystemTime,
}

pub struct AudioClipMetadata {
    id: usize,
    name: String,
    length: usize,
    state: PlaybackState,
}

/// ID of the clip, followed by "state"
type ClipUpdate = (usize, PlaybackState);

struct Audio {
    sounds: Vec<BufferedClip>,
    producer: Producer<ClipUpdate>,
}

const UPDATE_INTERVAL: Duration = Duration::from_millis(8);

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

    Model {
        stream,
        clips: load_sample_bank(Path::new("./test_bank.json")),
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

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            if model.stream.is_paused() {
                model.stream.play().expect("failed to start stream");
            } else {
                model.stream.pause().expect("failed to pause stream");
            }
        }
        Key::Key1 => {
            let assets = app.assets_path().expect("could not find assets directory");
            let path = assets.join("sounds").join("frog.wav");
            if let Ok(reader) = audrey::open(path) {
                if let Some(clip_matched) = model.clips.iter().find(|c| c.id == 0) {
                    let new_clip = BufferedClip {
                        id: clip_matched.id,
                        reader,
                        frames_played: 0,
                        last_update_sent: std::time::SystemTime::now(),
                    };
                    model
                        .stream
                        .send(move |audio| {
                            audio.sounds.push(new_clip);
                        })
                        .ok();
                }
            }
        }
        _ => {}
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if let Ok(receive) = model.consumer.pop() {
        let (id, state) = receive;
        match state {
            PlaybackState::Complete() => {
                if let Some(to_update) = model
                    .clips
                    .iter()
                    .enumerate()
                    .find(|(_i, clip_meta)| clip_meta.id == id)
                {
                    let (index, _info) = to_update;
                    model.clips[index].state = PlaybackState::Complete();
                } else {
                    panic!("No match for clip id {}", id);
                }
            }
            PlaybackState::Playing(frames_played) => {
                // println!("Got Playing state: {}", frames_played);
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

    let stream_state = if model.stream.is_playing() {
        "playing "
    } else {
        "paused"
    };
    draw.text(stream_state).y(45.);

    let start_y = -45.;
    for (i, c) in model.clips.iter().enumerate() {
        let state_text = match c.state {
            PlaybackState::Playing(frames_played) => {
                let progress = frames_played.to_f32().unwrap() / c.length.to_f32().unwrap();
                format!("{}%", (progress * 100.).trunc())
            }
            PlaybackState::Complete() => String::from("DONE"),
            PlaybackState::Ready() => String::from("READY"),
        };
        draw.text(&format!("#{}({}): ({})", c.id, &c.name, state_text))
            .left_justify()
            .y(start_y - (i * 15).to_f32().unwrap());
    }

    draw.to_frame(app, &frame).unwrap();
}
