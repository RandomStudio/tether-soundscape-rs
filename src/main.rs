use std::path::{Path, PathBuf};

use nannou::prelude::*;
use nannou_audio as audio;
use nannou_egui::Egui;

use loader::{get_sound_asset_path, SoundBank};
use playback::{
    render_audio, Audio, BufferedClip, CompleteUpdate, PlaybackState, ProgressUpdate, RequestUpdate,
};
use rtrb::{Consumer, Producer, RingBuffer};
use settings::{
    build_ui, Settings, DEFAULT_FADEIN, DEFAULT_FADEOUT, LINE_THICKNESS, MIN_RADIUS,
    RING_BUFFER_SIZE, UPDATE_INTERVAL,
};
use tween::TweenTime;
use utils::{get_clip_index_with_id, get_clip_index_with_id_mut, get_duration_range};

use crate::utils::{get_highest_id, millis_to_frames};

mod loader;
mod playback;
mod settings;
mod utils;

pub struct Model {
    rx_progress: Consumer<ProgressUpdate>,
    rx_complete: Consumer<CompleteUpdate>,
    tx_request: Producer<RequestUpdate>,
    stream: audio::Stream<Audio>,
    sound_bank: SoundBank,
    clips_playing: Vec<CurrentlyPlayingClip>,
    duration_range: [u32; 2],
    action_queue: Vec<QueueItem>,
    window_id: WindowId,
    egui: Egui,
    settings: Settings,
}
pub enum QueueItem {
    /// Start playback: name, optional fade duration in ms, should_loop
    Play(String, Option<u32>, bool),
    /// Stop/fade out: id in currently_playing Vec, optional fade duration in ms
    Stop(usize, Option<u32>),
    /// Remove clip: index in currentl_playing Vec, id for audio model
    Remove(usize, usize),
}

pub struct CurrentlyPlayingClip {
    id: usize,
    name: String,
    frames_count: u32,
    sample_rate: u32,
    state: PlaybackState,
    current_volume: f32,
    should_loop: bool,
    last_update_sent: std::time::SystemTime,
}

impl CurrentlyPlayingClip {
    pub fn length_in_frames(&self) -> u32 {
        self.frames_count
    }
    pub fn length_in_millis(&self) -> u32 {
        (self.frames_count.to_f32() / self.sample_rate.to_f32() / 1000.)
            .to_u32()
            .unwrap()
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    let window_id = app
        .new_window()
        .key_pressed(key_pressed)
        .raw_event(raw_window_event)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    let (tx_progress, rx_progress) = RingBuffer::new(RING_BUFFER_SIZE * 16);
    let (tx_complete, rx_complete) = RingBuffer::new(RING_BUFFER_SIZE);
    let (tx_request, rx_request) = RingBuffer::new(RING_BUFFER_SIZE);
    let audio_model = Audio::new(tx_progress, tx_complete, rx_request);
    // Initialise the state that we want to live on the audio thread.
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(render_audio)
        .sample_rate(96000)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    let sound_bank = SoundBank::new(app, Path::new("./test_bank.json"));
    let duration_range = get_duration_range(&sound_bank.clips());

    Model {
        stream,
        sound_bank,
        clips_playing: Vec::new(),
        duration_range,
        rx_progress,
        rx_complete,
        tx_request,
        action_queue: Vec::new(),
        window_id,
        egui,
        settings: Settings {
            fadein_duration: DEFAULT_FADEIN,
            fadeout_duration: DEFAULT_FADEOUT,
        },
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

pub fn queue_stop_all(
    clips_playing: &mut Vec<CurrentlyPlayingClip>,
    action_queue: &mut Vec<QueueItem>,
    fade: Option<u32>,
) {
    for (index, _clip) in clips_playing.iter().enumerate() {
        action_queue.push(QueueItem::Stop(index, fade));
    }
}

fn start_one(
    name: &str,
    sound_bank: &SoundBank,
    assets_path: PathBuf,
    clips_playing: &mut Vec<CurrentlyPlayingClip>,
    fade: Option<u32>,
    should_loop: bool,
    stream: &audio::Stream<Audio>,
) -> Result<(), ()> {
    if let Some(clip_matched) = sound_bank
        .clips()
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(name))
    {
        let path = get_sound_asset_path(assets_path, clip_matched.path());
        if let Ok(reader) = audrey::open(Path::new(&path)) {
            let id = get_highest_id(clips_playing);

            println!(
                "Start playback for clip name {}, given playing ID #{}",
                clip_matched.name(),
                id
            );
            let new_clip = BufferedClip::new(
                id,
                Some((0., clip_matched.volume().unwrap_or(1.0), fade.unwrap_or(0))),
                reader,
            );
            clips_playing.push(CurrentlyPlayingClip {
                id,
                name: String::from(clip_matched.name()),
                frames_count: clip_matched.frames_count(),
                state: PlaybackState::Ready(),
                current_volume: 0.,
                sample_rate: clip_matched.sample_rate(),
                should_loop,
                last_update_sent: std::time::SystemTime::now(),
            });
            stream
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
        _ => {}
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let window = app.window(model.window_id).unwrap();

    build_ui(model, update.since_start, window.rect());

    // Note the while loop - we try to process ALL progress update messages
    // every frame
    while let Ok(receive) = model.rx_progress.pop() {
        let (id, frames_played, current_volume) = receive;
        // println!("Got progress update: {}", frames_played);
        if let Some(to_update) = get_clip_index_with_id(&model.clips_playing, id) {
            let (index, _c) = to_update;
            model.clips_playing[index].state = PlaybackState::Playing(frames_played);
            model.clips_playing[index].current_volume = current_volume;
        }
    }

    for mut sound in &mut model.clips_playing {
        if sound.last_update_sent.elapsed().unwrap() > UPDATE_INTERVAL {
            sound.last_update_sent = std::time::SystemTime::now();
            // println!("Request for clip ID#{}", sound.id);
            model
                .tx_request
                .push(sound.id)
                .expect("failed to send request");
        }
    }

    while let Ok(id) = model.rx_complete.pop() {
        println!("Complete state received for clip ID {}", id);
        if let Some((index, clip)) = get_clip_index_with_id(&model.clips_playing, id) {
            if clip.should_loop {
                println!("Should loop! Repeat clip with name {}", clip.name);
                model
                    .action_queue
                    .push(QueueItem::Play(String::from(&clip.name), None, true));
            }
            model.action_queue.push(QueueItem::Remove(index, id));
        } else {
            panic!("No match for clip id {}", id);
        }
    }

    while let Some(queue_item) = model.action_queue.pop() {
        match queue_item {
            QueueItem::Play(name, fade, should_loop) => {
                start_one(
                    &name,
                    &model.sound_bank,
                    app.assets_path().expect("failed to fetch assets path"),
                    &mut model.clips_playing,
                    fade,
                    should_loop,
                    &model.stream,
                )
                .expect("failed to start clip");
            }
            QueueItem::Stop(id, fade_out) => {
                if let Some((_index, clip)) =
                    get_clip_index_with_id_mut(&mut model.clips_playing, id)
                {
                    let fadeout_frames = millis_to_frames(fade_out.unwrap_or(0), clip.sample_rate);
                    println!(
                        "Stop clip ID#{}: {}, fade out {}fr",
                        id, &clip.name, fadeout_frames
                    );
                    clip.should_loop = false;
                    model
                        .stream
                        .send(move |audio| audio.fadeout_sound(id, fadeout_frames))
                        .unwrap();
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

    draw.background().color(DARKSLATEGREY);

    let stream_state = if model.stream.is_playing() {
        format!("playing {} sounds", model.clips_playing.len())
    } else {
        String::from("paused")
    };
    draw.text(&stream_state).color(SLATEGREY);

    // let start_y = app.window(model.window_id).unwrap().rect().h() / 3.;

    let max_radius = app.window(model.window_id).unwrap().rect().h() / 2. * 0.9;
    for (_i, c) in model.clips_playing.iter().enumerate() {
        let x = 0.;

        if let PlaybackState::Playing(frames_played) = c.state {
            let opacity = c.current_volume;

            let [min, max] = model.duration_range;
            let radius = map_range(
                c.frames_count.to_f32(),
                min.to_f32(),
                max.to_f32(),
                MIN_RADIUS,
                max_radius,
            );
            let progress = frames_played.to_f32() / c.frames_count.to_f32();
            let target_angle = PI * 2.0 * progress; // "percent" of full circle
            let brightness = 0.5;

            draw.ellipse()
                .radius(radius)
                .x_y(x, 0.)
                .no_fill()
                .stroke(rgba(brightness, brightness, brightness, opacity))
                .stroke_weight(LINE_THICKNESS * 2.);

            let num_dots: usize = 1000;
            let brightness = 1.0;
            let white = rgba(brightness, brightness, brightness, opacity);
            for dot in 0..num_dots {
                let angle = -map_range(dot.to_f32(), 0., num_dots.to_f32(), 0., target_angle);
                let x = radius * angle.cos();
                let dot_y = radius * angle.sin();
                draw.ellipse()
                    .x_y(x, dot_y)
                    .radius(LINE_THICKNESS)
                    .color(white);
            }
            draw.text(&c.name).x_y(0., -radius - 15.).color(white);
        }
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
