use dioxus::prelude::*;

use env_logger::{Builder, Env};
use loader::SoundBank;
use log::{debug, info};
use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use clap::Parser;

use crate::settings::{Cli, ManualSettings};

// use env_logger::{Builder, Env};
// use log::{debug, error, info, trace, warn};
// use remote_control::{Instruction, RemoteControl};
// use tether_agent::TetherAgent;

// use loader::{get_sound_asset_path, SoundBank};
// use settings::{
//     Cli, ManualSettings, LINE_THICKNESS, MIN_RADIUS, RING_BUFFER_SIZE, UPDATE_INTERVAL,
// };
// use tween::TweenTime;

mod loader;
mod settings;

pub type SimplePanning = (f32, f32);

// // mod playback;
// mod remote_control;
// // mod ui;
// // mod utils;

// pub type FadeDuration = u32;

// pub struct Model {
//     sound_bank: SoundBank,
//     // clips_playing: Vec<CurrentlyPlayingClip>,
//     // duration_range: [FadeDuration; 2],
//     // action_queue: Vec<QueueItem>,
//     // last_state_publish: SystemTime,
//     // settings: ManualSettings,
//     // multi_channel_mode: bool,
//     tether: TetherAgent,
//     remote_control: RemoteControl,
// }
// pub enum QueueItem {
//     /// Start playback: name, optional fade duration in ms, should_loop,
//     /// finalised per-channel-volume
//     Play(String, Option<FadeDuration>, bool, Vec<f32>),
//     /// Stop/fade out: id in currently_playing Vec, optional fade duration in ms
//     Stop(usize, Option<FadeDuration>),
//     /// Remove clip: id in currently_playing Vec
//     Remove(usize),
// }

// pub struct CurrentlyPlayingClip {
//     id: usize,
//     name: String,
//     frames_count: u32,
//     sample_rate: u32,
//     state: PlaybackState,
//     current_volume: f32,
//     should_loop: bool,
//     last_update_sent: std::time::SystemTime,
// }

// impl CurrentlyPlayingClip {
//     pub fn length_in_frames(&self) -> u32 {
//         self.frames_count
//     }
//     pub fn length_in_millis(&self) -> u32 {
//         (self.frames_count.to_f32() / self.sample_rate.to_f32() / 1000.)
//             .to_u32()
//             .unwrap()
//     }
// // }

// struct Model {
//     source: Decoder<BufReader<File>>,
//     stream_handle: OutputStreamHandle,
// }

// impl Model {
//     pub fn new() -> Self {
//         Model {
//             source,
//             stream_handle,
//         }
//     }
// }

fn main() {
    let cli = Cli::parse();

    let mut builder = Builder::from_env(Env::default().default_filter_or(&cli.log_level));
    builder.filter_module("wgpu_core", log::LevelFilter::Error);
    builder.filter_module("wgpu_hal", log::LevelFilter::Warn);
    builder.filter_module("naga", log::LevelFilter::Warn);
    builder.filter_module("paho_mqtt", log::LevelFilter::Warn);
    builder.init();
    info!("Started; args: {:?}", cli);
    debug!("Debugging is enabled; could be verbose");

    // let settings = ManualSettings::defaults();

    if cli.text_mode {
        info!("TUI text-mode enabled; low graphics");
        dioxus_tui::launch(app);
    } else {
        info!("Full graphics mode enabled");
        dioxus_desktop::launch(app);
    }
}

pub fn app(cx: Scope) -> Element {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();

    let stream_handle_saved = use_ref(cx, || stream_handle);

    // Although we don't use Stream, we MUST retain a reference to it somewhere, or it will be silently
    // dropped, along with the OutputStreamHandle, before we try to use it!
    // See https://github.com/RustAudio/rodio/issues/330
    let _stream_saved = use_ref(cx, || stream);

    let sound_bank = use_state(cx, || SoundBank::new(Path::new("test_stereo.json"), false));

    // let play_sound = ;

    cx.render(rsx!(
        div {
            display: "flex",
            flex_direction: "column",
            width: "100%",
            height: "100%",

        sound_bank.clips().iter().map(|clip| {
            let label = format!("Play {}", clip.name());
            rsx!(
                    div {
            display: "flex",
            flex_direction: "row",
                        button {
                        onclick: |_| {
                            // println!("Playing {} ...", clip.path());
                            let file = BufReader::new(File::open(clip.path()).unwrap());
                            let source = Decoder::new(file).unwrap();
                            stream_handle_saved.with(|stream_ref| {
                                stream_ref
                                    .play_raw(source.convert_samples())
                                    .expect("failed to play");
                            });
                        },
                        label
                    }
                    }
            )
        })
    }))
}

// fn model(app: &App) -> Model {
//     let cli = Cli::parse();

//     let mut builder = Builder::from_env(Env::default().default_filter_or(&cli.log_level));
//     builder.filter_module("wgpu_core", log::LevelFilter::Error);
//     builder.filter_module("wgpu_hal", log::LevelFilter::Warn);
//     builder.filter_module("naga", log::LevelFilter::Warn);
//     builder.filter_module("paho_mqtt", log::LevelFilter::Warn);
//     builder.init();
//     info!("Started; args: {:?}", cli);
//     debug!("Debugging is enabled; could be verbose");

//     let settings = ManualSettings::defaults();

//     // Initialise the audio host so we can spawn an audio stream.
//     // let audio_host = audio::Host::new();

//     // let device = match cli.preferred_output_device {
//     //     Some(device_name) => {
//     //         if let Ok(devices) = audio_host.output_devices() {
//     //             let mut matching_device: Option<Device> = None;
//     //             for d in devices {
//     //                 debug!("output device: {:?}", d.name());
//     //                 if d.name().unwrap() == device_name {
//     //                     info!(
//     //                         "Found matching device {} == {}",
//     //                         &d.name().unwrap(),
//     //                         &device_name
//     //                     );
//     //                     matching_device = Some(d);
//     //                 }
//     //             }
//     //             matching_device
//     //         } else {
//     //             panic!("Failed to enumerate host audio devices");
//     //         }
//     //     }
//     //     None => audio_host.default_output_device(),
//     // };

//     // let (tx_progress, rx_progress) = RingBuffer::new(RING_BUFFER_SIZE * 16);
//     // let (tx_complete, rx_complete) = RingBuffer::new(RING_BUFFER_SIZE);
//     // let (tx_request, rx_request) = RingBuffer::new(RING_BUFFER_SIZE);
//     // let audio_model = Audio::new(tx_progress, tx_complete, rx_request);

//     if cli.multichannel_mode {
//         info!("Multichannel mode; use mono clips only");
//     } else {
//         info!("Stereo playback mode enabled; will disable multi-channel panning features");
//     }

//     // // Initialise the state that we want to live on the audio thread.
//     // let stream = audio_host
//     //     .new_output_stream(audio_model)
//     //     .render(if cli.multichannel_mode {
//     //         render_audio_multichannel
//     //     } else {
//     //         render_audio_stereo
//     //     })
//     //     .device(device.unwrap())
//     //     .sample_rate(cli.sample_rate)
//     //     .build()
//     //     .unwrap();

//     let mode = if cli.multichannel_mode {
//         "MULTICHANNEL"
//     } else {
//         "STEREO"
//     };

//     // let sound_bank = SoundBank::new(
//     //     app,
//     //     Path::new(
//     //         &cli.sample_bank_path
//     //             .unwrap_or(pick_default_sample_bank(cli.multichannel_mode)),
//     //     ),
//     //     cli.multichannel_mode,
//     // );
//     // let duration_range = get_duration_range(sound_bank.clips());

//     let tether = TetherAgent::new("soundscape", None, None);
//     if !cli.tether_disable {
//         tether.connect().expect("Failed to connect to Tether");
//     } else {
//         warn!("Tether connection disabled")
//     }

//     let remote_state_update = RemoteControl::new(&tether);

//     Model {
//         sound_bank,
//         clips_playing: Vec::new(),
//         duration_range,
//         rx_progress,
//         rx_complete,
//         tx_request,
//         action_queue: Vec::new(),
//         settings,
//         tether,
//         last_state_publish: std::time::SystemTime::now(),
//         multi_channel_mode: cli.multichannel_mode,
//         remote_control: remote_state_update,
//     }
// }
