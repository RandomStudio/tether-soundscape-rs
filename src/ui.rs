// use log::info;
// use nannou::prelude::*;
// use nannou_egui::egui::{self, Slider};
// use std::time::Duration;

use std::{fs::File, io::BufReader};

use egui::{ProgressBar, Ui};
use rodio::{Decoder, Sink};

use crate::{model::Model, playback::ClipWithSink};

pub fn render_local_controls(ui: &mut Ui, model: &mut Model) {
    ui.heading("Sample Bank");
    for sample in model.sound_bank.clips() {
        ui.horizontal(|ui| {
            ui.label(sample.name());
            if ui.button("once").clicked() {
                let clip_with_sink = ClipWithSink::new(
                    &sample,
                    &model.output_stream_handle,
                    String::from(sample.name()),
                );
                model.clips_playing.push(clip_with_sink);
            }
        });
    }
}

pub fn render_vis(ui: &mut Ui, model: &mut Model) {
    ui.heading(format!(
        "currently playing: x{} clips",
        model.clips_playing.len()
    ));
    for clip in model.clips_playing.iter() {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", clip.description()));
            ui.add(ProgressBar::new(clip.progress().unwrap_or(0.)));
        });
    }
}

// use crate::{
//     queue_stop_all,
//     settings::ManualSettings,
//     utils::{
//         clips_to_add, clips_to_remove, equalise_channel_volumes, frames_to_seconds,
//         get_clip_index_with_name, simple_panning_channel_volumes,
//     },
//     Model, QueueItem,
// };

// pub fn build_ui(model: &mut Model, since_start: Duration, is_multichannel_mode: bool) {
//     let egui = &mut model.egui;

//     egui.set_elapsed_time(since_start);
//     let ctx = egui.begin_frame();

//     let ManualSettings {
//         fadein_duration,
//         fadeout_duration,
//         simple_pan_position,
//         simple_pan_spread,
//         ignore_panning,
//         ..
//     } = &mut model.settings;

//     let output_channel_count = model
//         .output_stream_handle
//         .cpal_config()
//         .channels
//         .to_u32()
//         .unwrap();

//     egui::Window::new("Settings").show(&ctx, |ui| {
//         ui.horizontal(|ui| {
//             ui.label("Fade in duration");
//             ui.add(Slider::new(fadein_duration, 1..=10000).suffix("ms"));
//         });
//         ui.horizontal(|ui| {
//             ui.label("Fade out duration");
//             ui.add(Slider::new(fadeout_duration, 1..=10000).suffix("ms"));
//         });

//         ui.separator();

//         if is_multichannel_mode {
//             ui.collapsing("Multi-channel Panning", |ui| {
//                 ui.horizontal(|ui| {
//                     ui.checkbox(ignore_panning, "Equalise all channels (ignore panning)");
//                 });

//                 if !*ignore_panning {
//                     ui.horizontal(|ui| {
//                         ui.label("Pan position");
//                         let channel_count = model
//                             .output_stream_handle
//                             .cpal_config()
//                             .channels
//                             .to_f32()
//                             .unwrap()
//                             - 1.0;
//                         ui.add(Slider::new(simple_pan_position, 0. ..=channel_count));
//                     });
//                     ui.horizontal(|ui| {
//                         ui.label("Pan spread");
//                         let channel_count = model
//                             .output_stream_handle
//                             .cpal_config()
//                             .channels
//                             .to_f32()
//                             .unwrap();
//                         ui.add(Slider::new(simple_pan_spread, 1. ..=channel_count));
//                     });

//                     if ui.button("Calculate").clicked() {
//                         let per_channel_volume = simple_panning_channel_volumes(
//                             *simple_pan_position,
//                             *simple_pan_spread,
//                             output_channel_count,
//                         );
//                         println!("Mix: {:?}", per_channel_volume);
//                     }
//                 }
//             });
//         }

//         ui.heading("Clip triggers");
//         for c in model.sound_bank.clips() {
//             let duration_s = frames_to_seconds(c.frames_count(), c.sample_rate(), None);
//             let sample_rate = &format!("{}KHz", c.sample_rate().to_f32().unwrap() / 1000.);
//             ui.horizontal(|ui| {
//                 ui.label(format!("{} ({}s @{})", c.name(), duration_s, sample_rate));
//                 if ui.button("hit").clicked() {
//                     model.action_queue.push(QueueItem::Play(
//                         String::from(c.name()),
//                         None,
//                         false,
//                         if *ignore_panning {
//                             equalise_channel_volumes(output_channel_count)
//                         } else {
//                             simple_panning_channel_volumes(
//                                 *simple_pan_position,
//                                 *simple_pan_spread,
//                                 output_channel_count,
//                             )
//                         },
//                     ));
//                 }
//                 if ui.button("hit (fade in)").clicked() {
//                     model.action_queue.push(QueueItem::Play(
//                         String::from(c.name()),
//                         Some(*fadein_duration),
//                         false,
//                         if *ignore_panning {
//                             equalise_channel_volumes(output_channel_count)
//                         } else {
//                             simple_panning_channel_volumes(
//                                 *simple_pan_position,
//                                 *simple_pan_spread,
//                                 output_channel_count,
//                             )
//                         },
//                     ));
//                 }
//                 if ui.button("loop").clicked() {
//                     model.action_queue.push(QueueItem::Play(
//                         String::from(c.name()),
//                         None,
//                         true,
//                         if *ignore_panning {
//                             equalise_channel_volumes(output_channel_count)
//                         } else {
//                             simple_panning_channel_volumes(
//                                 *simple_pan_position,
//                                 *simple_pan_spread,
//                                 output_channel_count,
//                             )
//                         },
//                     ));
//                 }
//                 if ui.button("stop").clicked() {
//                     if let Some((_index, info)) =
//                         get_clip_index_with_name(&model.clips_playing, c.name())
//                     {
//                         model.action_queue.push(QueueItem::Stop(info.id, None));
//                     }
//                 }
//                 if ui.button("stop (fade out)").clicked() {
//                     if let Some((_index, info)) =
//                         get_clip_index_with_name(&model.clips_playing, c.name())
//                     {
//                         model
//                             .action_queue
//                             .push(QueueItem::Stop(info.id, Some(*fadeout_duration)));
//                     }
//                 }
//             });
//         }

//         ui.collapsing("Scenes", |ui| {
//             for s in model.sound_bank.scenes() {
//                 ui.horizontal(|ui| {
//                     let names = {
//                         let mut result = String::from("");
//                         for (i, name) in s.clips.iter().enumerate() {
//                             if i > 0 {
//                                 result.push_str(", ");
//                             }
//                             result.push_str(name);
//                         }
//                         result
//                     };
//                     ui.label(names);
//                     if ui.button("load").clicked() {
//                         let to_add = clips_to_add(&model.clips_playing, &s.clips);
//                         info!("Scene transition: x{} clips to add", to_add.len());
//                         for name in to_add {
//                             // TODO: get optional panning via instruction message
//                             model.action_queue.push(QueueItem::Play(
//                                 name,
//                                 Some(*fadein_duration),
//                                 true,
//                                 equalise_channel_volumes(output_channel_count),
//                             ));
//                         }
//                         let to_remove = clips_to_remove(&model.clips_playing, &s.clips);
//                         info!("Scene transition: x{} clips to remove", to_remove.len());
//                         for id in to_remove {
//                             model
//                                 .action_queue
//                                 .push(QueueItem::Stop(id, Some(*fadeout_duration)));
//                         }
//                     }
//                 });
//             }
//             ui.horizontal(|ui| {
//                 ui.label("(None)");
//                 if ui.button("Stop all").clicked() {
//                     queue_stop_all(&mut model.clips_playing, &mut model.action_queue, None);
//                 }
//                 if ui.button("Stop all (fade out)").clicked() {
//                     queue_stop_all(
//                         &mut model.clips_playing,
//                         &mut model.action_queue,
//                         Some(*fadeout_duration),
//                     );
//                 }
//             })
//         });
//     });
// }
