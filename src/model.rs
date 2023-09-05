use clap::Parser;
use log::{debug, info, warn};
use std::{path::Path, time::SystemTime};

use env_logger::{Builder, Env};
use rodio::OutputStreamHandle;
use tether_agent::TetherAgent;

use crate::{
    loader::SoundBank,
    // remote_control::RemoteControl,
    settings::{Cli, ManualSettings, RING_BUFFER_SIZE},
    // CurrentlyPlayingClip, QueueItem,
};

pub type FadeDuration = u32;

pub struct Model {
    // rx_progress: Consumer<ProgressUpdate>,
    // rx_complete: Consumer<CompleteUpdate>,
    // tx_request: Producer<RequestUpdate>,
    output_stream_handle: OutputStreamHandle,
    sound_bank: SoundBank,
    // clips_playing: Vec<CurrentlyPlayingClip>,
    // duration_range: [FadeDuration; 2],
    // action_queue: Vec<QueueItem>,
    last_state_publish: SystemTime,
    settings: ManualSettings,
    // multi_channel_mode: bool,
    tether: TetherAgent,
    // remote_control: Option<RemoteControl>,
}

impl Model {
    pub fn new(cli: &Cli, output_stream_handle: OutputStreamHandle) -> Model {
        let settings = ManualSettings::defaults();

        // let device = match cli.preferred_output_device {
        //     Some(device_name) => {
        //         if let Ok(devices) = audio_host.output_devices() {
        //             let mut matching_device: Option<Device> = None;
        //             for d in devices {
        //                 debug!("output device: {:?}", d.name());
        //                 if d.name().unwrap() == device_name {
        //                     info!(
        //                         "Found matching device {} == {}",
        //                         &d.name().unwrap(),
        //                         &device_name
        //                     );
        //                     matching_device = Some(d);
        //                 }
        //             }
        //             matching_device
        //         } else {
        //             panic!("Failed to enumerate host audio devices");
        //         }
        //     }
        //     None => audio_host.default_output_device(),
        // };

        // let (tx_progress, rx_progress) = RingBuffer::new(RING_BUFFER_SIZE * 16);
        // let (tx_complete, rx_complete) = RingBuffer::new(RING_BUFFER_SIZE);
        // let (tx_request, rx_request) = RingBuffer::new(RING_BUFFER_SIZE);

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

        Model {
            output_stream_handle,
            sound_bank,
            // clips_playing: Vec::new(),
            // action_queue: Vec::new(),
            last_state_publish: std::time::SystemTime::now(),
            settings,
            tether,
            // remote_control,
        }
    }
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // // TODO: continuous mode essential?
        // ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("egui");
        });
    }
}
