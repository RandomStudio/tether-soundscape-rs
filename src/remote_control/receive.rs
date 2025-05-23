use ::anyhow::anyhow;
use log::*;
use serde::Deserialize;
use tether_agent::three_part_topic::TetherOrCustomTopic;
use ts_rs::TS;

use crate::{playback::PanWithRange, utils::parse_optional_panning};

use super::RemoteControl;

pub enum ScenePickMode {
    LoopAll,
    OnceAll,
    OnceRandomSinglePick,
}

pub enum GlobalControlMode {
    PauseAll(),
    ResumeAll(),
    SilenceAll(),
    MasterVolume(f32),
}

pub enum Instruction {
    // Clip name, should_loop, optional volume (override), fade duration, optional panning
    Add(String, bool, Option<f32>, Option<u32>, Option<PanWithRange>),
    // Clip name, option fade duration
    Remove(String, Option<u32>),
    Scene(ScenePickMode, Vec<String>, Option<u32>),
    Global(GlobalControlMode),
}

#[derive(Deserialize, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SingleClipMessage {
    pub command: String,
    pub clip_name: String,
    pub fade_duration: Option<u32>,
    pub pan_position: Option<f32>,
    pub pan_spread: Option<f32>,
    pub volume: Option<f32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SceneMessage {
    pub mode: Option<String>,
    pub clip_names: Vec<String>,
    pub fade_duration: Option<u32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GlobalMessage {
    pub command: String,
    pub volume: Option<f32>,
}

impl RemoteControl {
    pub fn parse_instructions(
        &self,
        plug: &TetherOrCustomTopic,
        payload: &[u8],
    ) -> anyhow::Result<Instruction> {
        match plug {
            TetherOrCustomTopic::Tether(three_part_topic) => match three_part_topic.plug_name() {
                "clipCommands" => {
                    let clip_message: Result<SingleClipMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(payload);

                    if let Ok(parsed) = clip_message {
                        info!("Parsed Single Clip Message: {parsed:?}");

                        let panning: Option<PanWithRange> =
                            parse_optional_panning(parsed.pan_position, parsed.pan_spread);

                        match parsed.command.as_str() {
                            "hit" => Ok(Instruction::Add(
                                parsed.clip_name,
                                false,
                                parsed.volume,
                                parsed.fade_duration,
                                panning,
                            )),
                            "add" => Ok(Instruction::Add(
                                parsed.clip_name,
                                true,
                                parsed.volume,
                                parsed.fade_duration,
                                panning,
                            )),
                            "remove" => {
                                Ok(Instruction::Remove(parsed.clip_name, parsed.fade_duration))
                            }
                            _ => Err(anyhow!(
                                "Unrecognised command for Single Clip Message: {}",
                                &parsed.command
                            )),
                        }
                    } else {
                        Err(anyhow!("Error parsing Single Clip Message"))
                    }
                }
                "scenes" => {
                    let scene_message: Result<SceneMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(payload);

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
                            _ => Err(anyhow!(
                                "Unrecognised 'pick' option for Scene Message: {}",
                                &pick_mode
                            )),
                        }
                    } else {
                        Err(anyhow!("Error parsing Scene Message"))
                    }
                }
                "globalControls" => {
                    let global_message: Result<GlobalMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(payload);

                    if let Ok(parsed) = global_message {
                        info!("Paused GlobalCommand message: {parsed:?}");

                        match parsed.command.as_str() {
                            "pause" => Ok(Instruction::Global(GlobalControlMode::PauseAll())),
                            "play" => Ok(Instruction::Global(GlobalControlMode::ResumeAll())),
                            "silence" => Ok(Instruction::Global(GlobalControlMode::SilenceAll())),
                            "masterVolume" => Ok(Instruction::Global(
                                GlobalControlMode::MasterVolume(parsed.volume.unwrap_or_default()),
                            )),
                            _ => Err(anyhow!(
                                "Unrecognised command option for GlobalControls Message: {}",
                                &parsed.command
                            )),
                        }
                    } else {
                        Err(anyhow!("Failed to parse GlobalCommand message"))
                    }
                }
                &_ => Err(anyhow!("Unrecognised plug name")),
            },
            TetherOrCustomTopic::Custom(_) => panic!("Not a valid Tether topic"),
        }
    }
}
