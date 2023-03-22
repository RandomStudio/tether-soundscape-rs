use std::{path::Path, time::Duration};

use nannou::App;
use serde::{Deserialize, Serialize};

pub const UPDATE_INTERVAL: Duration = Duration::from_millis(8);
pub const CLIP_HEIGHT: f32 = 15.;
pub const CLIP_WIDTH: f32 = 200.;
#[derive(Serialize, Deserialize)]
pub struct AudioClipOnDisk {
    name: String,
    path: String,
    length: Option<usize>,
}

impl AudioClipOnDisk {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn length(&self) -> Option<usize> {
        self.length
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}

pub fn get_sound_asset_path(app: &App, base_path: &str) -> String {
    let assets = app.assets_path().expect("could not find assets directory");
    let path = assets.join("sounds").join(base_path);
    path.to_str().unwrap().into()
}

fn calculate_length(path: &std::path::Path) -> usize {
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
                    let length = calculate_length(path);
                    AudioClipOnDisk {
                        name: String::from(&sample.name),
                        path: String::from(path.to_str().unwrap()),
                        length: Some(length),
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
