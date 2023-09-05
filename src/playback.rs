use std::{
    fs::File,
    io::BufReader,
    time::{Duration, SystemTime},
};

use log::debug;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use rtrb::{Consumer, Producer};
use tween::{Linear, QuadIn, SineInOut, Tween, Tweener};

use crate::loader::AudioClipOnDisk;

// use crate::utils::millis_to_frames;

/// Volume value, duration IN FRAMES
type StoredTweener = Tweener<f32, u32, Box<dyn Tween<f32> + Send + Sync>>;

pub struct ClipWithSink {
    sink: Sink,
    duration: Option<Duration>,
    started: SystemTime,
}

impl ClipWithSink {
    pub fn new(sample: &AudioClipOnDisk, output_stream_handle: &OutputStreamHandle) -> Self {
        let file = BufReader::new(File::open(sample.path()).unwrap());
        let source = Decoder::new(file).unwrap();

        let duration = source.total_duration();

        let sink = Sink::try_new(output_stream_handle).expect("failed to create sink");
        sink.append(source);

        ClipWithSink {
            sink,
            duration,
            started: SystemTime::now(),
        }
    }

    pub fn sink(&self) -> &Sink {
        &self.sink
    }

    pub fn is_completed(&self) -> bool {
        self.sink.empty()
    }

    pub fn progress(&self) -> Option<f32> {
        match self.duration {
            None => None,
            Some(d) => {
                let elapsed = self.started.elapsed().unwrap_or(Duration::ZERO);
                let progress = elapsed.as_millis() as f32 / d.as_millis() as f32;
                Some(progress)
            }
        }
    }
}

// pub struct BufferedClip {
//     id: usize,
//     current_volume: f32,
//     reader: audrey::read::BufFileReader,
//     frames_played: u32,
//     phase: PlaybackPhase,
//     per_channel_volume: Vec<f32>,
//     source_channels_count: u32,
// }

// /// Start volume, End volume, Duration IN MILLISECONDS
// pub type Fade = (f32, f32, u32);

// pub enum PlaybackPhase {
//     Attack(StoredTweener),
//     Sustain(),
//     Release(StoredTweener),
//     Complete(),
// }

// impl BufferedClip {
//     pub fn new(
//         id: usize,
//         fade_in: Option<Fade>,
//         reader: Reader<BufReader<File>>,
//         per_channel_volume: Vec<f32>,
//     ) -> Self {
//         let start_volume = match fade_in {
//             Some(fade) => {
//                 let (start, _end, _duration) = fade;
//                 start
//             }
//             None => 0.,
//         };
//         let sample_rate = reader.description().sample_rate();
//         let channels = reader.description().channel_count();
//         BufferedClip {
//             id,
//             reader,
//             frames_played: 0,
//             current_volume: start_volume,
//             phase: match fade_in {
//                 Some((start, end, duration_ms)) => {
//                     let tween: Box<dyn Tween<f32> + Send + Sync> = Box::new(QuadIn);

//                     let stored_tweener = Tweener::new(
//                         start,
//                         end,
//                         millis_to_frames(duration_ms, sample_rate),
//                         tween,
//                     );
//                     PlaybackPhase::Attack(stored_tweener)
//                 }
//                 None => {
//                     let tween: Box<dyn Tween<f32> + Send + Sync> = Box::new(Linear);
//                     let stored_tweener = Tweener::new(0., 1.0, 1, tween);
//                     PlaybackPhase::Attack(stored_tweener)
//                 }
//             },
//             per_channel_volume,
//             source_channels_count: channels,
//         }
//     }

//     pub fn fade_out(&mut self, duration_frames: u32) {
//         let tween: Box<dyn Tween<f32> + Send + Sync> = Box::new(SineInOut);
//         let stored_tweener = Tweener::new(self.current_volume, 0., duration_frames, tween);
//         debug!("sound sustain => release, fade over {}fr", duration_frames);
//         self.phase = PlaybackPhase::Release(stored_tweener);
//     }
// }

// /// audio -> main: ID of the clip, followed by frames played (count), followed by current volume
// pub type ProgressUpdate = (usize, u32, f32);

// /// audio -> main: ID of the clip
// pub type CompleteUpdate = usize;

// /// audio <- main: ID of the clip
// pub type RequestUpdate = usize;

// pub enum PlaybackState {
//     Ready(),
//     Playing(u32),
// }

// pub struct Audio {
//     sounds: Vec<BufferedClip>,
//     tx_progress: Producer<ProgressUpdate>,
//     tx_complete: Producer<CompleteUpdate>,
//     rx_request: Consumer<RequestUpdate>,
// }

// impl Audio {
//     pub fn new(
//         tx_progress: Producer<ProgressUpdate>,
//         tx_complete: Producer<CompleteUpdate>,
//         rx_request: Consumer<RequestUpdate>,
//     ) -> Self {
//         Audio {
//             sounds: Vec::new(),
//             tx_progress,
//             tx_complete,
//             rx_request,
//         }
//     }
//     pub fn add_sound(&mut self, new_clip: BufferedClip) {
//         self.sounds.push(new_clip);
//     }
//     pub fn remove_sound(&mut self, id: usize) {
//         if let Some(to_remove) = self
//             .sounds
//             .iter()
//             .enumerate()
//             .find(|(_index, s)| s.id == id)
//         {
//             let (index, _s) = to_remove;
//             self.sounds.remove(index);
//         }
//     }

//     pub fn fadeout_sound(&mut self, id: usize, duration_frames: u32) {
//         if let Some(to_fadeout) = self
//             .sounds
//             .iter_mut()
//             .enumerate()
//             .find(|(_index, s)| s.id == id)
//         {
//             let (_index, s) = to_fadeout;
//             s.fade_out(duration_frames);
//         }
//     }
// }

// pub fn render_audio_multichannel(audio: &mut Audio, buffer: &mut Buffer) {
//     let mut have_ended = vec![];
//     let len_frames = buffer.len_frames().to_u32().unwrap();

//     // Sum all of the sounds onto the buffer.
//     for (i, sound) in audio.sounds.iter_mut().enumerate() {
//         let mut frame_count: u32 = 0;
//         let file_frames = sound
//             .reader
//             .frames::<[f32; 1]>() //
//             .filter_map(Result::ok);
//         for (buffer_frame, file_frame) in buffer.frames_mut().zip(file_frames) {
//             for (index, output_sample) in buffer_frame.iter_mut().enumerate() {
//                 let this_channel_volume = sound.per_channel_volume[index];
//                 *output_sample += file_frame[0] * sound.current_volume * this_channel_volume;
//             }
//             frame_count += 1;
//         }

//         handle_audio_progress(
//             frame_count,
//             len_frames,
//             sound,
//             &mut have_ended,
//             i,
//             &mut audio.tx_progress,
//             &mut audio.tx_complete,
//             &mut audio.rx_request,
//         );
//     }

//     // Remove all sounds that have ended.
//     for i in have_ended.into_iter().rev() {
//         audio.sounds.remove(i);
//     }
// }

// pub fn render_audio_stereo(audio: &mut Audio, buffer: &mut Buffer) {
//     let mut have_ended = vec![];
//     let len_frames = buffer.len_frames().to_u32().unwrap();

//     // Sum all of the sounds onto the buffer.
//     for (i, sound) in audio.sounds.iter_mut().enumerate() {
//         let mut frame_count: u32 = 0;
//         if sound.source_channels_count == 2 {
//             // STEREO SOURCE - zip (use as is)

//             let file_frames = sound
//                 .reader
//                 .frames::<[f32; 2]>() //
//                 .filter_map(Result::ok);
//             for (buffer_frame, file_frame) in buffer.frames_mut().zip(file_frames) {
//                 for (output_sample, file_sample) in buffer_frame.iter_mut().zip(&file_frame) {
//                     *output_sample += *file_sample * sound.current_volume;
//                 }
//                 frame_count += 1;
//             }
//         } else {
//             // MONO SOURCE - duplicate channels

//             // let len_frames = len_frames * 2;

//             let mut file_frames = sound
//                 .reader
//                 .frames::<[f32; 1]>() //
//                 .filter_map(Result::ok);

//             for buffer_frame in buffer.frames_mut() {
//                 if let Some(file_frame) = file_frames.next() {
//                     for output_sample in buffer_frame.iter_mut() {
//                         *output_sample += file_frame[0];
//                     }
//                     frame_count += 1;
//                 }
//             }
//         }

//         handle_audio_progress(
//             frame_count,
//             len_frames,
//             sound,
//             &mut have_ended,
//             i,
//             &mut audio.tx_progress,
//             &mut audio.tx_complete,
//             &mut audio.rx_request,
//         );
//     }

//     // Remove all sounds that have ended.
//     for i in have_ended.into_iter().rev() {
//         audio.sounds.remove(i);
//     }
// }

// fn handle_audio_progress(
//     frame_count: u32,
//     len_frames: u32,
//     sound: &mut BufferedClip,
//     have_ended: &mut Vec<usize>,
//     i: usize,
//     tx_progress: &mut Producer<ProgressUpdate>,
//     tx_complete: &mut Producer<CompleteUpdate>,
//     rx_request: &mut Consumer<RequestUpdate>,
// ) {
//     // If
//     // - sound yielded less samples than are in the buffer, it must have ended
//     // - or we're in the Complete phase
//     if frame_count < len_frames || matches!(&sound.phase, PlaybackPhase::Complete()) {
//         if !tx_complete.is_full() {
//             have_ended.push(i);
//             tx_complete.push(sound.id).unwrap();
//         }
//     } else {
//         sound.frames_played += frame_count;

//         sound.current_volume = match &mut sound.phase {
//             PlaybackPhase::Attack(tween) => tween.move_by(frame_count),
//             PlaybackPhase::Sustain() => sound.current_volume,
//             PlaybackPhase::Release(tween) => tween.move_by(frame_count),
//             PlaybackPhase::Complete() => 0.,
//         };

//         if let PlaybackPhase::Attack(tween) = &mut sound.phase {
//             if tween.is_finished() {
//                 debug!("sound attack => sustain");
//                 sound.phase = PlaybackPhase::Sustain();
//             }
//         }

//         if let PlaybackPhase::Release(tween) = &sound.phase {
//             if tween.is_finished() {
//                 debug!("sound release => complete");
//                 sound.phase = PlaybackPhase::Complete();
//             }
//         }

//         if let Ok(receive_id) = rx_request.pop() {
//             if sound.id == receive_id && !tx_progress.is_full() {
//                 tx_progress
//                     .push((sound.id, sound.frames_played, sound.current_volume))
//                     .expect("failed to send progress");
//             }
//         }
//     }
// }
