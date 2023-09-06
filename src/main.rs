use clap::Parser;

use env_logger::{Builder, Env};
use log::{debug, error, info, trace, warn};
use playback::ClipWithSink;
use remote_control::{Instruction, ScenePickMode};
// use remote_control::{Instruction, RemoteControl};
use rodio::{OutputStream, OutputStreamHandle};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tether_agent::TetherAgent;
use ui::{render_local_controls, render_vis};

use loader::{get_sound_asset_path, SoundBank};
// use playback::{
//     render_audio_multichannel, Audio, BufferedClip, CompleteUpdate, PlaybackState, ProgressUpdate,
//     RequestUpdate,
// };
use settings::{
    Cli, ManualSettings, LINE_THICKNESS, MIN_RADIUS, RING_BUFFER_SIZE, UPDATE_INTERVAL,
};
use tween::TweenTime;
// use ui::build_ui;
// use utils::{
//     equalise_channel_volumes, get_clip_index_with_id, get_clip_index_with_id_mut,
//     get_clip_index_with_name, get_duration_range, provided_or_default_panning,
// };

use crate::model::Model;

mod loader;
mod model;
mod playback;
mod remote_control;
mod settings;
mod ui;
// mod utils;

// pub struct CurrentlyPlayingClip {
//     id: usize,
//     name: String,
//     frames_count: u32,
//     sample_rate: u32,
//     state: PlaybackState,
//     current_volume: f32,
//     should_loop: bool,
//     last_update_sent: std::time::SystemTime,
// }

// impl CurrentlyPlayingClip {
//     pub fn length_in_frames(&self) -> u32 {
//         self.frames_count
//     }
//     pub fn length_in_millis(&self) -> u32 {
//         (self.frames_count.to_f32() / self.sample_rate.to_f32() / 1000.) as u32
//     }
// }

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level))
        .filter_module("paho_mqtt", log::LevelFilter::Warn)
        .init();

    let (_output_stream, stream_handle) = OutputStream::try_default().unwrap();

    info!("Running graphics mode; close the window to quit");
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 550.)),
        ..Default::default()
    };

    let model = Model::new(&cli, stream_handle);

    eframe::run_native(
        "Tether Remote Soundscape",
        options,
        Box::new(|_cc| Box::<Model>::new(model)),
    )
    .expect("Failed to launch GUI");
    info!("GUI ended; exit now...");
    std::process::exit(0);
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: continuous mode essential?
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ctx.screen_rect();
            egui::Window::new("Local Control")
                .default_pos([rect.width() * 0.75, rect.height() / 2.])
                .min_width(320.0)
                .show(ctx, |ui| {
                    render_local_controls(ui, self);
                });
            render_vis(ui, self);
        });

        // TODO: some (all?) of the logic/calls below can be made in a loop manually, when in text-mode
        if let Ok(_) = self.request_rx.try_recv() {
            // debug!("Received request rx");
            self.check_progress();
        }

        // Parse any remote control messages, which may generate CommandQueue items
        if let Some(remote_control) = &mut self.remote_control {
            while let Some((plug_name, message)) = self.tether.check_messages() {
                match remote_control.parse_instructions(&plug_name, &message) {
                    Ok(Instruction::Add(clip_name, should_loop, fade_ms, panning)) => {
                        self.action_queue.push(model::ActionQueueItem::Play(
                            clip_name,
                            match fade_ms {
                                Some(ms) => Some(Duration::from_millis(ms)),
                                None => None,
                            },
                            should_loop,
                            None,
                        ));
                        // self.play_one_clip(clip_name, should_loop, fade_ms);
                    }
                    Ok(Instruction::Remove(clip_name, fade_ms)) => {
                        for clip in self
                            .clips_playing
                            .iter_mut()
                            .filter(|x| x.name() == clip_name)
                        {
                            if let Some(ms) = fade_ms {
                                clip.fade_out(Duration::from_millis(ms));
                            } else {
                                clip.stop();
                            }
                        }
                    }
                    Ok(Instruction::Scene(scene_pick_mode, clip_names, fade_ms)) => {
                        match scene_pick_mode {
                            ScenePickMode::OnceAll => {
                                // TODO: check for
                                // - empty list (stop all)
                                for name in clip_names {
                                    self.action_queue.push(model::ActionQueueItem::Play(
                                        name,
                                        match fade_ms {
                                            Some(ms) => Some(Duration::from_millis(ms)),
                                            None => None,
                                        },
                                        false,
                                        None,
                                    ));
                                }
                            }
                            ScenePickMode::LoopAll => {
                                // TODO: check for
                                // - clips already playing (and LOOPING) (do not add)
                            }
                            ScenePickMode::Random => todo!(),
                        }
                    }
                    Err(_) => {
                        error!("Failed to parse remote Instruction");
                    }
                }
            }
        }
        while let Some(command) = self.action_queue.pop() {
            match command {
                model::ActionQueueItem::Play(clip_name, fade, should_loop, panning) => {
                    self.play_one_clip(clip_name, should_loop, fade);
                }
                model::ActionQueueItem::Stop(_, _) => todo!(),
                model::ActionQueueItem::Remove(_) => todo!(),
            };
        }
    }
}

impl Model {
    fn play_one_clip(&mut self, clip_name: String, should_loop: bool, fade: Option<Duration>) {
        if let Some(sample) = self
            .sound_bank
            .clips()
            .iter()
            .find(|x| x.name() == clip_name)
        {
            let clip_with_sink = ClipWithSink::new(
                &sample,
                should_loop,
                fade,
                &self.output_stream_handle,
                String::from(sample.name()),
            );
            self.clips_playing.push(clip_with_sink);
        } else {
            error!("Failed to find clip in bank with name, {}", clip_name);
        }
    }
}

// pub fn queue_stop_all(
//     clips_playing: &mut [CurrentlyPlayingClip],
//     action_queue: &mut Vec<QueueItem>,
//     fade: Option<u32>,
// ) {
//     for (_index, clip) in clips_playing.iter().enumerate() {
//         action_queue.push(QueueItem::Stop(clip.id, fade));
//     }
// }

// // TODO: reduce the number of arguments here. Use an enum
// // to encapsulate the "settings" for starting this clip?
// fn start_one(
//     name: &str,
//     sound_bank: &SoundBank,
//     assets_path: PathBuf,
//     clips_playing: &mut Vec<CurrentlyPlayingClip>,
//     fade: Option<u32>,
//     should_loop: bool,
//     stream: &audio::Stream<Audio>,
//     per_channel_volume: Vec<f32>,
// ) -> Result<(), ()> {
//     if let Some(clip_matched) = sound_bank
//         .clips()
//         .iter()
//         .find(|c| c.name().eq_ignore_ascii_case(name))
//     {
//         let path = get_sound_asset_path(assets_path, clip_matched.path());
//         if let Ok(reader) = audrey::open(Path::new(&path)) {
//             let id = get_highest_id(clips_playing);

//             info!(
//                 "Start playback for clip name {}, given playing ID #{}",
//                 clip_matched.name(),
//                 id
//             );
//             let new_clip = BufferedClip::new(
//                 id,
//                 Some((0., clip_matched.volume().unwrap_or(1.0), fade.unwrap_or(0))),
//                 reader,
//                 per_channel_volume,
//             );
//             clips_playing.push(CurrentlyPlayingClip {
//                 id,
//                 name: String::from(clip_matched.name()),
//                 frames_count: clip_matched.frames_count(),
//                 state: PlaybackState::Ready(),
//                 current_volume: 0.,
//                 sample_rate: clip_matched.sample_rate(),
//                 should_loop,
//                 last_update_sent: std::time::SystemTime::now(),
//             });
//             stream
//                 .send(move |audio| {
//                     audio.add_sound(new_clip);
//                 })
//                 .ok();
//             Ok(())
//         } else {
//             error!("No clip found with name {}", name);
//             Err(())
//         }
//     } else {
//         Err(())
//     }
// }

// fn key_pressed(_app: &App, model: &mut Model, key: Key) {
//     match key {
//         Key::Space => {
//             if model.output_stream_handle.is_paused() {
//                 model
//                     .output_stream_handle
//                     .play()
//                     .expect("failed to start stream");
//             } else {
//                 model
//                     .output_stream_handle
//                     .pause()
//                     .expect("failed to pause stream");
//             }
//         }
//         _ => {}
//     }
// }

// fn update(app: &App, model: &mut Model, update: Update) {
//     // let window = app.window(model.window_id).unwrap();

//     build_ui(model, update.since_start, model.multi_channel_mode);

//     // Note the while loop - we try to process ALL progress update messages
//     // every frame
//     while let Ok(receive) = model.rx_progress.pop() {
//         let (id, frames_played, current_volume) = receive;
//         if let Some(to_update) = get_clip_index_with_id(&model.clips_playing, id) {
//             let (index, _c) = to_update;
//             model.clips_playing[index].state = PlaybackState::Playing(frames_played);
//             model.clips_playing[index].current_volume = current_volume;
//         }
//     }

//     for sound in &mut model.clips_playing {
//         if sound.last_update_sent.elapsed().unwrap() > UPDATE_INTERVAL {
//             sound.last_update_sent = std::time::SystemTime::now();
//             trace!("Request for clip ID#{}", sound.id);
//             model
//                 .tx_request
//                 .push(sound.id)
//                 .expect("failed to send request");
//         }
//     }

//     if model.last_state_publish.elapsed().unwrap() > UPDATE_INTERVAL {
//         if let Some(remote_control) = &mut model.remote_control {
//             remote_control.publish_state(
//                 model.output_stream_handle.is_playing(),
//                 &model.clips_playing,
//                 &model.tether,
//             );
//         }
//     }

//     while let Ok(id) = model.rx_complete.pop() {
//         debug!("Complete state received for clip ID {}", id);
//         if let Some((_index, clip)) = get_clip_index_with_id(&model.clips_playing, id) {
//             if clip.should_loop {
//                 debug!("Should loop! Repeat clip with name {}", clip.name);
//                 model.action_queue.push(QueueItem::Play(
//                     String::from(&clip.name),
//                     None,
//                     true,
//                     equalise_channel_volumes(
//                         model.output_stream_handle.cpal_config().channels.into(),
//                     ), // TODO: get previous per-channel-volume
//                 ));
//             }
//             model.action_queue.push(QueueItem::Remove(id));
//         } else {
//             panic!("No match for clip id {}", id);
//         }
//     }

//     while let Some(queue_item) = model.action_queue.pop() {
//         match queue_item {
//             QueueItem::Play(name, fade, should_loop, per_channel_volume) => {
//                 start_one(
//                     &name,
//                     &model.sound_bank,
//                     app.assets_path().expect("failed to fetch assets path"),
//                     &mut model.clips_playing,
//                     fade,
//                     should_loop,
//                     &model.output_stream_handle,
//                     per_channel_volume,
//                 )
//                 .expect("failed to start clip");
//             }
//             QueueItem::Stop(id, fade_out) => {
//                 if let Some((_index, clip)) =
//                     get_clip_index_with_id_mut(&mut model.clips_playing, id)
//                 {
//                     let fadeout_frames = millis_to_frames(fade_out.unwrap_or(0), clip.sample_rate);
//                     info!(
//                         "Stop clip ID#{}: {}, fade out {}fr",
//                         id, &clip.name, fadeout_frames
//                     );
//                     clip.should_loop = false;
//                     model
//                         .output_stream_handle
//                         .send(move |audio| audio.fadeout_sound(id, fadeout_frames))
//                         .unwrap();
//                 }
//             }
//             QueueItem::Remove(id) => {
//                 model
//                     .output_stream_handle
//                     .send(move |audio| {
//                         audio.remove_sound(id);
//                     })
//                     .unwrap();
//                 if let Some((index, _c)) = model
//                     .clips_playing
//                     .iter()
//                     .enumerate()
//                     .find(|(_i, c)| c.id == id)
//                 {
//                     model.clips_playing.remove(index);
//                 } else {
//                     panic!("Failed to find clip with ID {}", id);
//                 }
//             }
//         }
//     }

//     if model.tether.is_connected() {
//         if let Some((plug_name, message)) = model.tether.check_messages() {
//             if let Some(remote_control) = &mut model.remote_control {
//                 match remote_control.parse_instructions(&plug_name, &message) {
//                     Ok(Instruction::Add(
//                         clip_name,
//                         should_loop,
//                         fade_duration,
//                         message_panning,
//                     )) => {
//                         if let Some(clip_matched) = model
//                             .sound_bank
//                             .clips()
//                             .iter()
//                             .find(|c| c.name().eq_ignore_ascii_case(&clip_name))
//                         {
//                             let clip_default_panning = clip_matched.panning();
//                             model.action_queue.push(QueueItem::Play(
//                                 clip_name,
//                                 fade_duration,
//                                 should_loop,
//                                 provided_or_default_panning(
//                                     message_panning,
//                                     clip_default_panning,
//                                     model.output_stream_handle.cpal_config().channels.into(),
//                                 ),
//                             ));
//                         } else {
//                             error!("Could not find clip named {} to play", &clip_name);
//                         }
//                     }
//                     Ok(Instruction::Remove(clip_name, fade_duration)) => {
//                         if let Some((_index, info)) =
//                             get_clip_index_with_name(&model.clips_playing, &clip_name)
//                         {
//                             model
//                                 .action_queue
//                                 .push(QueueItem::Stop(info.id, fade_duration));
//                         } else {
//                             error!("Could not find clip named {} to stop", &clip_name);
//                         }
//                     }
//                     Ok(Instruction::Scene(pick_mode, clip_names, fade_duration)) => {
//                         let to_add = &clip_names;
//                         info!("Scene transition: x{} clips to add", to_add.len());
//                         match pick_mode {
//                             ScenePickMode::LoopAll => {
//                                 for name in to_add {
//                                     if let Some(clip_matched) = model
//                                         .sound_bank
//                                         .clips()
//                                         .iter()
//                                         .find(|c| c.name().eq_ignore_ascii_case(&name))
//                                     {
//                                         let clip_default_panning = clip_matched.panning();
//                                         model.action_queue.push(QueueItem::Play(
//                                             String::from(name),
//                                             fade_duration,
//                                             true,
//                                             provided_or_default_panning(
//                                                 None,
//                                                 clip_default_panning,
//                                                 model
//                                                     .output_stream_handle
//                                                     .cpal_config()
//                                                     .channels
//                                                     .into(),
//                                             ),
//                                         ));
//                                     } else {
//                                         error!(
//                                             "Could not find clip named {} to play in scene",
//                                             name
//                                         );
//                                     }
//                                 }

//                                 let to_remove = clips_to_remove(&model.clips_playing, &clip_names);
//                                 info!("Scene transition: x{} clips to remove", to_remove.len());
//                                 for id in to_remove {
//                                     model.action_queue.push(QueueItem::Stop(id, fade_duration));
//                                 }
//                             }
//                             ScenePickMode::OnceAll => {
//                                 for name in to_add {
//                                     if let Some(clip_matched) = model
//                                         .sound_bank
//                                         .clips()
//                                         .iter()
//                                         .find(|c| c.name().eq_ignore_ascii_case(&name))
//                                     {
//                                         let clip_default_panning = clip_matched.panning();
//                                         model.action_queue.push(QueueItem::Play(
//                                             String::from(name),
//                                             fade_duration,
//                                             false,
//                                             provided_or_default_panning(
//                                                 None,
//                                                 clip_default_panning,
//                                                 model
//                                                     .output_stream_handle
//                                                     .cpal_config()
//                                                     .channels
//                                                     .into(),
//                                             ),
//                                         ));
//                                     } else {
//                                         error!(
//                                             "Could not find clip named {} to play in scene",
//                                             name
//                                         );
//                                     }
//                                 }

//                                 let to_remove = clips_to_remove(&model.clips_playing, &clip_names);
//                                 info!("Scene transition: x{} clips to remove", to_remove.len());
//                                 for id in to_remove {
//                                     model.action_queue.push(QueueItem::Stop(id, fade_duration));
//                                 }
//                             }
//                             ScenePickMode::Random => {
//                                 let random_clip_name = pick_random_clip(clip_names);
//                                 if let Some(clip_matched) = model
//                                     .sound_bank
//                                     .clips()
//                                     .iter()
//                                     .find(|c| c.name().eq_ignore_ascii_case(&random_clip_name))
//                                 {
//                                     let clip_default_panning = clip_matched.panning();
//                                     model.action_queue.push(QueueItem::Play(
//                                         String::from(clip_matched.name()),
//                                         fade_duration,
//                                         false,
//                                         provided_or_default_panning(
//                                             None,
//                                             clip_default_panning,
//                                             model
//                                                 .output_stream_handle
//                                                 .cpal_config()
//                                                 .channels
//                                                 .into(),
//                                         ),
//                                     ));
//                                 }
//                             }
//                         }
//                     }
//                     Err(_) => {
//                         error!("Failed to parse remote Instruction");
//                     }
//                 }
//             }
//         }
//     }
// }

// fn view(app: &App, model: &Model, frame: Frame) {
//     let draw = app.draw();

//     draw.background().color(DARKSLATEGREY);

//     let stream_state = if model.output_stream_handle.is_playing() {
//         format!("playing {} sounds", model.clips_playing.len())
//     } else {
//         String::from("paused")
//     };
//     draw.text(&stream_state).color(SLATEGREY);

//     let max_radius = app.window(model.window_id).unwrap().rect().h() / 2. * 0.9;
//     for (_i, c) in model.clips_playing.iter().enumerate() {
//         let x = 0.;

//         if let PlaybackState::Playing(frames_played) = c.state {
//             let opacity = c.current_volume;

//             let [min, max] = model.duration_range;
//             let radius = map_range(
//                 c.frames_count.to_f32(),
//                 min.to_f32(),
//                 max.to_f32(),
//                 MIN_RADIUS,
//                 max_radius,
//             );
//             let progress = frames_played.to_f32() / c.frames_count.to_f32();
//             let target_angle = PI * 2.0 * progress; // "percent" of full circle
//             let brightness = 0.5;

//             draw.ellipse()
//                 .radius(radius)
//                 .x_y(x, 0.)
//                 .no_fill()
//                 .stroke(rgba(brightness, brightness, brightness, opacity))
//                 .stroke_weight(LINE_THICKNESS * 2.);

//             let num_dots: usize = 1000;
//             let brightness = 1.0;
//             let white = rgba(brightness, brightness, brightness, opacity);
//             for dot in 0..num_dots {
//                 let angle = -map_range(dot.to_f32(), 0., num_dots.to_f32(), 0., target_angle);
//                 let x = radius * angle.cos();
//                 let dot_y = radius * angle.sin();
//                 draw.ellipse()
//                     .x_y(x, dot_y)
//                     .radius(LINE_THICKNESS)
//                     .color(white);
//             }
//             draw.text(&c.name).x_y(0., -radius - 15.).color(white);
//         }
//     }

//     draw.to_frame(app, &frame).unwrap();
//     model.egui.draw_to_frame(&frame).unwrap();
// }
