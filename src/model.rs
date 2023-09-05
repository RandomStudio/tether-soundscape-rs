use clap::Parser;
use egui::Align2;
use log::{debug, info, warn};
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use env_logger::{Builder, Env};
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use tether_agent::TetherAgent;

use crate::{
    loader::SoundBank,
    playback::ClipWithSink,
    // CurrentlyPlayingClip, QueueItem,
    // remote_control::RemoteControl,
    settings::{Cli, ManualSettings, RING_BUFFER_SIZE},
    ui::{render_local_controls, render_vis},
};

pub type FadeDuration = u32;

pub struct Model {
    request_loop_handle: JoinHandle<()>,
    // pub request_channel: (Sender<()>, Receiver<()>),
    pub request_rx: Receiver<()>,
    pub output_stream_handle: OutputStreamHandle,
    pub sound_bank: SoundBank,
    pub clips_playing: Vec<ClipWithSink>,
    // clips_playing: Vec<CurrentlyPlayingClip>,
    // duration_range: [FadeDuration; 2],
    // action_queue: Vec<QueueItem>,
    pub last_state_publish: SystemTime,
    pub settings: ManualSettings,
    // multi_channel_mode: bool,
    pub tether: TetherAgent,
    // remote_control: Option<RemoteControl>,
}

impl Model {
    pub fn new(cli: &Cli, output_stream_handle: OutputStreamHandle) -> Model {
        let settings = ManualSettings::defaults();

        if cli.multichannel_mode {
            info!("Multichannel mode; use mono clips only");
        } else {
            info!("Stereo playback mode enabled; will disable multi-channel panning features");
        }

        // let mode = if cli.multichannel_mode {
        //     "MULTICHANNEL"
        // } else {
        //     "STEREO"
        // };

        let sound_bank = SoundBank::new(Path::new("test_stereo.json"), false);
        // let duration_range = get_duration_range(sound_bank.clips());

        let tether = TetherAgent::new("soundscape", None, None);
        if !cli.tether_disable {
            tether.connect().expect("Failed to connect to Tether");
        } else {
            warn!("Tether connection disabled")
        }

        // let remote_control = if cli.tether_disable {
        //     None
        // } else {
        //     Some(RemoteControl::new(&tether))
        // };

        let (tx, rx) = mpsc::channel();

        // let request_tx = tx.clone();

        let request_loop_handle = thread::spawn(move || loop {
            tx.send(()).expect("failed to send via channel");
            // debug!("tx request");
            // TODO: the interval below should be configured
            thread::sleep(Duration::from_millis(16));
        });

        Model {
            // request_channel: (tx, rx),
            request_rx: rx,
            request_loop_handle,
            output_stream_handle,
            sound_bank,
            clips_playing: Vec::new(),
            // action_queue: Vec::new(),
            last_state_publish: std::time::SystemTime::now(),
            settings,
            tether,
            // remote_control,
        }
    }

    pub fn check_progress(&mut self) {
        for clip in &mut self.clips_playing {
            clip.update_progress();
        }
        let completed = self.clips_playing.iter().position(|x| x.is_completed());
        if let Some(i) = completed {
            debug!("Removing clip index {}", i);
            self.clips_playing.remove(i);
        }
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

        // TODO: this call can be made in a loop manually, when in text-mode
        if let Ok(_) = self.request_rx.try_recv() {
            // debug!("Received request rx");
            self.check_progress();
        }
    }
}
