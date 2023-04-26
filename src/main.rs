use audio::Device;
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_egui::Egui;

use clap::Parser;

use env_logger::{Builder, Env};
use log::{debug, error, info, trace, warn};
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};
use tether::Instruction;

use loader::{get_sound_asset_path, SoundBank};
use playback::{
    render_audio_multichannel, Audio, BufferedClip, CompleteUpdate, PlaybackState, ProgressUpdate,
    RequestUpdate,
};
use rtrb::{Consumer, Producer, RingBuffer};
use settings::{
    Cli, ManualSettings, LINE_THICKNESS, MIN_RADIUS, RING_BUFFER_SIZE, UPDATE_INTERVAL,
};
use tween::TweenTime;
use ui::build_ui;
use utils::{
    all_channels_equal, get_clip_index_with_id, get_clip_index_with_id_mut,
    get_clip_index_with_name, get_duration_range,
};

use crate::{
    playback::render_audio_stereo,
    settings::pick_default_sample_bank,
    tether::TetherAgent,
    utils::{clips_to_remove, get_highest_id, millis_to_frames},
};

mod loader;
mod playback;
mod settings;
mod tether;
mod ui;
mod utils;

pub type FadeDuration = u32;

pub struct Model {
    rx_progress: Consumer<ProgressUpdate>,
    rx_complete: Consumer<CompleteUpdate>,
    tx_request: Producer<RequestUpdate>,
    stream: audio::Stream<Audio>,
    sound_bank: SoundBank,
    clips_playing: Vec<CurrentlyPlayingClip>,
    duration_range: [FadeDuration; 2],
    action_queue: Vec<QueueItem>,
    last_state_publish: SystemTime,
    window_id: WindowId,
    egui: Egui,
    settings: ManualSettings,
    multi_channel_mode: bool,
    tether: TetherAgent,
}
pub enum QueueItem {
    /// Start playback: name, optional fade duration in ms, should_loop,
    /// optional per-channel-volume
    Play(String, Option<FadeDuration>, bool, Option<Vec<f32>>),
    /// Stop/fade out: id in currently_playing Vec, optional fade duration in ms
    Stop(usize, Option<FadeDuration>),
    /// Remove clip: id in currently_playing Vec
    Remove(usize),
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
    let cli = Cli::parse();

    let mut builder = Builder::from_env(Env::default().default_filter_or(&cli.log_level));
    builder.filter_module("wgpu_core", log::LevelFilter::Error);
    builder.filter_module("wgpu_hal", log::LevelFilter::Warn);
    builder.filter_module("naga", log::LevelFilter::Warn);
    builder.init();
    info!("Started; args: {:?}", cli);
    debug!("Debugging is enabled; could be verbose");

    let settings = ManualSettings::defaults();

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

    let device = match cli.preferred_output_device {
        Some(device_name) => {
            if let Ok(devices) = audio_host.output_devices() {
                let mut matching_device: Option<Device> = None;
                for d in devices {
                    debug!("output device: {:?}", d.name());
                    if d.name().unwrap() == device_name {
                        info!(
                            "Found matching device {} == {}",
                            &d.name().unwrap(),
                            &device_name
                        );
                        matching_device = Some(d);
                    }
                }
                matching_device
            } else {
                panic!("Failed to enumerate host audio devices");
            }
        }
        None => audio_host.default_output_device(),
    };

    let (tx_progress, rx_progress) = RingBuffer::new(RING_BUFFER_SIZE * 16);
    let (tx_complete, rx_complete) = RingBuffer::new(RING_BUFFER_SIZE);
    let (tx_request, rx_request) = RingBuffer::new(RING_BUFFER_SIZE);
    let audio_model = Audio::new(tx_progress, tx_complete, rx_request);

    if cli.multichannel_mode {
        info!("Multichannel mode; use mono clips only");
    } else {
        info!("Stereo playback mode enabled; will disable multi-channel panning features");
    }

    // Initialise the state that we want to live on the audio thread.
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(if cli.multichannel_mode {
            render_audio_multichannel
        } else {
            render_audio_stereo
        })
        .device(device.unwrap())
        .sample_rate(cli.sample_rate)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let mode = if cli.multichannel_mode {
        "MULTICHANNEL"
    } else {
        "STEREO"
    };
    window.set_title(&format!(
        "Tether Soundscape @{} KHz - {} MODE",
        cli.sample_rate / 1000,
        mode
    ));
    let egui = Egui::from_window(&window);

    let sound_bank = SoundBank::new(
        app,
        Path::new(
            &cli.sample_bank_path
                .unwrap_or(pick_default_sample_bank(cli.multichannel_mode)),
        ),
        cli.multichannel_mode,
    );
    let duration_range = get_duration_range(sound_bank.clips());

    let mut tether = TetherAgent::new(cli.tether_host);
    if !cli.tether_disable {
        tether.connect();
    } else {
        warn!("Tether connection disabled")
    }

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
        settings,
        tether,
        last_state_publish: std::time::SystemTime::now(),
        multi_channel_mode: cli.multichannel_mode,
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

pub fn queue_stop_all(
    clips_playing: &mut [CurrentlyPlayingClip],
    action_queue: &mut Vec<QueueItem>,
    fade: Option<u32>,
) {
    for (_index, clip) in clips_playing.iter().enumerate() {
        action_queue.push(QueueItem::Stop(clip.id, fade));
    }
}

// TODO: reduce the number of arguments here. Use an enum
// to encapsulate the "settings" for starting this clip?
fn start_one(
    name: &str,
    sound_bank: &SoundBank,
    assets_path: PathBuf,
    clips_playing: &mut Vec<CurrentlyPlayingClip>,
    fade: Option<u32>,
    should_loop: bool,
    stream: &audio::Stream<Audio>,
    per_channel_volume: Vec<f32>,
) -> Result<(), ()> {
    if let Some(clip_matched) = sound_bank
        .clips()
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(name))
    {
        let path = get_sound_asset_path(assets_path, clip_matched.path());
        if let Ok(reader) = audrey::open(Path::new(&path)) {
            let id = get_highest_id(clips_playing);

            info!(
                "Start playback for clip name {}, given playing ID #{}",
                clip_matched.name(),
                id
            );
            let new_clip = BufferedClip::new(
                id,
                Some((0., clip_matched.volume().unwrap_or(1.0), fade.unwrap_or(0))),
                reader,
                per_channel_volume,
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
            error!("No clip found with name {}", name);
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
    // let window = app.window(model.window_id).unwrap();

    build_ui(model, update.since_start, model.multi_channel_mode);

    // Note the while loop - we try to process ALL progress update messages
    // every frame
    while let Ok(receive) = model.rx_progress.pop() {
        let (id, frames_played, current_volume) = receive;
        if let Some(to_update) = get_clip_index_with_id(&model.clips_playing, id) {
            let (index, _c) = to_update;
            model.clips_playing[index].state = PlaybackState::Playing(frames_played);
            model.clips_playing[index].current_volume = current_volume;
        }
    }

    for mut sound in &mut model.clips_playing {
        if sound.last_update_sent.elapsed().unwrap() > UPDATE_INTERVAL {
            sound.last_update_sent = std::time::SystemTime::now();
            trace!("Request for clip ID#{}", sound.id);
            model
                .tx_request
                .push(sound.id)
                .expect("failed to send request");
        }
    }

    if model.last_state_publish.elapsed().unwrap() > UPDATE_INTERVAL {
        model
            .tether
            .publish_state(model.stream.is_playing(), &model.clips_playing);
    }

    while let Ok(id) = model.rx_complete.pop() {
        debug!("Complete state received for clip ID {}", id);
        if let Some((_index, clip)) = get_clip_index_with_id(&model.clips_playing, id) {
            if clip.should_loop {
                debug!("Should loop! Repeat clip with name {}", clip.name);
                model.action_queue.push(QueueItem::Play(
                    String::from(&clip.name),
                    None,
                    true,
                    None, // TODO: get previous per-channel-volume
                ));
            }
            model.action_queue.push(QueueItem::Remove(id));
        } else {
            panic!("No match for clip id {}", id);
        }
    }

    while let Some(queue_item) = model.action_queue.pop() {
        match queue_item {
            QueueItem::Play(name, fade, should_loop, per_channel_volume) => {
                start_one(
                    &name,
                    &model.sound_bank,
                    app.assets_path().expect("failed to fetch assets path"),
                    &mut model.clips_playing,
                    fade,
                    should_loop,
                    &model.stream,
                    per_channel_volume.unwrap_or(all_channels_equal(
                        model.stream.cpal_config().channels.into(),
                    )),
                )
                .expect("failed to start clip");
            }
            QueueItem::Stop(id, fade_out) => {
                if let Some((_index, clip)) =
                    get_clip_index_with_id_mut(&mut model.clips_playing, id)
                {
                    let fadeout_frames = millis_to_frames(fade_out.unwrap_or(0), clip.sample_rate);
                    info!(
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
            QueueItem::Remove(id) => {
                model
                    .stream
                    .send(move |audio| {
                        audio.remove_sound(id);
                    })
                    .unwrap();
                if let Some((index, _c)) = model
                    .clips_playing
                    .iter()
                    .enumerate()
                    .find(|(_i, c)| c.id == id)
                {
                    model.clips_playing.remove(index);
                } else {
                    panic!("Failed to find clip with ID {}", id);
                }
            }
        }
    }

    if model.tether.is_connected() {
        if let Some(instruction) = model.tether.check_messages() {
            match instruction {
                Instruction::Hit(clip_names) => {
                    for c in clip_names {
                        // TODO: get optional panning via instruction message
                        model
                            .action_queue
                            .push(QueueItem::Play(c, None, false, None));
                    }
                }
                Instruction::Add(clip_names, fade_duration) => {
                    for c in clip_names {
                        // TODO: get optional panning via instruction message
                        info!("Remote request to play clip named {}", &c);
                        model
                            .action_queue
                            .push(QueueItem::Play(c, fade_duration, false, None));
                    }
                }
                Instruction::Remove(clip_names, fade_duration) => {
                    for c in clip_names {
                        if let Some((_index, info)) =
                            get_clip_index_with_name(&model.clips_playing, &c)
                        {
                            info!("Remote request to remove (stop) clip named {}", &c);
                            model
                                .action_queue
                                .push(QueueItem::Stop(info.id, fade_duration));
                        } else {
                            error!("Could not find clip named {} to stop", c);
                        }
                    }
                }
                Instruction::Scene(clip_names, fade_duration) => {
                    let to_add = &clip_names;
                    info!("Scene transition: x{} clips to add", to_add.len());
                    for name in to_add {
                        // TODO: check at this point whether the clips exist (available)?
                        // TODO: get optional panning via instruction message
                        model.action_queue.push(QueueItem::Play(
                            String::from(name),
                            fade_duration,
                            true,
                            None,
                        ));
                    }
                    let to_remove = clips_to_remove(&model.clips_playing, &clip_names);
                    info!("Scene transition: x{} clips to remove", to_remove.len());
                    for id in to_remove {
                        model.action_queue.push(QueueItem::Stop(id, fade_duration));
                    }
                }
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
