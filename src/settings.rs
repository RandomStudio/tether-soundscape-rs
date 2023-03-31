use nannou::prelude::*;
use nannou_egui::egui::{self, Slider};
use std::time::Duration;

use crate::{get_clip_index_with_name, utils::frames_to_seconds, Model, QueueItem};

pub const UPDATE_INTERVAL: Duration = Duration::from_millis(16);
pub const MIN_RADIUS: f32 = 100.;
pub const LINE_THICKNESS: f32 = 2.;
pub const DEFAULT_FADEIN: u32 = 2000;
pub const DEFAULT_FADEOUT: u32 = 2000;
pub const RING_BUFFER_SIZE: usize = 32;

pub struct Settings {
    pub fadein_duration: u32,
    pub fadeout_duration: u32,
}

pub fn build_ui(model: &mut Model, since_start: Duration, _window_rect: Rect) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(since_start);
    let ctx = egui.begin_frame();

    let Settings {
        fadein_duration,
        fadeout_duration,
    } = &mut model.settings;

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Fade in duration");
            ui.add(Slider::new(fadein_duration, 1..=10000).suffix("ms"));
        });
        ui.horizontal(|ui| {
            ui.label("Fade out duration");
            ui.add(Slider::new(fadeout_duration, 1..=10000).suffix("ms"));
        });

        ui.separator();

        ui.collapsing("Clip triggers", |ui| {
            for c in model.sound_bank.clips() {
                let duration_s = frames_to_seconds(c.frames_count(), c.sample_rate(), None);
                let sample_rate = &format!("{}KHz", c.sample_rate().to_f32().unwrap() / 1000.);
                ui.horizontal(|ui| {
                    ui.label(format!("{} ({}s @{})", c.name(), duration_s, sample_rate));
                    if ui.button("hit").clicked() {
                        model.action_queue.push(QueueItem::Play(
                            String::from(c.name()),
                            None,
                            false,
                        ));
                    }
                    if ui.button("hit (fade in)").clicked() {
                        model.action_queue.push(QueueItem::Play(
                            String::from(c.name()),
                            Some(*fadein_duration),
                            false,
                        ));
                    }
                    if ui.button("loop").clicked() {
                        model.action_queue.push(QueueItem::Play(
                            String::from(c.name()),
                            None,
                            true,
                        ));
                    }
                    if ui.button("stop").clicked() {
                        if let Some((_index, info)) =
                            get_clip_index_with_name(&model.clips_playing, c.name())
                        {
                            model.action_queue.push(QueueItem::Stop(info.id, None));
                        }
                    }
                    if ui.button("stop (fade out)").clicked() {
                        if let Some((_index, info)) =
                            get_clip_index_with_name(&model.clips_playing, c.name())
                        {
                            model
                                .action_queue
                                .push(QueueItem::Stop(info.id, Some(*fadeout_duration)));
                        }
                    }
                });
            }
        });

        ui.collapsing("Scenes", |ui| {
            for s in model.sound_bank.scenes() {
                ui.horizontal(|ui| {
                    let names = {
                        let mut result = String::from("");
                        for (i, name) in s.clips.iter().enumerate() {
                            if i > 0 {
                                result.push_str(",");
                            }
                            result.push_str(&name);
                        }
                        result
                    };
                    ui.label(names);
                    if ui.button("load").clicked() {
                        for name in &s.clips {
                            model.action_queue.push(QueueItem::Play(
                                String::from(name),
                                Some(*fadein_duration),
                                true,
                            ));
                        }
                    }
                });
            }
            ui.horizontal(|ui| {
                ui.label("(None)");
                if ui.button("Stop all").clicked() {
                    for (index, _clip) in model.clips_playing.iter().enumerate() {
                        model.action_queue.push(QueueItem::Stop(index, None));
                    }
                }
                if ui.button("Stop all (fade out)").clicked() {
                    for (index, _clip) in model.clips_playing.iter().enumerate() {
                        model
                            .action_queue
                            .push(QueueItem::Stop(index, Some(*fadeout_duration)));
                    }
                }
            })
        });
    });
}
