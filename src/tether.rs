use log::{debug, error, info};
use mqtt::{Client, Message, Receiver};
use paho_mqtt as mqtt;
use rmp_serde::to_vec_named;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, process, time::Duration};

use crate::{playback::PlaybackState, CurrentlyPlayingClip, FadeDuration};
use nannou::prelude::ToPrimitive;

const INPUT_TOPICS: &[&str] = &["+/+/instructions"];
const INPUT_QOS: &[i32; INPUT_TOPICS.len()] = &[2];

pub struct TetherAgent {
    client: Client,
    receiver: Receiver<Option<Message>>,
}

type ClipName = String;
pub enum Instruction {
    Hit(Vec<ClipName>),
    Add(Vec<ClipName>, Option<FadeDuration>),
    Remove(Vec<ClipName>, Option<FadeDuration>),
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
pub struct InstructionMessage {
    pub instruction_type: String,
    pub clip_names: Vec<ClipName>,
    pub fade_duration: Option<FadeDuration>,
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

        TetherAgent { client, receiver }
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
        info!("Connecting to the MQTT server...");
        match self.client.connect(conn_opts) {
            Ok(res) => {
                info!("Connected OK: {res:?}");
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
                "instructions" => {
                    let light_message: Result<InstructionMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(&payload);

                    match light_message {
                        Ok(parsed) => {
                            info!("Parsed InstructionMessage: {parsed:?}");
                            if let Ok(valid_instruction) = get_instruction(&parsed) {
                                Some(valid_instruction)
                            } else {
                                None
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse InstructionMessage: {}", e);
                            None
                        }
                    }
                }
                _ => {
                    error!("Should not be receiving message on topic {}", m.topic());
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn publish_state(&self, is_stream_playing: bool, clips: &[CurrentlyPlayingClip]) {
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
        let msg = mqtt::Message::new("soundscape/unknown/state", payload, 2);
        self.client
            .publish(msg)
            .expect("Failed to publish state/progress");
    }
}

fn get_instruction(msg: &InstructionMessage) -> Result<Instruction, ()> {
    let instruction_type = msg.instruction_type.as_str();
    let fade_duration = msg.fade_duration;
    match instruction_type {
        "hit" => Ok(Instruction::Hit(msg.clip_names.clone())),
        "add" => Ok(Instruction::Add(msg.clip_names.clone(), fade_duration)),
        "remove" => Ok(Instruction::Remove(msg.clip_names.clone(), fade_duration)),
        "scene" => Ok(Instruction::Scene(msg.clip_names.clone(), fade_duration)),
        _ => {
            error!("Unknown instructionType {}", msg.instruction_type);
            Err(())
        }
    }
}

fn parse_plug_name(topic: &str) -> &str {
    let parts: Vec<&str> = topic.split('/').collect();
    parts[2]
}
