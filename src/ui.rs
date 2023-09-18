// use log::info;
// use nannou::prelude::*;
// use nannou_egui::egui::{self, Slider};
// use std::time::Duration;

use std::time::{Duration, SystemTime};

use egui::{Color32, ProgressBar, RichText, Ui};

use crate::model::{ActionQueueItem, MessageStats, Model};

pub fn render_local_controls(ui: &mut Ui, model: &mut Model) {
    ui.heading("Sample Bank");
    for sample in model.sound_bank.clips() {
        ui.horizontal(|ui| {
            ui.label(sample.name());
            if ui.button("once").clicked() {
                // let clip_with_sink = ClipWithSink::new(
                //     model.clips_playing.len(),
                //     &sample,
                //     false,
                //     None,
                //     None,
                //     &model.output_stream_handle,
                //     model.output_channels_used,
                // );
                // model.clips_playing.push(clip_with_sink);
                model.action_queue.push(ActionQueueItem::Play(
                    sample.name().into(),
                    None,
                    false,
                    None,
                ));
            }
            if ui.button("once (fade 2s)").clicked() {
                // let clip_with_sink = ClipWithSink::new(
                //     model.clips_playing.len(),
                //     &sample,
                //     false,
                //     Some(Duration::from_millis(2000)),
                //     None,
                //     &model.output_stream_handle,
                //     model.output_channels_used,
                // );
                // model.clips_playing.push(clip_with_sink);
                model.action_queue.push(ActionQueueItem::Play(
                    sample.name().into(),
                    Some(Duration::from_secs(2)),
                    false,
                    None,
                ));
            }
            if ui.button("loop").clicked() {
                // let clip_with_sink = ClipWithSink::new(
                //     model.clips_playing.len(),
                //     &sample,
                //     true,
                //     None,
                //     None,
                //     &model.output_stream_handle,
                //     model.output_channels_used,
                // );
                // model.clips_playing.push(clip_with_sink);
                model.action_queue.push(ActionQueueItem::Play(
                    sample.name().into(),
                    None,
                    true,
                    None,
                ));
            }
            if ui.button("loop (fade 5s)").clicked() {
                // let clip_with_sink = ClipWithSink::new(
                //     model.clips_playing.len(),
                //     &sample,
                //     true,
                //     Some(Duration::from_secs(5)),
                //     None,
                //     &model.output_stream_handle,
                //     model.output_channels_used,
                // );
                // model.clips_playing.push(clip_with_sink);
                model.action_queue.push(ActionQueueItem::Play(
                    sample.name().into(),
                    Some(Duration::from_secs(5)),
                    true,
                    None,
                ));
            }
            if ui.button("stop").clicked() {
                for clip in &model.clips_playing {
                    if clip.name() == sample.name() {
                        model
                            .action_queue
                            .push(ActionQueueItem::Stop(clip.id(), None));
                    }
                }
            }
            if ui.button("fade out(2s)").clicked() {
                for clip in &mut model.clips_playing {
                    if clip.name() == sample.name() {
                        model.action_queue.push(ActionQueueItem::Stop(
                            clip.id(),
                            Some(Duration::from_secs(2)),
                        ));
                    }
                }
            }
        });
    }
}

pub fn render_vis(ui: &mut Ui, model: &mut Model) {
    ui.heading("Status");
    if model.tether_disabled {
        ui.label(RichText::new("Tether disabled 🚫").color(Color32::YELLOW));
    } else {
        if model.tether.is_connected() {
            ui.label(RichText::new("Tether connected ✔").color(Color32::GREEN));
        } else {
            ui.label(RichText::new("Tether not (yet) connected x").color(Color32::RED));
        }
        ui.horizontal(|ui| {
            ui.label("Output channels in use:");
            ui.label(RichText::new(format!("x{}", model.output_channels_used)).strong());
        });
    }
    // Message stats
    let MessageStats {
        last_clip_message,
        last_events_message,
        last_scene_message,
        last_state_message,
    } = model.message_stats;
    ui.horizontal(|ui| {
        ui.label("Clip messages IN");
        ui.label(RichText::new("⏺").color(colour_by_elapsed(last_clip_message)));
    });
    ui.horizontal(|ui| {
        ui.label("Scene messages IN");
        ui.label(RichText::new("⏺").color(colour_by_elapsed(last_scene_message)));
    });
    ui.horizontal(|ui| {
        ui.label("State messages OUT");
        ui.label(RichText::new("⏺").color(colour_by_elapsed(last_state_message)));
    });
    ui.horizontal(|ui| {
        ui.label("Event messages OUT");
        ui.label(RichText::new("⏺").color(colour_by_elapsed(last_events_message)));
    });

    ui.separator();

    ui.heading(format!("Playing: x{} clips", model.clips_playing.len()));
    for clip in model.clips_playing.iter() {
        ui.horizontal(|ui| {
            ui.label(format!("#{}: {}", clip.id(), clip.name()));
            if clip.is_looping() {
                ui.label("🔁");
            }
            if ui.button("🗑").clicked() {
                model
                    .action_queue
                    .push(ActionQueueItem::Stop(clip.id(), None));
            }
            let brightness: u8 = (clip.current_volume() * 255.) as u8;
            let c = Color32::from_rgb(0, 0, brightness);
            ui.add(
                ProgressBar::new(clip.progress().unwrap_or(0.))
                    .show_percentage()
                    .fill(c),
            );
        });
    }
}

fn colour_by_elapsed(last_time: Option<SystemTime>) -> Color32 {
    match last_time {
        None => Color32::DARK_RED,
        Some(t) => match t.elapsed().expect("elapsed fail").as_millis() {
            0..=1000 => Color32::GREEN,
            1001..=3000 => Color32::YELLOW,
            _ => Color32::GRAY,
        },
    }
}
