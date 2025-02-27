use egui::{Color32, ProgressBar, RichText, Ui};

use crate::model::{ActionQueueItem, Model};

pub fn render_clips_section(ui: &mut Ui, model: &mut Model) {
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
            if clip.is_paused() {
                ui.label(RichText::new("Paused").color(Color32::GRAY));
            } else {
                ui.add(
                    ProgressBar::new(clip.progress().unwrap_or(0.))
                        .show_percentage()
                        .fill(c),
                );
            }
        });
    }
}
