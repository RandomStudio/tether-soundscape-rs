use audio::Host;
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

struct Model {
    audio_host: Host,
    streams: Vec<audio::Stream<Audio>>,
}

type Callback = fn();
struct Audio {
    sound: audrey::read::BufFileReader,
    is_completed: bool,
    on_completed: Callback,
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

    Model {
        audio_host,
        streams: Vec::new(),
    }
}

fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    if !audio.is_completed {
        let len_frames = buffer.len_frames();

        let sound = &mut audio.sound;
        // Sum all of the sounds onto the buffer.
        let mut frame_count = 0;
        let file_frames = sound.frames::<[f32; 2]>().filter_map(Result::ok);
        for (frame, file_frame) in buffer.frames_mut().zip(file_frames) {
            for (sample, file_sample) in frame.iter_mut().zip(&file_frame) {
                *sample += *file_sample;
            }
            frame_count += 1;
        }

        // If the sound yielded less samples than are in the buffer, it must have ended.
        if frame_count < len_frames {
            audio.is_completed = true;
            println!("Sound ended!");
            (audio.on_completed)();
        }
    }
}

fn simple_callback() {
    println!("Boo!");
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    if key == Key::Space {
        let assets = app.assets_path().expect("could not find assets directory");
        let path = assets.join("sounds").join("frog.wav");
        if let Ok(sound) = audrey::open(path) {
            let audio_model = Audio {
                sound,
                is_completed: false,
                on_completed: simple_callback,
            };
            // Initialise the state that we want to live on the audio thread.
            let stream = model
                .audio_host
                .new_output_stream(audio_model)
                .render(audio)
                .sample_rate(96000)
                .build()
                .unwrap();

            stream
                .send(move |audio| {
                    // audio.sound = sound;
                    println!("audio model: {}", audio.is_completed);
                })
                .ok();

            model.streams.push(stream);
        } else {
            panic!("Failed to load sound");
        }
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    // if let Some(completed) = model.streams.iter().find(|s| s.is_paused()) {
    //     println!("Completed stream");
    // }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(DARKSLATEGREY);
    draw.text(&format!("playing {} sounds", model.streams.len()));

    draw.to_frame(app, &frame).unwrap();
}
