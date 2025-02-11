use std::path::{Path, PathBuf};

use log::*;
use serde::{Deserialize, Serialize};

use crate::{playback::PanWithRange, utils::parse_optional_panning};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundBank {
    clips: Vec<AudioClipOnDisk>,
    // scenes: Vec<Scene>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudioClipOnDisk {
    name: String,
    path: String,
    // #[serde(default)]
    // frames_count: u32,
    // #[serde(default)]
    // sample_rate: u32,
    volume: Option<f32>,
    pan_position: Option<f32>,
    pan_spread: Option<f32>,
}

// #[derive(Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Scene {
//     pub clips: Vec<String>,
// }

impl AudioClipOnDisk {
    pub fn name(&self) -> &str {
        &self.name
    }
    // pub fn frames_count(&self) -> u32 {
    //     self.frames_count
    // }
    // pub fn sample_rate(&self) -> u32 {
    //     self.sample_rate
    // }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn volume(&self) -> Option<f32> {
        self.volume
    }
    pub fn panning(&self) -> Option<PanWithRange> {
        parse_optional_panning(self.pan_position, self.pan_spread)
    }
}

pub fn get_sound_asset_path(assets_path: PathBuf, base_path: &str) -> String {
    let path = assets_path.join(base_path);
    path.to_str().unwrap().into()
}

impl SoundBank {
    pub fn new(json_path: &Path) -> Self {
        info!("Loading sample bank from {:?} ...", &json_path);
        match std::fs::read_to_string(json_path) {
            Ok(text) => match serde_json::from_str::<SoundBank>(&text) {
                Ok(bank) => {
                    let clips = bank
                        .clips
                        .iter()
                        .map(|sample| {
                            let path_str = get_sound_asset_path(
                                json_path.parent().unwrap().to_path_buf(),
                                &sample.path,
                            );
                            let path = Path::new(&path_str);
                            // let (frames_count, sample_rate) = read_length_and_rate(path, mono_only);
                            let volume = sample.volume;
                            // let panning =
                            //     parse_optional_panning(sample.pan_position, sample.pan_spread);
                            let entry = AudioClipOnDisk {
                                name: String::from(&sample.name),
                                path: String::from(path.to_str().unwrap()),
                                volume,
                                pan_position: sample.pan_position,
                                pan_spread: sample.pan_spread,
                            };
                            debug!("Created sample bank entry OK: {:?}", entry);
                            entry
                        })
                        .collect();
                    // let scenes = bank.scenes;
                    // SoundBank { clips, scenes }
                    SoundBank { clips }
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

    // pub fn scenes(&self) -> &Vec<Scene> {
    //     &self.scenes
    // }
}
