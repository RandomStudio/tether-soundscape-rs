use nannou::prelude::*;
use nannou_egui::egui::{self, Slider};
use std::time::Duration;

use crate::{get_clip_index_with_name, utils::frames_to_seconds, Model, QueueItem};

pub const UPDATE_INTERVAL: Duration = Duration::from_millis(8);
pub const CLIP_HEIGHT: f32 = 15.;
pub const CLIP_WIDTH: f32 = 200.;
pub const SAMPLE_RATE: u32 = 96000;
pub const DEFAULT_FADEIN: u32 = 100;
pub const DEFAULT_FADEOUT: u32 = 2000;

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

        for (_index, c) in model.clips_available.iter().enumerate() {
            let duration_s = frames_to_seconds(c.frames_count(), c.sample_rate(), None);
            let sample_rate = &format!("{}KHz", c.sample_rate().to_f32().unwrap() / 1000.);
            ui.horizontal(|ui| {
                ui.label(format!("{} ({}s @{})", c.name(), duration_s, sample_rate));
                if ui.button("hit").clicked() {
                    model
                        .action_queue
                        .push(QueueItem::Play(String::from(c.name()), false));
                }
                if ui.button("loop").clicked() {
                    model
                        .action_queue
                        .push(QueueItem::Play(String::from(c.name()), true));
                }
                if ui.button("stop").clicked() {
                    if let Some((_index, info)) =
                        get_clip_index_with_name(&model.clips_playing, c.name())
                    {
                        model.action_queue.push(QueueItem::Stop(info.id));
                    }
                }
            });
        }
    });
}
