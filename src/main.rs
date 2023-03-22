use std::path::Path;

use nannou::prelude::*;
use nannou_audio as audio;

use playback::{render_audio, Audio, BufferedClip, ClipUpdate, PlaybackState};
use rtrb::{Consumer, RingBuffer};
use settings::{get_sound_asset_path, load_sample_bank, AudioClipOnDisk};

mod playback;
mod settings;

struct Model {
    consumer: Consumer<ClipUpdate>,
    stream: audio::Stream<Audio>,
    clips_available: Vec<AudioClipOnDisk>,
    clips_playing: Vec<CurrentlyPlayingClip>,
    shift_key_down: bool,
    play_queue: Vec<(String, bool)>,
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
        shift_key_down: false,
        play_queue: Vec::new(),
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

fn trigger_clip(
    app: &App,
    clips_available: &[AudioClipOnDisk],
    clips_playing: &mut Vec<CurrentlyPlayingClip>,
    name: &str,
    should_loop: bool,
) -> Result<BufferedClip, ()> {
    if let Some(clip_matched) = clips_available
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(name))
    {
        let path_str = get_sound_asset_path(app, clip_matched.path());
        if let Ok(reader) = audrey::open(Path::new(&path_str)) {
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
            Ok(new_clip)
        } else {
            println!("No clip found with name {}", name);
            Err(())
        }
    } else {
        Err(())
    }
}

fn start_playback(model: &mut Model, new_clip: BufferedClip) {
    model
        .stream
        .send(move |audio| {
            audio.add_sound(new_clip);
        })
        .ok();
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
            model
                .play_queue
                .push((String::from("frog"), model.shift_key_down));
        }
        Key::Key2 => {
            model
                .play_queue
                .push((String::from("mice"), model.shift_key_down));
        }
        Key::Key3 => {
            model
                .play_queue
                .push((String::from("squirrel"), model.shift_key_down));
        }

        Key::LShift => {
            model.shift_key_down = true;
        }
        _ => {}
    }
}

fn key_released(_app: &App, model: &mut Model, key: Key) {
    if key == Key::LShift {
        model.shift_key_down = false;
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if let Ok(receive) = model.consumer.pop() {
        let (id, state) = receive;
        match state {
            PlaybackState::Complete() => {
                println!("Complete state received for clip ID {}", id);
                if let Some(to_update) = model
                    .clips_playing
                    .iter()
                    .enumerate()
                    .find(|(_i, clip_meta)| clip_meta.id == id)
                {
                    let (index, info) = to_update;
                    println!(
                        "Complete state matches clip with playing index {} and ID {}, name {}",
                        index, info.id, info.name
                    );

                    if info.should_loop {
                        println!("Should loop! Repeat clip with name {}", &info.name);
                        model.play_queue.push((String::from(&info.name), true));
                    }
                    model.clips_playing[index].state = PlaybackState::Complete();
                    model.clips_playing.remove(index);
                } else {
                    panic!("No match for clip id {}", id);
                }
            }
            PlaybackState::Playing(frames_played) => {
                // println!("Got Playing state: {}", frames_played);
                if let Some(to_update) = model
                    .clips_playing
                    .iter()
                    .enumerate()
                    .find(|(_i, clip_meta)| clip_meta.id == id)
                {
                    let (index, _c) = to_update;
                    model.clips_playing[index].state = PlaybackState::Playing(frames_played);
                }
            }
            PlaybackState::Ready() => {}
        }
    }

    while let Some((name, should_loop)) = model.play_queue.pop() {
        if let Ok(new_clip) = trigger_clip(
            app,
            &model.clips_available,
            &mut model.clips_playing,
            &name,
            should_loop,
        ) {
            start_playback(model, new_clip);
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(if model.shift_key_down {
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
        draw.text(&format!("{} : {}", c.name(), &length))
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
