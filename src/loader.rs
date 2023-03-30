use std::path::Path;

use nannou::App;
use serde::{Deserialize, Serialize};

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

fn read_length_and_rate(path: &std::path::Path) -> (u32, u32) {
    let mut reader = audrey::open(path).unwrap();
    let mut count = 0;
    reader.frames::<[f32; 2]>().for_each(|_f| count += 1);
    let description = reader.description();
    let sample_rate = description.sample_rate();
    (count, sample_rate)
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
                    let (frames_count, sample_rate) = read_length_and_rate(path);
                    let volume = sample.volume;
                    AudioClipOnDisk {
                        name: String::from(&sample.name),
                        path: String::from(path.to_str().unwrap()),
                        frames_count,
                        sample_rate,
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
