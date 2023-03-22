use std::path::Path;

use nannou::prelude::*;
use nannou_audio as audio;

use playback::{render_audio, Audio, BufferedClip, ClipUpdate, PlaybackState};
use rtrb::{Consumer, RingBuffer};
use settings::{get_sound_asset_path, load_sample_bank, AudioClipOnDisk};

mod playback;
mod settings;

enum QueueItem {
    /// name, should_loop
    Play(String, bool),
    /// id in currently_playing Vec
    Stop(usize),
    /// index in currentl_playing Vec, id for audio model
    Remove(usize, usize),
}

struct Model {
    consumer: Consumer<ClipUpdate>,
    stream: audio::Stream<Audio>,
    clips_available: Vec<AudioClipOnDisk>,
    clips_playing: Vec<CurrentlyPlayingClip>,
    left_shift_key_down: bool,
    right_shift_key_down: bool,
    action_queue: Vec<QueueItem>,
}

pub struct CurrentlyPlayingClip {
    id: usize,
    name: String,
    length: usize,
    state: PlaybackState,
    should_loop: bool,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .key_pressed(key_pressed)
        .key_released(key_released)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    let (producer, consumer) = RingBuffer::new(2);
    let audio_model = Audio::new(producer);
    // Initialise the state that we want to live on the audio thread.
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(render_audio)
        .sample_rate(96000)
        .build()
        .unwrap();

    Model {
        stream,
        clips_available: load_sample_bank(app, Path::new("./test_bank.json")),
        clips_playing: Vec::new(),
        consumer,
        left_shift_key_down: false,
        right_shift_key_down: false,
        action_queue: Vec::new(),
    }
}

fn get_highest_id(clips: &[CurrentlyPlayingClip]) -> usize {
    let mut highest_so_far = 0;
    for el in clips {
        if el.id >= highest_so_far {
            highest_so_far = el.id + 1;
        }
    }
    highest_so_far
}

fn trigger_clip(app: &App, model: &mut Model, name: &str, should_loop: bool) -> Result<(), ()> {
    if let Some(clip_matched) = model
        .clips_available
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(name))
    {
        let path_str = get_sound_asset_path(app, clip_matched.path());
        if let Ok(reader) = audrey::open(Path::new(&path_str)) {
            let clips_playing = &mut model.clips_playing;
            let id = get_highest_id(clips_playing);

            println!(
                "Start playback for clip name {}, given playing ID #{}",
                clip_matched.name(),
                id
            );
            let new_clip = BufferedClip::new(id, reader);
            clips_playing.push(CurrentlyPlayingClip {
                id,
                name: String::from(clip_matched.name()),
                length: clip_matched.length().unwrap_or(0),
                state: PlaybackState::Ready(),
                should_loop,
            });
            model
                .stream
                .send(move |audio| {
                    audio.add_sound(new_clip);
                })
                .ok();
            Ok(())
        } else {
            println!("No clip found with name {}", name);
            Err(())
        }
    } else {
        Err(())
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            if model.stream.is_paused() {
                model.stream.play().expect("failed to start stream");
            } else {
                model.stream.pause().expect("failed to pause stream");
            }
        }
        Key::Key1 => {
            if model.right_shift_key_down {
                if let Some((_index, info)) = get_clip_index_with_name(&model.clips_playing, "frog")
                {
                    model.action_queue.push(QueueItem::Stop(info.id));
                }
            } else {
                model.action_queue.push(QueueItem::Play(
                    String::from("frog"),
                    model.left_shift_key_down,
                ));
            }
        }
        Key::Key2 => {
            if model.right_shift_key_down {
                if let Some((_index, info)) = get_clip_index_with_name(&model.clips_playing, "mice")
                {
                    model.action_queue.push(QueueItem::Stop(info.id));
                }
            } else {
                model.action_queue.push(QueueItem::Play(
                    String::from("mice"),
                    model.left_shift_key_down,
                ));
            }
        }
        Key::Key3 => {
            if model.right_shift_key_down {
                if let Some((_index, info)) =
                    get_clip_index_with_name(&model.clips_playing, "squirrel")
                {
                    model.action_queue.push(QueueItem::Stop(info.id));
                }
            } else {
                model.action_queue.push(QueueItem::Play(
                    String::from("squirrel"),
                    model.left_shift_key_down,
                ));
            }
        }

        Key::LShift => {
            model.left_shift_key_down = true;
        }
        Key::RShift => {
            model.right_shift_key_down = true;
        }
        _ => {}
    }
}

fn key_released(_app: &App, model: &mut Model, key: Key) {
    if key == Key::LShift {
        model.left_shift_key_down = false;
    }
    if key == Key::RShift {
        model.right_shift_key_down = false;
    }
}

fn get_clip_index_with_name<'a>(
    clips: &'a [CurrentlyPlayingClip],
    name: &str,
) -> Option<(usize, &'a CurrentlyPlayingClip)> {
    clips
        .iter()
        .enumerate()
        .find(|(_index, c)| c.name == name)
        .map(|(index, c)| (index, c))
}

fn get_clip_index_with_id(
    clips: &[CurrentlyPlayingClip],
    id: usize,
) -> Option<(usize, &CurrentlyPlayingClip)> {
    clips
        .iter()
        .enumerate()
        .find(|(_index, c)| c.id == id)
        .map(|(index, c)| (index, c))
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if let Ok(receive) = model.consumer.pop() {
        let (id, state) = receive;
        match state {
            PlaybackState::Complete() => {
                println!("Complete state received for clip ID {}", id);
                if let Some((index, clip)) = get_clip_index_with_id(&model.clips_playing, id) {
                    if clip.should_loop {
                        println!("Should loop! Repeat clip with name {}", clip.name);
                        model
                            .action_queue
                            .push(QueueItem::Play(String::from(&clip.name), true));
                    }
                    model.clips_playing[index].state = PlaybackState::Complete();
                    model.clips_playing.remove(index);
                } else {
                    panic!("No match for clip id {}", id);
                }
            }
            PlaybackState::Playing(frames_played) => {
                // println!("Got Playing state: {}", frames_played);
                if let Some(to_update) = get_clip_index_with_id(&model.clips_playing, id) {
                    let (index, _c) = to_update;
                    model.clips_playing[index].state = PlaybackState::Playing(frames_played);
                }
            }
            PlaybackState::Ready() => {}
        }
    }

    while let Some(queue_item) = model.action_queue.pop() {
        match queue_item {
            QueueItem::Play(name, should_loop) => {
                trigger_clip(app, model, &name, should_loop).unwrap();
            }
            QueueItem::Stop(id) => {
                if let Some((index, clip)) = get_clip_index_with_id(&model.clips_playing, id) {
                    model.action_queue.push(QueueItem::Remove(index, clip.id));
                }
            }
            QueueItem::Remove(index, id) => {
                model
                    .stream
                    .send(move |audio| {
                        audio.remove_sound(id);
                    })
                    .unwrap();
                model.clips_playing.remove(index);
            }
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(if model.left_shift_key_down {
        SLATEGREY
    } else {
        DARKSLATEGREY
    });
    draw.text(&format!("playing {} sounds", model.clips_playing.len()));

    let stream_state = if model.stream.is_playing() {
        "playing "
    } else {
        "paused"
    };
    draw.text(stream_state).y(45.);

    let start_y = -45.;

    let available_x = -100.;
    for (i, c) in model.clips_available.iter().enumerate() {
        let length = match c.length() {
            Some(frames) => format!("{} fr", frames),
            None => String::from("unknown"),
        };
        draw.text(&format!("KEY #{} ({}) : {}", (i + 1), c.name(), &length))
            .left_justify()
            .x(available_x)
            .y(start_y - (i * 15).to_f32().unwrap());
    }

    let playing_x = 100.;
    for (i, c) in model.clips_playing.iter().enumerate() {
        let state_text = match c.state {
            PlaybackState::Playing(frames_played) => {
                let progress = frames_played.to_f32().unwrap() / c.length.to_f32().unwrap();
                format!("{}%", (progress * 100.).trunc())
            }
            PlaybackState::Complete() => String::from("DONE"),
            PlaybackState::Ready() => String::from("READY"),
        };
        let loop_text = if c.should_loop { "LOOP" } else { "ONCE" };
        draw.text(&format!(
            "#{} ({}): ({}) - {}",
            c.id, &c.name, state_text, loop_text
        ))
        .left_justify()
        .x(playing_x)
        .y(start_y - (i * 15).to_f32().unwrap());
    }

    draw.to_frame(app, &frame).unwrap();
}
