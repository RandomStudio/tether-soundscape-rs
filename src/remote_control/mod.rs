pub mod publish;
pub mod receive;

use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use tether_agent::{build_topic, PlugDefinition, PlugOptionsBuilder, TetherAgent};

pub struct RemoteControl {
    state_output_plug: PlugDefinition,
    events_output_plug: PlugDefinition,
    input_plugs: HashMap<String, PlugDefinition>,
    state_send_interval: Duration,
    state_max_empty: usize,
    count_empty_state_sends: Option<usize>,
    last_update_sent: SystemTime, // last_clip_count_sent: Option<usize>,
}

impl RemoteControl {
    pub fn new(
        tether_agent: &TetherAgent,
        override_subscribe_id: &Option<String>,
        state_send_interval: Duration,
        state_max_empty: usize,
    ) -> Self {
        let mut input_plugs: HashMap<String, PlugDefinition> = HashMap::new();

        let id = match override_subscribe_id {
            Some(s) => String::from(s),
            None => String::from("+"),
        };

        input_plugs.insert(
            "clipCommands".into(),
            PlugOptionsBuilder::create_input("clipCommands")
                .qos(2)
                .topic(&build_topic("+".into(), &id, "clipCommands"))
                .build(tether_agent)
                .expect("failed to create clipCommands Input"), // tether_agent
                                                                //     .create_input_plug("clipCommands", Some(2), None)
                                                                //     .unwrap(),
        );
        input_plugs.insert(
            "scenes".into(),
            PlugOptionsBuilder::create_input("scenes")
                .qos(2)
                .topic(&build_topic("+".into(), &id, "scenes"))
                .build(tether_agent)
                .expect("failed to create scenes Input"), // tether_agent
                                                          //     .create_input_plug("scenes", Some(2), None)
                                                          //     .unwrap(),
        );
        input_plugs.insert(
            "globalControls".into(),
            PlugOptionsBuilder::create_input("globalControls")
                .qos(2)
                .topic(&build_topic("+".into(), &id, "globalControls"))
                .build(tether_agent)
                .expect("failed to create globalCommands Input"), // tether_agent
        );

        let state_output_plug = PlugOptionsBuilder::create_output("state")
            .qos(0)
            .build(tether_agent)
            .expect("failed to create state Output");

        let events_output_plug = PlugOptionsBuilder::create_output("events")
            .qos(2)
            .build(tether_agent)
            .expect("failed to create state Output");

        RemoteControl {
            count_empty_state_sends: None,
            state_output_plug,
            events_output_plug,
            input_plugs,
            state_send_interval,
            state_max_empty,
            last_update_sent: SystemTime::now(), // last_clip_count_sent: None,
        }
    }
}
