use std::time::SystemTime;

use egui::{Color32, RichText, Ui};

use crate::model::{MessageStats, Model};

pub fn render_status_section(ui: &mut Ui, model: &mut Model) {
    ui.heading("Status");

    ui.columns(3, |cols| {
        let ui = &mut cols[0];
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

        let ui = &mut cols[0];
        // Message stats
        let MessageStats {
            last_clip_message,
            last_events_message,
            last_global_control_message,
            last_scene_message,
            last_state_message,
        } = model.message_stats;
        let ui = &mut cols[1];
        ui.horizontal(|ui| {
            ui.label("Clip messages IN");
            ui.label(RichText::new("⏺").color(colour_by_elapsed(last_clip_message)));
        });
        ui.horizontal(|ui| {
            ui.label("Scene messages IN");
            ui.label(RichText::new("⏺").color(colour_by_elapsed(last_scene_message)));
        });
        ui.horizontal(|ui| {
            ui.label("Global Control messages IN");
            ui.label(RichText::new("⏺").color(colour_by_elapsed(last_global_control_message)));
        });

        let ui = &mut cols[2];
        ui.horizontal(|ui| {
            ui.label("State messages OUT");
            ui.label(RichText::new("⏺").color(colour_by_elapsed(last_state_message)));
        });
        ui.horizontal(|ui| {
            ui.label("Event messages OUT");
            ui.label(RichText::new("⏺").color(colour_by_elapsed(last_events_message)));
        });
    });
}

fn colour_by_elapsed(last_time: Option<SystemTime>) -> Color32 {
    match last_time {
        None => Color32::DARK_GRAY,
        Some(t) => match t.elapsed().expect("elapsed fail").as_millis() {
            0..=1000 => Color32::GREEN,
            1001..=3000 => Color32::YELLOW,
            _ => Color32::GRAY,
        },
    }
}
