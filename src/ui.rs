use log::info;
use nannou::prelude::*;
use nannou_egui::egui::{self, Slider};
use std::time::Duration;

use crate::{
    queue_stop_all,
    settings::Settings,
    utils::{clips_to_add, clips_to_remove, frames_to_seconds, get_clip_index_with_name},
    Model, QueueItem,
};

pub fn build_ui(model: &mut Model, since_start: Duration, _window_rect: Rect) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(since_start);
    let ctx = egui.begin_frame();

    let Settings {
        fadein_duration,
        fadeout_duration,
        ..
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
                                result.push_str(", ");
                            }
                            result.push_str(name);
                        }
                        result
                    };
                    ui.label(names);
                    if ui.button("load").clicked() {
                        let to_add = clips_to_add(&model.clips_playing, &s.clips);
                        info!("Scene transition: x{} clips to add", to_add.len());
                        for name in to_add {
                            model.action_queue.push(QueueItem::Play(
                                String::from(name),
                                Some(*fadein_duration),
                                true,
                            ));
                        }
                        let to_remove = clips_to_remove(&model.clips_playing, &s.clips);
                        info!("Scene transition: x{} clips to remove", to_remove.len());
                        for id in to_remove {
                            model
                                .action_queue
                                .push(QueueItem::Stop(id, Some(*fadeout_duration)));
                        }
                    }
                });
            }
            ui.horizontal(|ui| {
                ui.label("(None)");
                if ui.button("Stop all").clicked() {
                    queue_stop_all(&mut model.clips_playing, &mut model.action_queue, None);
                }
                if ui.button("Stop all (fade out)").clicked() {
                    queue_stop_all(
                        &mut model.clips_playing,
                        &mut model.action_queue,
                        Some(*fadeout_duration),
                    );
                }
            })
        });
    });
}
