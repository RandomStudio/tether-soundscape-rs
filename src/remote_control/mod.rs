pub mod publish;
pub mod receive;

use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use tether_agent::{PlugDefinition, PlugOptionsBuilder, TetherAgent};

pub struct RemoteControl {
    state_output_plug: PlugDefinition,
    events_output_plug: PlugDefinition,
    // input_plugs: HashMap<String, PlugDefinition>,
    state_send_interval: Duration,
    state_max_empty: usize,
    count_empty_state_sends: Option<usize>,
    last_update_sent: SystemTime, // last_clip_count_sent: Option<usize>,
}

impl RemoteControl {
    pub fn new(
        tether_agent: &TetherAgent,
        override_subscribe_id: Option<&str>,
        state_send_interval: Duration,
        state_max_empty: usize,
    ) -> Self {
        let _input_plugs: HashMap<String, PlugDefinition> = HashMap::from([
            (
                "clipCommands".into(),
                PlugOptionsBuilder::create_input("clipCommands")
                    .qos(Some(2))
                    // .topic(&build_topic("+".into(), &id, "clipCommands"))
                    .id(override_subscribe_id)
                    .build(tether_agent)
                    .expect("failed to create clipCommands Input"), // tether_agent
            ),
            (
                "scenes".into(),
                PlugOptionsBuilder::create_input("scenes")
                    .qos(Some(2))
                    // .topic(&build_topic("+".into(), &id, "scenes"))
                    .id(override_subscribe_id)
                    .build(tether_agent)
                    .expect("failed to create scenes Input"), // tether_agent
            ),
            (
                "globalControls".into(),
                PlugOptionsBuilder::create_input("globalControls")
                    .qos(Some(2))
                    // .topic(&build_topic("+".into(), &id, "globalControls"))
                    .id(override_subscribe_id)
                    .build(tether_agent)
                    .expect("failed to create globalCommands Input"), // tether_agent
            ),
        ]);

        let state_output_plug = PlugOptionsBuilder::create_output("state")
            .qos(Some(0))
            .build(tether_agent)
            .expect("failed to create state Output");

        let events_output_plug = PlugOptionsBuilder::create_output("events")
            .qos(Some(2))
            .build(tether_agent)
            .expect("failed to create state Output");

        RemoteControl {
            count_empty_state_sends: None,
            state_output_plug,
            events_output_plug,
            state_send_interval,
            state_max_empty,
            last_update_sent: SystemTime::now(), // last_clip_count_sent: None,
        }
    }
}
