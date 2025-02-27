use std::time::Duration;

use egui::{Grid, Ui};

use crate::model::{ActionQueueItem, Model};

pub fn render_local_controls(ui: &mut Ui, model: &mut Model) {
    ui.heading("Local Control");

    Grid::new("clips_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for sample in model.sound_bank.clips() {
                ui.label(sample.name());
                ui.horizontal(|ui| {
                    if ui.button("once").clicked() {
                        model.action_queue.push(ActionQueueItem::Play(
                            sample.name().into(),
                            None,
                            None,
                            false,
                            None,
                        ));
                    }
                    if ui.button("once (fade 2s)").clicked() {
                        model.action_queue.push(ActionQueueItem::Play(
                            sample.name().into(),
                            None,
                            Some(Duration::from_secs(2)),
                            false,
                            None,
                        ));
                    }
                    if ui.button("loop").clicked() {
                        model.action_queue.push(ActionQueueItem::Play(
                            sample.name().into(),
                            None,
                            None,
                            true,
                            None,
                        ));
                    }
                    if ui.button("loop (fade 5s)").clicked() {
                        model.action_queue.push(ActionQueueItem::Play(
                            sample.name().into(),
                            None,
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
                // ui.label("-----");
                ui.end_row();
            }
        });
}
