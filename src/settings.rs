use clap::Parser;
use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

pub const UPDATE_INTERVAL: Duration = Duration::from_millis(16);
pub const MIN_RADIUS: f32 = 100.;
pub const LINE_THICKNESS: f32 = 2.;
pub const DEFAULT_FADEIN: u32 = 2000;
pub const DEFAULT_FADEOUT: u32 = 2000;
pub const RING_BUFFER_SIZE: usize = 32;
const TETHER_HOST: std::net::IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub const SAMPLE_RATE: u32 = 48000;

pub const TEST_BANK_STEREO: &str = "./test_stereo.json";
pub const TEST_BANK_MONO: &str = "./test_mono.json";

pub fn pick_default_sample_bank(multi_channel_mode: bool) -> String {
    if multi_channel_mode {
        String::from(TEST_BANK_MONO)
    } else {
        String::from(TEST_BANK_STEREO)
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to JSON file with clips array; if omitted, a suitable demo
    /// file will be used
    #[arg(long = "sampleBank")]
    pub sample_bank_path: Option<String>,

    /// Expected sample rate of input clips in Hz; all clips should match
    #[arg(long = "sampleRate",default_value_t=SAMPLE_RATE)]
    pub sample_rate: u32,

    /// Flag to disable Tether connection
    #[arg(long = "tether.disable")]
    pub tether_disable: bool,

    /// The IP address of the Tether MQTT broker (server)
    #[arg(long = "tether.host", default_value_t=TETHER_HOST)]
    pub tether_host: std::net::IpAddr,

    /// Preferred output device name; use host default device if not supplied
    #[arg(long = "outputDevice")]
    pub preferred_output_device: Option<String>,

    /// MultiChannel mode expects mono clips only; allows "panning"
    /// within 2 or more channels
    #[arg(long = "multiChannel")]
    pub multichannel_mode: bool,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,
}
pub struct ManualSettings {
    pub fadein_duration: u32,
    pub fadeout_duration: u32,
    pub simple_pan_position: f32,
    pub simple_pan_spread: f32,
    pub ignore_panning: bool,
}

impl ManualSettings {
    pub fn defaults() -> Self {
        ManualSettings {
            fadein_duration: DEFAULT_FADEIN,
            fadeout_duration: DEFAULT_FADEOUT,
            simple_pan_position: 0.5,
            simple_pan_spread: 2.0,
            ignore_panning: true,
        }
    }
}
