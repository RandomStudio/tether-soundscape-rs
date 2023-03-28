use nannou::prelude::*;
use nannou_egui::egui::{self, Slider};
use std::time::Duration;

use crate::Model;

pub const UPDATE_INTERVAL: Duration = Duration::from_millis(8);
pub const CLIP_HEIGHT: f32 = 15.;
pub const CLIP_WIDTH: f32 = 200.;
pub const SAMPLE_RATE: u32 = 96000;

pub struct Settings {
    pub fadein_duration: usize,
}

pub fn build_ui(model: &mut Model, since_start: Duration, _window_rect: Rect) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(since_start);
    let ctx = egui.begin_frame();

    let Settings { fadein_duration } = &mut model.settings;

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Fade in duration");
            ui.add(Slider::new(fadein_duration, 1..=10000).suffix("ms"));
        })
    });
}
