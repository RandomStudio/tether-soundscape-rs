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
pub const OUTPUT_CHANNELS: u32 = 2;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    #[arg(long = "sampleRate",default_value_t=SAMPLE_RATE)]
    pub sample_rate: u32,

    /// Flag to disable Tether connection
    #[arg(long = "tether.disable")]
    pub tether_disable: bool,

    /// The IP address of the Tether MQTT broker (server)
    #[arg(long = "tether.host", default_value_t=TETHER_HOST)]
    pub tether_host: std::net::IpAddr,

    /// Number of output channels to use
    #[arg(long = "outputChannels", default_value_t=OUTPUT_CHANNELS)]
    pub output_channels: u32,

    /// Preferred output device name; use host default device if not supplied
    #[arg(long = "outputDevice")]
    pub preferred_output_device: Option<String>,
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
