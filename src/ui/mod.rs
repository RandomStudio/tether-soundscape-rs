mod clips_section;
mod local_controls;
mod status_section;

use clips_section::render_clips_section;
use local_controls::render_local_controls;
use status_section::render_status_section;

use crate::model::Model;

pub fn render_gui(ctx: &egui::Context, model: &mut Model) {
    egui::TopBottomPanel::top("status").show(ctx, |ui| {
        render_status_section(ui, model);
    });
    egui::SidePanel::right("local_control").show(ctx, |ui| {
        render_local_controls(ui, model);
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        render_clips_section(ui, model);
    });
}
