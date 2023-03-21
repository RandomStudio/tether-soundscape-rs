use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::AudioClipMetadata;

#[derive(Serialize, Deserialize)]
struct AudioClipOnDisk {
    name: String,
    path: String,
}

fn calculate_length(path: &std::path::Path) -> usize {
    let mut reader = audrey::open(path).unwrap();
    let mut count = 0;
    reader.frames::<[f32; 2]>().for_each(|_f| count += 1);
    count
}

pub fn load_sample_bank(json_path: &Path) -> Vec<AudioClipMetadata> {
    match std::fs::read_to_string(json_path) {
        Ok(text) => match serde_json::from_str::<Vec<AudioClipOnDisk>>(&text) {
            Ok(samples) => samples
                .iter()
                .enumerate()
                .map(|(i, sample)| {
                    let sample_path = Path::new(&sample.path);
                    let length = calculate_length(sample_path);
                    AudioClipMetadata {
                        id: i,
                        name: String::from(&sample.name),
                        length,
                        state: crate::PlaybackState::Ready(),
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
