use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use log::{debug, error, info, trace, warn};
use rmp_serde::to_vec_named;
// use rmp_serde::to_vec_named;
use serde::{Deserialize, Serialize};
use tether_agent::{mqtt::Message, PlugDefinition, PlugOptionsBuilder, TetherAgent};

use crate::playback::{ClipWithSink, PlaybackPhase};

// const INPUT_TOPICS: &[&str] = &["+/+/clipCommands", "+/+/scenes", "+/+/globalControls"];
// const INPUT_QOS: &[i32; INPUT_TOPICS.len()] = &[2, 2, 2];

type ClipName = String;

/// Position (in range 0>numChannels-1) and spread (in range 1>numChannels)
pub type PanWithRange = (f32, f32);

pub enum ScenePickMode {
    LoopAll,
    OnceAll,
    OnceRandomSinglePick,
}

type FadeDurationMS = u64;
pub enum Instruction {
    // Clip name, should_loop, optional fade duration, optional panning
    Add(ClipName, bool, Option<FadeDurationMS>, Option<PanWithRange>),
    // Clip name, option fade duration
    Remove(ClipName, Option<FadeDurationMS>),
    Scene(ScenePickMode, Vec<ClipName>, Option<FadeDurationMS>),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClipPlayingEssentialState {
    id: usize,
    name: String,
    progress: f32,
    current_volume: f32,
    looping: bool,
    phase: String,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SoundscapeStateMessage {
    pub clips: Vec<ClipPlayingEssentialState>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SingleClipMessage {
    pub command: String,
    pub clip_name: ClipName,
    pub fade_duration: Option<FadeDurationMS>,
    pub pan_position: Option<f32>,
    pub pan_spread: Option<f32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SceneMessage {
    pub mode: Option<String>,
    pub clip_names: Vec<ClipName>,
    pub fade_duration: Option<FadeDurationMS>,
}

/// If at least a pan position is provided, then return a valid "SimplePanning" tuple,
/// and use a default "pan spread" unless provided with one as well;
/// otherwise, return None
fn parse_optional_panning(parsed: &SingleClipMessage) -> Option<PanWithRange> {
    match parsed.pan_position {
        None => None,
        Some(pan_position) => Some((pan_position, parsed.pan_spread.unwrap_or(1.0))),
    }
}

pub struct RemoteControl {
    output_plug: PlugDefinition,
    input_plugs: HashMap<String, PlugDefinition>,
    state_send_interval: Duration,
    last_update_sent: SystemTime, // last_clip_count_sent: Option<usize>,
}

impl RemoteControl {
    pub fn new(tether_agent: &TetherAgent, state_send_interval: Duration) -> Self {
        let mut input_plugs: HashMap<String, PlugDefinition> = HashMap::new();
        input_plugs.insert(
            "clipCommands".into(),
            PlugOptionsBuilder::create_input("clipCommands")
                .qos(2)
                .build(tether_agent)
                .expect("failed to create clipCommands Input"), // tether_agent
                                                                //     .create_input_plug("clipCommands", Some(2), None)
                                                                //     .unwrap(),
        );
        input_plugs.insert(
            "scenes".into(),
            PlugOptionsBuilder::create_input("scenes")
                .qos(2)
                .build(tether_agent)
                .expect("failed to create scenes Input"), // tether_agent
                                                          //     .create_input_plug("scenes", Some(2), None)
                                                          //     .unwrap(),
        );
        input_plugs.insert(
            "globalControls".into(),
            PlugOptionsBuilder::create_input("globalControls")
                .qos(2)
                .build(tether_agent)
                .expect("failed to create globalCommands Input"), // tether_agent
                                                                  //     .create_input_plug("globalControls", Some(2), None)
                                                                  //     .unwrap(),
        );

        let output_plug = PlugOptionsBuilder::create_output("state")
            .qos(0)
            .build(tether_agent)
            .expect("failed to create state Output");

        RemoteControl {
            output_plug,
            input_plugs,
            state_send_interval,
            last_update_sent: SystemTime::now(), // last_clip_count_sent: None,
        }
    }

    pub fn parse_instructions(
        &self,
        plug_name: &str,
        message: &Message,
    ) -> Result<Instruction, ()> {
        let payload = message.payload();

        if let Some(matched_plug) = self.input_plugs.get(plug_name) {
            match matched_plug.name() {
                "clipCommands" => {
                    let clip_message: Result<SingleClipMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(&payload);

                    if let Ok(parsed) = clip_message {
                        info!("Parsed Single Clip Message: {parsed:?}");

                        let panning: Option<PanWithRange> = parse_optional_panning(&parsed);

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
                        info!("Parsed Scene Message: {parsed:?}");

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
                            "onceRandom" => Ok(Instruction::Scene(
                                ScenePickMode::OnceRandomSinglePick,
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

    pub fn publish_state_if_ready(&mut self, agent: &TetherAgent, clips: &[ClipWithSink]) {
        let elapsed = self.last_update_sent.elapsed().unwrap();

        if elapsed <= self.state_send_interval {
            trace!(
                "Not ready: {} <= {}",
                elapsed.as_millis(),
                self.state_send_interval.as_millis()
            );
            return;
        }

        trace!("Ready to send state update");
        self.last_update_sent = SystemTime::now();

        let clip_states = clips
            .iter()
            .map(|c| ClipPlayingEssentialState {
                id: c.id(),
                name: c.name().into(),
                progress: c.progress().unwrap_or(0.),
                looping: c.is_looping(),
                current_volume: c.current_volume(),
                phase: match c.phase() {
                    PlaybackPhase::Attack(_) => "attack",
                    PlaybackPhase::Sustain() => "sustain",
                    PlaybackPhase::Release(..) => "release",
                }
                .into(),
            })
            .collect();
        let state = SoundscapeStateMessage { clips: clip_states };
        let payload: Vec<u8> = to_vec_named(&state).unwrap();
        agent
            .publish(&self.output_plug, Some(&payload))
            .expect("Failed to publish state/progress");
    }
}
