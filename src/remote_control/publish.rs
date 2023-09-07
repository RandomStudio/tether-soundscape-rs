use std::time::SystemTime;

use log::trace;
use rmp_serde::to_vec_named;
use serde::Serialize;
use tether_agent::TetherAgent;

use crate::playback::{ClipWithSink, PlaybackPhase};

use super::RemoteControl;

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

impl RemoteControl {
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
            .publish(&self.state_output_plug, Some(&payload))
            .expect("Failed to publish state/progress");
    }

    pub fn publish_event() {}
}
