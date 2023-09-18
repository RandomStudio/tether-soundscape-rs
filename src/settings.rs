use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};

const TETHER_HOST: std::net::IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to JSON file with clips array; if omitted, a suitable demo
    /// file will be used
    #[arg(long = "sampleBank")]
    pub sample_bank_path: Option<String>,

    /// Flag to disable GUI and run in text-only mode
    #[arg(long = "headless")]
    pub headless_mode: bool,

    /// How often to update and progress and volume (if fading in/out)
    #[arg(long = "updateInterval", default_value_t = 16)]
    pub update_interval: u64,

    /// Flag to disable Tether connection
    #[arg(long = "tether.disable")]
    pub tether_disable: bool,

    /// The IP address of the Tether MQTT broker (server)
    #[arg(long = "tether.host", default_value_t=TETHER_HOST)]
    pub tether_host: std::net::IpAddr,

    /// ID/Group to use when publishing messages (state, events)
    #[arg(long = "tether.publish.id", default_value_t=String::from("any"))]
    pub tether_publish_id: String,

    /// ID/Group to use for remote control subscription. Useful if you need to restrict
    /// which controller agent you "listen" to. By default, will use wildcard (+)
    #[arg(long = "tether.subscribe.id")]
    pub tether_subscribe_id: Option<String>,

    /// Preferred output device name; use host default device if not supplied
    #[arg(long = "output.device")]
    pub preferred_output_device: Option<String>,

    /// How many channels to use for output; use max available for the device if omitted
    #[arg(long = "output.channels")]
    pub output_channels: Option<u16>,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    /// How often to publish state via Tether (ignored if disabled)
    #[arg(long = "statePublish.updateInterval", default_value_t = 40)]
    pub state_interval: u64,

    /// How many "empty" state messages (no clips currently) playing, will be
    /// allowed before the state publishing temporarily pauses. This avoids message
    /// clutter. Set a value <= 0 to disable this behaviour and ensure messages continuously send.
    #[arg(long = "statePublish.emptyMax", default_value_t = 8)]
    pub state_max_empty: usize,

    /// How often to publish state via Tether (ignored if disabled)
    #[arg(long = "statePublish.disable")]
    pub state_disable: bool,
}
// pub struct ManualSettings {
//     pub fadein_duration: u32,
//     pub fadeout_duration: u32,
//     pub simple_pan_position: f32,
//     pub simple_pan_spread: f32,
//     pub ignore_panning: bool,
// }

// impl ManualSettings {
//     pub fn defaults() -> Self {
//         ManualSettings {
//             fadein_duration: DEFAULT_FADEIN,
//             fadeout_duration: DEFAULT_FADEOUT,
//             simple_pan_position: 0.5,
//             simple_pan_spread: 2.0,
//             ignore_panning: true,
//         }
//     }
// }
