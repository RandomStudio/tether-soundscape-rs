use std::path::Path;

use nannou::prelude::*;
use nannou_audio as audio;
use nannou_egui::Egui;

use loader::{get_sound_asset_path, load_sample_bank, AudioClipOnDisk};
use playback::{
    render_audio, Audio, BufferedClip, CompleteUpdate, PlaybackState, ProgressUpdate, RequestUpdate,
};
use rtrb::{Consumer, Producer, RingBuffer};
use settings::{build_ui, Settings, CLIP_HEIGHT, CLIP_WIDTH, SAMPLE_RATE, UPDATE_INTERVAL};

mod loader;
mod playback;
mod settings;

pub struct Model {
    rx_progress: Consumer<ProgressUpdate>,
    rx_complete: Consumer<CompleteUpdate>,
    tx_request: Producer<RequestUpdate>,
    stream: audio::Stream<Audio>,
    clips_available: Vec<AudioClipOnDisk>,
    clips_playing: Vec<CurrentlyPlayingClip>,
    left_shift_key_down: bool,
    right_shift_key_down: bool,
    action_queue: Vec<QueueItem>,
    window_id: WindowId,
    egui: Egui,
    settings: Settings,
}
enum QueueItem {
    /// name, should_loop
    Play(String, bool),
    /// id in currently_playing Vec
    Stop(usize),
    /// index in currentl_playing Vec, id for audio model
    Remove(usize, usize),
}

pub struct CurrentlyPlayingClip {
    id: usize,
    name: String,
    length: usize,
    state: PlaybackState,
    should_loop: bool,
    last_update_sent: std::time::SystemTime,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    let window_id = app
        .new_window()
        .key_pressed(key_pressed)
        .key_released(key_released)
        .raw_event(raw_window_event)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    let (tx_progress, rx_progress) = RingBuffer::new(2);
    let (tx_complete, rx_complete) = RingBuffer::new(32);
    let (tx_request, rx_request) = RingBuffer::new(32);
    let audio_model = Audio::new(tx_progress, tx_complete, rx_request);
    // Initialise the state that we want to live on the audio thread.
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(render_audio)
        .sample_rate(SAMPLE_RATE)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    Model {
        stream,
        clips_available: load_sample_bank(app, Path::new("./test_bank.json")),
        clips_playing: Vec::new(),
        rx_progress,
        rx_complete,
        tx_request,
        left_shift_key_down: false,
        right_shift_key_down: false,
        action_queue: Vec::new(),
        window_id,
        egui,
        settings: Settings {
            fadein_duration: 100000,
        },
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

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
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
            let new_clip = BufferedClip::new(
                id,
                Some((
                    0.,
                    clip_matched.volume().unwrap_or(1.0),
                    model.settings.fadein_duration,
                )),
                reader,
            );
            clips_playing.push(CurrentlyPlayingClip {
                id,
                name: String::from(clip_matched.name()),
                length: clip_matched.length().unwrap_or(0),
                state: PlaybackState::Ready(),
                should_loop,
                last_update_sent: std::time::SystemTime::now(),
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

fn update(app: &App, model: &mut Model, update: Update) {
    let window = app.window(model.window_id).unwrap();

    build_ui(model, update.since_start, window.rect());

    for mut sound in &mut model.clips_playing {
        if sound.last_update_sent.elapsed().unwrap() > UPDATE_INTERVAL {
            sound.last_update_sent = std::time::SystemTime::now();
            model
                .tx_request
                .push(sound.id)
                .expect("failed to send request");
            // audio
            //     .tx_progress
            //     .push((sound.id, sound.frames_played))
            //     .unwrap();
        }
    }

    if let Ok(receive) = model.rx_progress.pop() {
        let (id, frames_played) = receive;
        // println!("Got progress update: {}", frames_played);
        if let Some(to_update) = get_clip_index_with_id(&model.clips_playing, id) {
            let (index, _c) = to_update;
            model.clips_playing[index].state = PlaybackState::Playing(frames_played);
        }
    }

    if let Ok(id) = model.rx_complete.pop() {
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

    let stream_state = if model.stream.is_playing() {
        format!("playing {} sounds", model.clips_playing.len())
    } else {
        String::from("paused")
    };
    draw.text(&stream_state).y(45.);

    let start_y = 0.;

    let available_x = -200.;
    for (i, c) in model.clips_available.iter().enumerate() {
        let length = match c.length() {
            Some(frames) => format!("{} fr", frames),
            None => String::from("unknown"),
        };
        draw.text(&format!("KEY #{} ({}) : {}", (i + 1), c.name(), &length))
            .left_justify()
            .x(available_x)
            .y(start_y - (i).to_f32().unwrap() * CLIP_HEIGHT);
    }

    for (i, c) in model.clips_playing.iter().enumerate() {
        let x = 0.;
        let y = start_y - (i).to_f32().unwrap() * CLIP_HEIGHT;

        // Empty box
        draw.rect()
            .no_fill()
            .stroke(BLUE)
            .stroke_weight(1.0)
            .w_h(CLIP_WIDTH, CLIP_HEIGHT)
            .x_y(x, y);

        if let PlaybackState::Playing(frames_played) = c.state {
            // Filling box
            let progress = frames_played.to_f32().unwrap() / c.length.to_f32().unwrap();
            let width = map_range(progress, 0., 1., 0., CLIP_WIDTH);
            draw.rect()
                .color(DARKBLUE)
                .x_y(x + width / 2. - CLIP_WIDTH / 2., y)
                .w_h(width, CLIP_HEIGHT);
        }

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
        .x(x)
        .y(y);
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
