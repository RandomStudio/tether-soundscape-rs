use log::{debug, error, warn};
use std::{
    path::Path,
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use rodio::OutputStreamHandle;
use tether_agent::{TetherAgent, TetherAgentOptionsBuilder};

use crate::{
    loader::SoundBank,
    playback::{ClipWithSink, PanWithRange},
    remote_control::{
        publish::SoundscapeEvent,
        receive::{Instruction, ScenePickMode},
        RemoteControl,
    },
    settings::Cli,
    utils::{optional_ms_to_duration, pick_random_clip},
};

pub enum ActionQueueItem {
    /// Start playback: name, optional fade duration, should_loop,
    /// optional pan position with range
    Play(String, Option<Duration>, bool, Option<PanWithRange>),
    /// Stop/fade out: id in currently_playing Vec, optional fade duration
    Stop(usize, Option<Duration>),
}

pub struct Model {
    _request_loop_handle: JoinHandle<()>,
    // pub request_channel: (Sender<()>, Receiver<()>),
    pub request_rx: Receiver<()>,
    pub output_stream_handle: OutputStreamHandle,
    pub output_channels_used: u16,
    pub sound_bank: SoundBank,
    pub clips_playing: Vec<ClipWithSink>,
    // clips_playing: Vec<CurrentlyPlayingClip>,
    // duration_range: [FadeDuration; 2],
    pub action_queue: Vec<ActionQueueItem>,
    pub last_state_publish: SystemTime,
    pub tether: TetherAgent,
    pub tether_disabled: bool,
    pub remote_control: Option<RemoteControl>,
}

impl Model {
    pub fn new(
        cli: &Cli,
        output_stream_handle: OutputStreamHandle,
        output_channels_used: u16,
    ) -> Model {
        let sound_bank = SoundBank::new(Path::new("test.json"));
        // let duration_range = get_duration_range(sound_bank.clips());

        // let tether = TetherAgent::new("soundscape", None, None);
        let tether_options = TetherAgentOptionsBuilder::new("soundscape").auto_connect(false);
        let tether = if cli.tether_disable {
            warn!("Tether connection disabled");
            tether_options
                .build()
                .expect("failed to init (not connect) Tether")
        } else {
            tether_options
                .auto_connect(true)
                .build()
                .expect("failed to connect Tether")
        };

        let remote_control = if cli.tether_disable {
            None
        } else {
            Some(RemoteControl::new(
                &tether,
                Duration::from_millis(cli.state_interval),
            ))
        };

        let (tx, rx) = mpsc::channel();

        let update_interval = cli.update_interval; // clone for move

        let request_loop_handle = thread::spawn(move || loop {
            tx.send(()).expect("failed to send via channel");
            thread::sleep(Duration::from_millis(update_interval));
        });

        Model {
            // request_channel: (tx, rx),
            request_rx: rx,
            _request_loop_handle: request_loop_handle,
            output_stream_handle,
            output_channels_used,
            sound_bank,
            clips_playing: Vec::new(),
            action_queue: Vec::new(),
            last_state_publish: std::time::SystemTime::now(),
            tether,
            remote_control,
            tether_disabled: cli.tether_disable,
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

    pub fn play_one_clip(
        &mut self,
        clip_name: &str,
        should_loop: bool,
        fade: Option<Duration>,
        override_panning: Option<PanWithRange>,
    ) {
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
                override_panning,
                &self.output_stream_handle,
                self.output_channels_used,
            );
            self.clips_playing.push(clip_with_sink);
        } else {
            error!("Failed to find clip in bank with name, {}", clip_name);
        }
    }

    pub fn internal_update(&mut self) {
        if let Ok(_) = self.request_rx.try_recv() {
            self.check_progress();
        }

        // Parse any remote control messages, which may generate CommandQueue items
        if let Some(remote_control) = &mut self.remote_control {
            while let Some((plug_name, message)) = self.tether.check_messages() {
                match remote_control.parse_instructions(&plug_name, &message) {
                    Ok(Instruction::Add(clip_name, should_loop, fade_ms, panning)) => {
                        self.action_queue.push(ActionQueueItem::Play(
                            clip_name,
                            match fade_ms {
                                Some(ms) => Some(Duration::from_millis(ms)),
                                None => None,
                            },
                            should_loop,
                            panning,
                        ));
                    }
                    Ok(Instruction::Remove(clip_name, fade_ms)) => {
                        for clip in self
                            .clips_playing
                            .iter_mut()
                            .filter(|x| x.name() == clip_name)
                        {
                            // if let Some(ms) = fade_ms {
                            //     clip.fade_out(Duration::from_millis(ms));
                            // } else {
                            //     clip.stop();
                            // }
                            self.action_queue.push(ActionQueueItem::Stop(
                                clip.id(),
                                fade_ms.map(|fade| Duration::from_millis(fade)),
                            ))
                        }
                    }
                    Ok(Instruction::Scene(scene_pick_mode, clip_names, fade_ms)) => {
                        match scene_pick_mode {
                            ScenePickMode::OnceAll => {
                                if clip_names.len() == 0 {
                                    debug!("Empty scene list; stop all currently playing");
                                    for clip in &self.clips_playing {
                                        self.action_queue.push(ActionQueueItem::Stop(
                                            clip.id(),
                                            optional_ms_to_duration(fade_ms),
                                        ))
                                    }
                                } else {
                                    for name in clip_names {
                                        self.action_queue.push(ActionQueueItem::Play(
                                            name,
                                            match fade_ms {
                                                Some(ms) => Some(Duration::from_millis(ms)),
                                                None => None,
                                            },
                                            false,
                                            None,
                                        ));
                                    }
                                }
                            }
                            ScenePickMode::LoopAll => {
                                // TODO: check for
                                // - empty list (stop all)
                                // - clips already playing (and LOOPING) (do not add)
                                if clip_names.len() == 0 {
                                    debug!("Empty scene list; stop all currently playing that are looping");
                                    for clip in &self.clips_playing {
                                        self.action_queue.push(ActionQueueItem::Stop(
                                            clip.id(),
                                            optional_ms_to_duration(fade_ms),
                                        ))
                                    }
                                } else {
                                    let to_add = clip_names.iter().filter(|candidate| {
                                        self.clips_playing
                                            .iter()
                                            .find(|playing| {
                                                playing.name().eq_ignore_ascii_case(&candidate)
                                            })
                                            .is_none()
                                    });
                                    let to_remove = self.clips_playing.iter().filter(|playing| {
                                        clip_names
                                            .iter()
                                            .find(|requested| {
                                                requested.eq_ignore_ascii_case(&playing.name())
                                            })
                                            .is_none()
                                    });
                                    for name in to_add {
                                        self.action_queue.push(ActionQueueItem::Play(
                                            name.into(),
                                            optional_ms_to_duration(fade_ms),
                                            true,
                                            None,
                                        ));
                                    }
                                    for clip in to_remove {
                                        self.action_queue.push(ActionQueueItem::Stop(
                                            clip.id(),
                                            optional_ms_to_duration(fade_ms),
                                        ));
                                    }
                                }
                            }
                            ScenePickMode::OnceRandomSinglePick => {
                                let pick_name = pick_random_clip(clip_names);
                                self.action_queue.push(ActionQueueItem::Play(
                                    pick_name,
                                    optional_ms_to_duration(fade_ms),
                                    false,
                                    None,
                                ));
                            }
                        }
                    }
                    Err(_) => {
                        error!("Failed to parse remote Instruction");
                    }
                }
            }
        }
        while let Some(command) = self.action_queue.pop() {
            match command {
                ActionQueueItem::Play(clip_name, fade, should_loop, panning) => {
                    self.play_one_clip(&clip_name, should_loop, fade, panning);
                    if let Some(remote) = &self.remote_control {
                        remote.publish_event(SoundscapeEvent::ClipStarted(clip_name), &self.tether)
                    }
                }
                ActionQueueItem::Stop(id, fade) => {
                    if let Some(clip) = self.clips_playing.iter_mut().find(|x| x.id() == id) {
                        match fade {
                            Some(duration) => clip.fade_out(duration),
                            None => clip.stop(),
                        };
                        if let Some(remote) = &self.remote_control {
                            remote.publish_event(
                                SoundscapeEvent::ClipEnded(clip.name().into()),
                                &self.tether,
                            )
                        }
                    }
                }
            };
        }

        if let Some(remote) = &mut self.remote_control {
            remote.publish_state_if_ready(&self.tether, &self.clips_playing);
        }
    }
}
