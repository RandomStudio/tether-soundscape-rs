use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use rtrb::{Consumer, Producer, RingBuffer};

struct Model {
    consumer: Consumer<AudioClipInfo>,
    stream: audio::Stream<Audio>,
    clips: Vec<AudioClipInfo>,
}

#[derive(Debug, PartialEq)]
enum ClipState {
    Ready(),
    Playing(f32),
    Complete(),
}

struct AudioClip {
    id: usize,
    reader: audrey::read::BufFileReader,
}

/// ID of the clip, followed by "state"
type AudioClipInfo = (usize, ClipState);

struct Audio {
    sounds: Vec<AudioClip>,
    producer: Producer<AudioClipInfo>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    let (producer, consumer) = RingBuffer::new(2);
    let audio_model = Audio {
        sounds: Vec::new(),
        producer,
    };
    // Initialise the state that we want to live on the audio thread.
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        .sample_rate(96000)
        .build()
        .unwrap();

    stream.play().unwrap();

    Model {
        stream,
        clips: Vec::new(),
        consumer,
    }
}

fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let mut have_ended = vec![];
    let len_frames = buffer.len_frames();

    // Sum all of the sounds onto the buffer.
    for (i, sound) in audio.sounds.iter_mut().enumerate() {
        let mut frame_count = 0;
        let file_frames = sound.reader.frames::<[f32; 2]>().filter_map(Result::ok);
        for (frame, file_frame) in buffer.frames_mut().zip(file_frames) {
            for (sample, file_sample) in frame.iter_mut().zip(&file_frame) {
                *sample += *file_sample;
            }
            frame_count += 1;
        }

        // If the sound yielded less samples than are in the buffer, it must have ended.
        if frame_count < len_frames {
            have_ended.push(i);
            audio.producer.push((sound.id, ClipState::Complete()));
        } else {
            // TODO: calculate progress
            // You may need to calculate total frames/samples (once) at load, save this,
            // inc a counter of each frame and use this as fraction to calculate progress
        }
    }

    // Remove all sounds that have ended.
    for i in have_ended.into_iter().rev() {
        audio.sounds.remove(i);
    }
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    if key == Key::Space {
        let assets = app.assets_path().expect("could not find assets directory");
        let path = assets.join("sounds").join("frog.wav");
        if let Ok(reader) = audrey::open(path) {
            let id = model.clips.len();
            let new_clip = AudioClip { id, reader };
            model.clips.push((id, ClipState::Ready()));
            model
                .stream
                .send(move |audio| {
                    audio.sounds.push(new_clip);
                })
                .ok();
        } else {
            panic!("Failed to load sound");
        }
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if let Ok(receive) = model.consumer.pop() {
        let (id, state) = receive;
        match state {
            ClipState::Complete() => {
                if let Some(to_remove) = model
                    .clips
                    .iter()
                    .enumerate()
                    .find(|(_i, clip_info)| clip_info.0 == id)
                {
                    let (i, _info) = to_remove;
                    model.clips.remove(i);
                } else {
                    panic!("No match for clip id {}", id);
                }
            }
            ClipState::Playing(progress) => {
                // println!("Clip #{} progress {}", received.id, progress);
            }
            ClipState::Ready() => {}
        }
    }
    // if let Ok(received) = &model.consumer.try_recv() {
    //     println!("Received state {:?} from #{}", received.state, received.id);

    //     match received.state {
    //         ClipState::Complete() => {
    //             if let Some(to_remove) = model
    //                 .clips
    //                 .iter()
    //                 .enumerate()
    //                 .find(|(_i, clip)| clip.id == received.id)
    //             {
    //                 let (i, _info) = to_remove;
    //                 model.clips.remove(i);
    //             } else {
    //                 panic!("No match for clip id {}", received.id);
    //             }
    //         }
    //         ClipState::Playing(progress) => {
    //             // println!("Clip #{} progress {}", received.id, progress);
    //         }
    //         ClipState::Ready() => {}
    //     }
    // }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(DARKSLATEGREY);
    draw.text(&format!("playing {} sounds", model.clips.len()));

    draw.to_frame(app, &frame).unwrap();
}
