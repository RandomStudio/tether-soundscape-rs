use clap::Parser;

use env_logger::Env;
use log::info;

use rodio::OutputStream;
use std::time::Duration;
use ui::{render_local_controls, render_vis};

use settings::Cli;

use crate::model::Model;

mod loader;
mod model;
mod playback;
mod remote_control;
mod settings;
mod ui;
mod utils;

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level))
        .filter_module("paho_mqtt", log::LevelFilter::Warn)
        .init();

    let (_output_stream, stream_handle) = OutputStream::try_default().unwrap();

    let mut model = Model::new(&cli, stream_handle);

    if cli.headless_mode {
        info!("Running headless mode; Ctrl+C to quit");
        loop {
            model.internal_update();
            std::thread::sleep(Duration::from_millis(1));
        }
    } else {
        info!("Running graphics mode; close the window to quit");
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1280.0, 550.)),
            ..Default::default()
        };
        eframe::run_native(
            "Tether Remote Soundscape",
            options,
            Box::new(|_cc| Box::<Model>::new(model)),
        )
        .expect("Failed to launch GUI");
        info!("GUI ended; exit now...");
        std::process::exit(0);
    }
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: continuous mode essential?
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ctx.screen_rect();
            egui::Window::new("Local Control")
                .default_pos([rect.width() * 0.75, rect.height() / 2.])
                .min_width(320.0)
                .show(ctx, |ui| {
                    render_local_controls(ui, self);
                });
            render_vis(ui, self);
        });

        self.internal_update();
    }
}

//     if model.last_state_publish.elapsed().unwrap() > UPDATE_INTERVAL {
//         if let Some(remote_control) = &mut model.remote_control {
//             remote_control.publish_state(
//                 model.output_stream_handle.is_playing(),
//                 &model.clips_playing,
//                 &model.tether,
//             );
//         }
//     }
