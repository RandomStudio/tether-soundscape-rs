use std::path::{Path, PathBuf};

use log::{error, info};
use nannou::App;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundBank {
    clips: Vec<AudioClipOnDisk>,
    scenes: Vec<Scene>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioClipOnDisk {
    name: String,
    path: String,
    #[serde(default)]
    frames_count: u32,
    #[serde(default)]
    sample_rate: u32,
    volume: Option<f32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub clips: Vec<String>,
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

pub fn get_sound_asset_path(assets_path: PathBuf, base_path: &str) -> String {
    let path = assets_path.join("sounds").join(base_path);
    path.to_str().unwrap().into()
}

fn read_length_and_rate(path: &std::path::Path, mono_only: bool) -> (u32, u32) {
    let mut reader =
        audrey::open(path).expect(&format!("Failed to load sound file with path {:?}", &path));
    let mut count = 0;
    if reader.description().channel_count() == 2 {
        if mono_only {
            error!("Clip {:?} has 2 channels", path);
            panic!("In multichannel mode, you may only load mono clips!");
        }
        reader.frames::<[f32; 2]>().for_each(|_f| count += 1);
    } else {
        reader.frames::<[f32; 1]>().for_each(|_f| count += 1);
    }
    let description = reader.description();
    let sample_rate = description.sample_rate();
    (count, sample_rate)
}

impl SoundBank {
    pub fn new(app: &App, json_path: &Path, mono_only: bool) -> Self {
        info!("Loading sample bank from {:?} ...", &json_path);
        match std::fs::read_to_string(json_path) {
            Ok(text) => match serde_json::from_str::<SoundBank>(&text) {
                Ok(bank) => {
                    let clips = bank
                        .clips
                        .iter()
                        .enumerate()
                        .map(|(_i, sample)| {
                            let path_str = get_sound_asset_path(
                                app.assets_path().expect("failed to fetch asset path"),
                                sample.path(),
                            );
                            let path = Path::new(&path_str);
                            let (frames_count, sample_rate) = read_length_and_rate(path, mono_only);
                            let volume = sample.volume;
                            AudioClipOnDisk {
                                name: String::from(&sample.name),
                                path: String::from(path.to_str().unwrap()),
                                frames_count,
                                sample_rate,
                                volume,
                            }
                        })
                        .collect();
                    let scenes = bank.scenes;
                    SoundBank { clips, scenes }
                }
                Err(e) => {
                    panic!("Failed to parse sample bank JSON: {}", e);
                }
            },
            Err(e) => {
                panic!("Failed to load sample bank JSON: {}", e);
            }
        }
    }

    pub fn clips(&self) -> &Vec<AudioClipOnDisk> {
        &self.clips
    }

    pub fn scenes(&self) -> &Vec<Scene> {
        &self.scenes
    }
}
