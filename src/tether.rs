use log::{debug, error, info};
use mqtt::{Client, Message, Receiver};
use paho_mqtt as mqtt;
use rmp_serde::to_vec_named;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, process, time::Duration};

use crate::{playback::PlaybackState, CurrentlyPlayingClip, FadeDuration};
use nannou::prelude::ToPrimitive;

const INPUT_TOPICS: &[&str] = &["+/+/clipCommands", "+/+/scenes", "+/+/globalControls"];
const INPUT_QOS: &[i32; INPUT_TOPICS.len()] = &[2, 2, 2];
const OUTPUT_TOPIC: &str = "soundscape/unknown/state";

type ClipName = String;

/// Position (in range 0>numChannels-1) and spread (in range 1>numChannels)
pub type SimplePanning = (f32, f32);

pub enum Instruction {
    // Clip name, should_loop, optional fade duration, optional panning
    Add(ClipName, bool, Option<FadeDuration>, Option<SimplePanning>),
    // Clip name, option fade duration
    Remove(ClipName, Option<FadeDuration>),
    Scene(Vec<ClipName>, Option<FadeDuration>),
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
    pub pick: Option<String>,
    pub clip_names: Vec<ClipName>,
    pub fade_duration: Option<FadeDuration>,
}

pub struct TetherAgent {
    client: Client,
    receiver: Receiver<Option<Message>>,
    last_clip_count_sent: Option<usize>,
}

impl TetherAgent {
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    pub fn new(tether_host: IpAddr) -> Self {
        let broker_uri = format!("tcp://{tether_host}:1883");

        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(broker_uri)
            .client_id("")
            .finalize();

        // Create the client connection
        let client = mqtt::Client::new(create_opts).unwrap();

        // Initialize the consumer before connecting
        let receiver = client.start_consuming();

        TetherAgent {
            client,
            receiver,
            last_clip_count_sent: None,
        }
    }

    pub fn connect(&mut self) {
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .user_name("tether")
            .password("sp_ceB0ss!")
            .keep_alive_interval(Duration::from_secs(30))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(true)
            .finalize();

        // Make the connection to the broker
        debug!("Connecting to the MQTT server...");
        match self.client.connect(conn_opts) {
            Ok(res) => {
                info!("MQTT client connected OK");
                debug!("Connected OK: {res:?}");
                match self.client.subscribe_many(INPUT_TOPICS, INPUT_QOS) {
                    Ok(res) => {
                        debug!("Subscribe OK: {res:?}");
                    }
                    Err(e) => {
                        error!("Error subscribing: {e:?}");
                    }
                }
            }
            Err(e) => {
                error!("Error connecting to the broker: {e:?}");
                process::exit(1);
            }
        }
    }

    pub fn check_messages(&self) -> Option<Instruction> {
        if let Some(m) = self.receiver.try_iter().find_map(|m| m) {
            let payload = m.payload().to_vec();

            let plug_name = parse_plug_name(m.topic());

            match plug_name {
                "clipCommands" => {
                    let clip_message: Result<SingleClipMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(&payload);

                    if let Ok(parsed) = clip_message {
                        info!("Parsed Single Clip Message: {parsed:?}");

                        let panning: Option<SimplePanning> = parse_optional_panning(&parsed);

                        match parsed.command.as_str() {
                            "hit" => Some(Instruction::Add(
                                parsed.clip_name,
                                false,
                                parsed.fade_duration,
                                panning,
                            )),
                            "add" => Some(Instruction::Add(
                                parsed.clip_name,
                                true,
                                parsed.fade_duration,
                                panning,
                            )),
                            "remove" => {
                                Some(Instruction::Remove(parsed.clip_name, parsed.fade_duration))
                            }
                            _ => {
                                error!(
                                    "Unrecognised command for Single Clip Message: {}",
                                    &parsed.command
                                );
                                None
                            }
                        }
                    } else {
                        error!("Error parsing Single Clip Message");
                        None
                    }
                }
                "scenes" => {
                    let scene_message: Result<SceneMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(&payload);

                    if let Ok(parsed) = scene_message {
                        let pick_mode = parsed.pick.unwrap_or(String::from("all"));
                        match pick_mode.as_str() {
                            "all" => {
                                Some(Instruction::Scene(parsed.clip_names, parsed.fade_duration))
                            }
                            "pickRandom" => {
                                // TODO: handle pick random
                                Some(Instruction::Scene(parsed.clip_names, parsed.fade_duration))
                            }
                            _ => {
                                error!(
                                    "Unrecognised 'pick' option for Scene Message: {}",
                                    &pick_mode
                                );
                                None
                            }
                        }
                    } else {
                        error!("Error parsing Scene Message");
                        None
                    }
                }
                "globalControls" => {
                    // TODO
                    None
                }
                _ => {
                    error!("Should not be receiving message on topic {}", m.topic());
                    None
                }
            }
        } else {
            // No error - there simply is no message waiting on the queue
            None
        }
    }

    pub fn publish_state(&mut self, is_stream_playing: bool, clips: &[CurrentlyPlayingClip]) {
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
            let msg = mqtt::Message::new(OUTPUT_TOPIC, payload, 1);
            self.client
                .publish(msg)
                .expect("Failed to publish state/progress");
        }
    }
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

fn parse_plug_name(topic: &str) -> &str {
    let parts: Vec<&str> = topic.split('/').collect();
    parts[2]
}
