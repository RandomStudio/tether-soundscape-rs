use clap::Parser;

use env_logger::Env;
use log::{debug, error, info};
use model::ActionQueueItem;
use remote_control::{Instruction, ScenePickMode};

use rodio::OutputStream;
use std::time::Duration;
use ui::{render_local_controls, render_vis};
use utils::{optional_ms_to_duration, pick_random_clip};

use settings::Cli;

use crate::model::Model;

mod loader;
mod model;
mod playback;
mod remote_control;
mod settings;
mod ui;
mod utils;

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
                    Ok(Instruction::Add(clip_name, should_loop, fade_ms, _panning)) => {
                        self.action_queue.push(ActionQueueItem::Play(
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
                                if clip_names.len() == 0 {
                                    debug!("Empty scene list; stop all currently playing");
                                    for clip in &self.clips_playing {
                                        self.action_queue.push(ActionQueueItem::Stop(
                                            clip.id(),
                                            optional_ms_to_duration(fade_ms),
                                        ))
                                    }
                                } else {
                                    for name in clip_names {
                                        self.action_queue.push(ActionQueueItem::Play(
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
                            }
                            ScenePickMode::LoopAll => {
                                // TODO: check for
                                // - empty list (stop all)
                                // - clips already playing (and LOOPING) (do not add)
                                if clip_names.len() == 0 {
                                    debug!("Empty scene list; stop all currently playing that are looping");
                                    for clip in &self.clips_playing {
                                        self.action_queue.push(ActionQueueItem::Stop(
                                            clip.id(),
                                            optional_ms_to_duration(fade_ms),
                                        ))
                                    }
                                } else {
                                    let to_add = clip_names.iter().filter(|candidate| {
                                        self.clips_playing
                                            .iter()
                                            .find(|playing| {
                                                playing.name().eq_ignore_ascii_case(&candidate)
                                            })
                                            .is_none()
                                    });
                                    let to_remove = self.clips_playing.iter().filter(|playing| {
                                        clip_names
                                            .iter()
                                            .find(|requested| {
                                                requested.eq_ignore_ascii_case(&playing.name())
                                            })
                                            .is_none()
                                    });
                                    for name in to_add {
                                        self.action_queue.push(ActionQueueItem::Play(
                                            name.into(),
                                            optional_ms_to_duration(fade_ms),
                                            true,
                                            None,
                                        ));
                                    }
                                    for clip in to_remove {
                                        self.action_queue.push(ActionQueueItem::Stop(
                                            clip.id(),
                                            optional_ms_to_duration(fade_ms),
                                        ));
                                    }
                                }
                            }
                            ScenePickMode::OnceRandomSinglePick => {
                                let pick_name = pick_random_clip(clip_names);
                                self.action_queue.push(ActionQueueItem::Play(
                                    pick_name,
                                    optional_ms_to_duration(fade_ms),
                                    false,
                                    None,
                                ));
                            }
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
                ActionQueueItem::Play(clip_name, fade, should_loop, _panning) => {
                    self.play_one_clip(clip_name, should_loop, fade);
                }
                ActionQueueItem::Stop(id, fade) => {
                    if let Some(clip) = self.clips_playing.iter_mut().find(|x| x.id() == id) {
                        match fade {
                            Some(duration) => clip.fade_out(duration),
                            None => clip.stop(),
                        };
                    }
                }
            };
        }
    }
}

//     if model.last_state_publish.elapsed().unwrap() > UPDATE_INTERVAL {
//         if let Some(remote_control) = &mut model.remote_control {
//             remote_control.publish_state(
//                 model.output_stream_handle.is_playing(),
//                 &model.clips_playing,
//                 &model.tether,
//             );
//         }
//     }
