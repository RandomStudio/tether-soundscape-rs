use std::collections::HashMap;

use log::{error, info, warn};
use nannou::prelude::ToPrimitive;
use rmp_serde::to_vec_named;
use serde::{Deserialize, Serialize};
use tether_agent::{mqtt::Message, PlugDefinition, TetherAgent};

use crate::{playback::PlaybackState, CurrentlyPlayingClip, FadeDuration};

// const INPUT_TOPICS: &[&str] = &["+/+/clipCommands", "+/+/scenes", "+/+/globalControls"];
// const INPUT_QOS: &[i32; INPUT_TOPICS.len()] = &[2, 2, 2];

type ClipName = String;

/// Position (in range 0>numChannels-1) and spread (in range 1>numChannels)
pub type SimplePanning = (f32, f32);

pub enum ScenePickMode {
    LoopAll,
    OnceAll,
    Random,
}
pub enum Instruction {
    // Clip name, should_loop, optional fade duration, optional panning
    Add(ClipName, bool, Option<FadeDuration>, Option<SimplePanning>),
    // Clip name, option fade duration
    Remove(ClipName, Option<FadeDuration>),
    Scene(ScenePickMode, Vec<ClipName>, Option<FadeDuration>),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClipPlayingEssentialState {
    id: usize,
    name: String,
    progress: f32,
    current_volume: f32,
    looping: bool,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SoundscapeStateMessage {
    pub is_playing: bool,
    pub clips: Vec<ClipPlayingEssentialState>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SingleClipMessage {
    pub command: String,
    pub clip_name: ClipName,
    pub fade_duration: Option<FadeDuration>,
    pub pan_position: Option<f32>,
    pub pan_spread: Option<f32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SceneMessage {
    pub mode: Option<String>,
    pub clip_names: Vec<ClipName>,
    pub fade_duration: Option<FadeDuration>,
}

/// If at least a pan position is provided, then return a valid "SimplePanning" tuple,
/// and use a default "pan spread" unless provided with one as well;
/// otherwise, return None
fn parse_optional_panning(parsed: &SingleClipMessage) -> Option<SimplePanning> {
    match parsed.pan_position {
        None => None,
        Some(pan_position) => Some((pan_position, parsed.pan_spread.unwrap_or(1.0))),
    }
}

pub struct RemoteControl {
    output_plug: PlugDefinition,
    input_plugs: HashMap<String, PlugDefinition>,
    last_clip_count_sent: Option<usize>,
}

impl RemoteControl {
    pub fn new(tether_agent: &TetherAgent) -> Self {
        let mut input_plugs: HashMap<String, PlugDefinition> = HashMap::new();
        input_plugs.insert(
            "clipCommands".into(),
            tether_agent
                .create_input_plug("clipCommands", Some(2), None)
                .unwrap(),
        );
        input_plugs.insert(
            "scenes".into(),
            tether_agent
                .create_input_plug("scenes", Some(2), None)
                .unwrap(),
        );
        input_plugs.insert(
            "globalControls".into(),
            tether_agent
                .create_input_plug("globalControls", Some(2), None)
                .unwrap(),
        );
        RemoteControl {
            output_plug: tether_agent
                .create_output_plug("state", Some(0), None)
                .expect("failed to create state Output Plug"),
            input_plugs,
            last_clip_count_sent: None,
        }
    }

    pub fn parse_instructions(
        &self,
        plug_name: &str,
        message: &Message,
    ) -> Result<Instruction, ()> {
        let payload = message.payload();

        if let Some(matched_plug) = self.input_plugs.get(plug_name) {
            match matched_plug.name.as_str() {
                "clipCommands" => {
                    let clip_message: Result<SingleClipMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(&payload);

                    if let Ok(parsed) = clip_message {
                        info!("Parsed Single Clip Message: {parsed:?}");

                        let panning: Option<SimplePanning> = parse_optional_panning(&parsed);

                        match parsed.command.as_str() {
                            "hit" => Ok(Instruction::Add(
                                parsed.clip_name,
                                false,
                                parsed.fade_duration,
                                panning,
                            )),
                            "add" => Ok(Instruction::Add(
                                parsed.clip_name,
                                true,
                                parsed.fade_duration,
                                panning,
                            )),
                            "remove" => {
                                Ok(Instruction::Remove(parsed.clip_name, parsed.fade_duration))
                            }
                            _ => {
                                error!(
                                    "Unrecognised command for Single Clip Message: {}",
                                    &parsed.command
                                );
                                Err(())
                            }
                        }
                    } else {
                        error!("Error parsing Single Clip Message");
                        Err(())
                    }
                }
                "scenes" => {
                    let scene_message: Result<SceneMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(&payload);

                    if let Ok(parsed) = scene_message {
                        let pick_mode = parsed.mode.unwrap_or(String::from("loopAll"));
                        match pick_mode.as_str() {
                            "loopAll" => Ok(Instruction::Scene(
                                ScenePickMode::LoopAll,
                                parsed.clip_names,
                                parsed.fade_duration,
                            )),
                            "onceAll" => Ok(Instruction::Scene(
                                ScenePickMode::OnceAll,
                                parsed.clip_names,
                                parsed.fade_duration,
                            )),
                            "random" => Ok(Instruction::Scene(
                                ScenePickMode::Random,
                                parsed.clip_names,
                                parsed.fade_duration,
                            )),
                            _ => {
                                error!(
                                    "Unrecognised 'pick' option for Scene Message: {}",
                                    &pick_mode
                                );
                                Err(())
                            }
                        }
                    } else {
                        error!("Error parsing Scene Message");
                        Err(())
                    }
                }
                "globalControls" => {
                    // TODO
                    warn!("globalControls not handled yet");
                    Err(())
                }
                &_ => {
                    error!("Unrecognised plug name");
                    Err(())
                }
            }
        } else {
            error!("Could not match any plug");
            Err(())
        }
    }

    pub fn publish_state(
        &mut self,
        is_stream_playing: bool,
        clips: &[CurrentlyPlayingClip],
        agent: &TetherAgent,
    ) {
        let should_publish = {
            match self.last_clip_count_sent {
                None => true,
                Some(last_count) => !clips.is_empty() || clips.len() != last_count,
            }
        };
        if should_publish {
            self.last_clip_count_sent = Some(clips.len());
            let clip_states = clips
                .iter()
                .map(|c| {
                    let progress = match c.state {
                        PlaybackState::Playing(frames_played) => {
                            frames_played.to_f32().unwrap() / c.frames_count.to_f32().unwrap()
                        }
                        _ => 0.,
                    };

                    ClipPlayingEssentialState {
                        id: c.id,
                        name: c.name.clone(),
                        progress,
                        looping: c.should_loop,
                        current_volume: c.current_volume,
                    }
                })
                .collect();
            let state = SoundscapeStateMessage {
                clips: clip_states,
                is_playing: is_stream_playing,
            };
            let payload: Vec<u8> = to_vec_named(&state).unwrap();
            agent
                .publish(&self.output_plug, Some(&payload))
                .expect("Failed to publish state/progress");
        }
    }
}
