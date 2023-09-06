use log::{debug, error, info, warn};
use std::{
    path::Path,
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use rodio::OutputStreamHandle;
use tether_agent::TetherAgent;

use crate::{
    loader::SoundBank,
    playback::ClipWithSink,
    remote_control::{PanWithRange, RemoteControl},
    // CurrentlyPlayingClip, QueueItem,
    // remote_control::RemoteControl,
    settings::{Cli, ManualSettings},
};

pub enum ActionQueueItem {
    /// Start playback: name, optional fade duration, should_loop,
    /// optional per-channel-volume
    Play(String, Option<Duration>, bool, Option<Vec<PanWithRange>>),
    /// Stop/fade out: id in currently_playing Vec, optional fade duration
    Stop(usize, Option<Duration>),
}

pub struct Model {
    request_loop_handle: JoinHandle<()>,
    // pub request_channel: (Sender<()>, Receiver<()>),
    pub request_rx: Receiver<()>,
    pub output_stream_handle: OutputStreamHandle,
    pub sound_bank: SoundBank,
    pub clips_playing: Vec<ClipWithSink>,
    // clips_playing: Vec<CurrentlyPlayingClip>,
    // duration_range: [FadeDuration; 2],
    pub action_queue: Vec<ActionQueueItem>,
    pub last_state_publish: SystemTime,
    pub settings: ManualSettings,
    // multi_channel_mode: bool,
    pub tether: TetherAgent,
    pub remote_control: Option<RemoteControl>,
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

        let remote_control = if cli.tether_disable {
            None
        } else {
            Some(RemoteControl::new(&tether))
        };

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
            action_queue: Vec::new(),
            last_state_publish: std::time::SystemTime::now(),
            settings,
            tether,
            remote_control,
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

    pub fn play_one_clip(&mut self, clip_name: String, should_loop: bool, fade: Option<Duration>) {
        if let Some(sample) = self
            .sound_bank
            .clips()
            .iter()
            .find(|x| x.name() == clip_name)
        {
            let clip_with_sink = ClipWithSink::new(
                self.clips_playing.len(),
                &sample,
                should_loop,
                fade,
                &self.output_stream_handle,
                String::from(sample.name()),
            );
            self.clips_playing.push(clip_with_sink);
        } else {
            error!("Failed to find clip in bank with name, {}", clip_name);
        }
    }
}
