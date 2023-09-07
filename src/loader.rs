use std::path::{Path, PathBuf};

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::remote_control::PanWithRange;

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
    panning: Option<PanWithRange>,
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
        self.panning
    }
}

pub fn get_sound_asset_path(assets_path: PathBuf, base_path: &str) -> String {
    let path = assets_path.join("sounds").join(base_path);
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
                        .enumerate()
                        .map(|(_i, sample)| {
                            let path_str = get_sound_asset_path(
                                Path::new("assets").to_path_buf(),
                                &sample.path,
                            );
                            let path = Path::new(&path_str);
                            // let (frames_count, sample_rate) = read_length_and_rate(path, mono_only);
                            let volume = sample.volume;
                            let panning = sample.panning;
                            let entry = AudioClipOnDisk {
                                name: String::from(&sample.name),
                                path: String::from(path.to_str().unwrap()),
                                volume,
                                panning,
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
