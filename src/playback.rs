use std::{
    fs::File,
    io::BufReader,
    time::{Duration, SystemTime},
};

use rodio::{source::ChannelVolume, Decoder, OutputStreamHandle, Sink, Source};
use tween::{Linear, Tween, Tweener};

use crate::{loader::AudioClipOnDisk, panning::simple_panning_channel_volumes};

// use crate::utils::millis_to_frames;

/// Volume value, duration in milliseconds
type StoredTweener = Tweener<f32, u128, Box<dyn Tween<f32> + Send + Sync>>;

/// Position (in range 0>numChannels-1) and spread (in range 1>numChannels)
pub type PanWithRange = (f32, f32);

pub enum PlaybackPhase {
    Attack(StoredTweener),
    Sustain(),
    Release(SystemTime, StoredTweener),
}

pub struct ClipWithSink {
    id: usize,
    sink: Sink,
    duration: Option<Duration>,
    started: SystemTime,
    last_known_progress: Option<f32>,
    is_looping: bool,
    name: String,
    current_phase: PlaybackPhase,
    current_volume: f32,
}

impl ClipWithSink {
    pub fn new(
        id: usize,
        sample: &AudioClipOnDisk,
        should_loop: bool,
        fade_in: Option<Duration>,
        override_panning: Option<PanWithRange>,
        output_stream_handle: &OutputStreamHandle,
        output_channels: u16,
    ) -> Self {
        let sink = Sink::try_new(output_stream_handle).expect("failed to create sink");

        let file = BufReader::new(File::open(sample.path()).unwrap());
        // let source = Decoder::new(file).unwrap();
        // let duration = source.total_duration();
        let mut duration = None;

        let panning: Option<PanWithRange> = if override_panning.is_some() {
            override_panning
        } else {
            sample.panning()
        };

        let decoder = Decoder::new(file).unwrap();

        let mut source: Option<Box<dyn Source<Item = _> + Send>> = None;

        if let Some((position, spread)) = panning {
            let s = ChannelVolume::new(
                decoder,
                simple_panning_channel_volumes(position, spread, output_channels),
            );
            source = Some(Box::new(s));
        } else {
            source = Some(Box::new(decoder));
        }

        if let Some(src) = source {
            duration = src.total_duration();
            if should_loop {
                sink.append(src.repeat_infinite());
            } else {
                sink.append(src);
            }
        }

        let tween: Box<dyn Tween<f32> + Send + Sync> = Box::new(Linear);
        let stored_tweener = Tweener::new(
            0.,
            sample.volume().unwrap_or(1.0),
            fade_in.unwrap_or(Duration::from_millis(8)).as_millis(),
            tween,
        );

        ClipWithSink {
            id,
            sink,
            duration,
            started: SystemTime::now(),
            last_known_progress: Some(0.),
            name: String::from(sample.name()),
            current_phase: PlaybackPhase::Attack(stored_tweener),
            current_volume: 0.,
            is_looping: should_loop,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.sink.empty()
    }

    pub fn update_progress(&mut self) {
        let elapsed = self.started.elapsed().unwrap_or(Duration::ZERO);

        // Set volume according to phase...
        self.current_volume = match &mut self.current_phase {
            PlaybackPhase::Attack(tween) => tween.move_to(elapsed.as_millis()),
            PlaybackPhase::Sustain() => self.current_volume,
            PlaybackPhase::Release(fade_start, tween) => {
                let elapsed_since_fade_start = fade_start.elapsed().unwrap_or_default();
                tween.move_to(elapsed_since_fade_start.as_millis())
            }
        };

        self.sink.set_volume(self.current_volume);

        // Transition phases automatically in some cases...
        match &mut self.current_phase {
            PlaybackPhase::Attack(tween) => {
                if tween.is_finished() {
                    self.current_phase = PlaybackPhase::Sustain();
                }
            }
            PlaybackPhase::Release(_fade_start, tween) => {
                if tween.is_finished() {
                    self.stop();
                }
            }
            _ => {}
        }

        if let Some(d) = self.duration {
            let progress = (elapsed.as_millis() % d.as_millis()) as f32 / d.as_millis() as f32;
            self.last_known_progress = Some(progress);
        }
    }

    pub fn progress(&self) -> Option<f32> {
        self.last_known_progress
    }

    pub fn stop(&self) {
        self.sink.clear();
    }

    pub fn fade_out(&mut self, duration: Duration) {
        let tween: Box<dyn Tween<f32> + Send + Sync> = Box::new(Linear);
        let stored_tweener = Tweener::new(self.current_volume, 0., duration.as_millis(), tween);

        self.current_phase = PlaybackPhase::Release(SystemTime::now(), stored_tweener);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn current_volume(&self) -> f32 {
        self.current_volume
    }

    pub fn is_looping(&self) -> bool {
        self.is_looping
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn phase(&self) -> &PlaybackPhase {
        &self.current_phase
    }
}
