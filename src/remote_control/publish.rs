use std::time::SystemTime;

use log::trace;
use rmp_serde::to_vec_named;
use serde::Serialize;
use tether_agent::TetherAgent;

use crate::playback::{ClipWithSink, PlaybackPhase};

use super::RemoteControl;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ClipPlayingEssentialState {
    id: usize,
    name: String,
    progress: f32,
    current_volume: f32,
    looping: bool,
    phase: String,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SoundscapeStateMessage {
    pub clips: Vec<ClipPlayingEssentialState>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SoundscapeEvent {
    ClipStarted(String),
    ClipEnded(String),
}

impl RemoteControl {
    pub fn publish_state_if_ready(&mut self, agent: &TetherAgent, clips: &[ClipWithSink]) -> bool {
        let elapsed = self.last_update_sent.elapsed().unwrap();

        if elapsed <= self.state_send_interval {
            trace!(
                "Not ready: {} <= {}",
                elapsed.as_millis(),
                self.state_send_interval.as_millis()
            );
            return false;
        }

        trace!("Ready to send state update");
        self.last_update_sent = SystemTime::now();

        let clip_states: Vec<ClipPlayingEssentialState> = clips
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

        let no_clips_playing = &clip_states.is_empty();

        let state = SoundscapeStateMessage { clips: clip_states };

        // Check if we have already sent too many "zero length" states
        if *no_clips_playing {
            if let Some(count) = self.count_empty_state_sends {
                if count > self.state_max_empty {
                    return false;
                }
                self.count_empty_state_sends = Some(count + 1);
            } else {
                self.count_empty_state_sends = Some(1);
            }
        } else {
            self.count_empty_state_sends = None;
        }

        let payload: Vec<u8> = to_vec_named(&state).unwrap();
        agent
            .publish(&self.state_output_plug, Some(&payload))
            .expect("failed to publish state/progress");
        true
    }

    pub fn publish_event(&self, event: SoundscapeEvent, tether: &TetherAgent) {
        let payload = to_vec_named(&event).unwrap();
        tether
            .publish(&self.events_output_plug, Some(&payload))
            .expect("failed to publish event");
    }
}
