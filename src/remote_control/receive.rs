use log::{error, info, warn};
use serde::Deserialize;
use tether_agent::mqtt::Message;

use crate::{playback::PanWithRange, utils::parse_optional_panning};

use super::RemoteControl;

type ClipName = String;

pub enum ScenePickMode {
    LoopAll,
    OnceAll,
    OnceRandomSinglePick,
}

type FadeDurationMS = u64;
pub enum Instruction {
    // Clip name, should_loop, optional volume (override), fade duration, optional panning
    Add(
        ClipName,
        bool,
        Option<f32>,
        Option<FadeDurationMS>,
        Option<PanWithRange>,
    ),
    // Clip name, option fade duration
    Remove(ClipName, Option<FadeDurationMS>),
    Scene(ScenePickMode, Vec<ClipName>, Option<FadeDurationMS>),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SingleClipMessage {
    pub command: String,
    pub clip_name: ClipName,
    pub fade_duration: Option<FadeDurationMS>,
    pub pan_position: Option<f32>,
    pub pan_spread: Option<f32>,
    pub volume: Option<f32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SceneMessage {
    pub mode: Option<String>,
    pub clip_names: Vec<ClipName>,
    pub fade_duration: Option<FadeDurationMS>,
}

impl RemoteControl {
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
}
