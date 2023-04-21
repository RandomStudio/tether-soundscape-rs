use clap::Parser;
use std::time::Duration;

pub const UPDATE_INTERVAL: Duration = Duration::from_millis(16);
pub const MIN_RADIUS: f32 = 100.;
pub const LINE_THICKNESS: f32 = 2.;
pub const DEFAULT_FADEIN: u32 = 2000;
pub const DEFAULT_FADEOUT: u32 = 2000;
pub const RING_BUFFER_SIZE: usize = 32;

pub const SAMPLE_RATE: u32 = 48000;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    #[arg(long = "sampleRate",default_value_t=SAMPLE_RATE)]
    pub sample_rate: u32,
}
pub struct Settings {
    pub fadein_duration: u32,
    pub fadeout_duration: u32,
}

impl Settings {
    pub fn defaults() -> Self {
        Settings {
            fadein_duration: DEFAULT_FADEIN,
            fadeout_duration: DEFAULT_FADEOUT,
        }
    }
}
