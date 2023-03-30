use std::path::Path;

use nannou::App;
use serde::{Deserialize, Serialize};

use crate::settings::SAMPLE_RATE;

#[derive(Serialize, Deserialize)]
pub struct AudioClipOnDisk {
    name: String,
    path: String,
    #[serde(default)]
    frames_count: u32,
    #[serde(default)]
    sample_rate: u32,
    volume: Option<f32>,
}

impl AudioClipOnDisk {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn volume(&self) -> Option<f32> {
        self.volume
    }
}

pub fn get_sound_asset_path(app: &App, base_path: &str) -> String {
    let assets = app.assets_path().expect("could not find assets directory");
    let path = assets.join("sounds").join(base_path);
    path.to_str().unwrap().into()
}

fn fetch_frames_count(path: &std::path::Path) -> u32 {
    let mut reader = audrey::open(path).unwrap();
    let mut count = 0;
    reader.frames::<[f32; 2]>().for_each(|_f| count += 1);
    count
}

pub fn load_sample_bank(app: &App, json_path: &Path) -> Vec<AudioClipOnDisk> {
    match std::fs::read_to_string(json_path) {
        Ok(text) => match serde_json::from_str::<Vec<AudioClipOnDisk>>(&text) {
            Ok(samples) => samples
                .iter()
                .enumerate()
                .map(|(_i, sample)| {
                    let path_str = get_sound_asset_path(app, sample.path());
                    let path = Path::new(&path_str);
                    let frames_count = fetch_frames_count(path);
                    let volume = sample.volume;
                    AudioClipOnDisk {
                        name: String::from(&sample.name),
                        path: String::from(path.to_str().unwrap()),
                        frames_count,
                        // TODO: calculate rather than assume sample rate
                        sample_rate: SAMPLE_RATE,
                        volume,
                    }
                })
                .collect(),
            Err(e) => {
                panic!("Failed to parse sample bank JSON: {}", e);
            }
        },
        Err(e) => {
            panic!("Failed to load sample bank JSON: {}", e);
        }
    }
}
