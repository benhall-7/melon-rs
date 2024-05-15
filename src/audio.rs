use std::sync::{Arc, Mutex};

use rodio::{OutputStream, Sink, Source};

use crate::melon::nds::Nds;

pub struct Audio {
    _output_stream: OutputStream,
    _game_audio: Sink,
    game_stream: Arc<Mutex<Vec<i16>>>,
}

impl Audio {
    pub fn new() -> Self {
        let (output_stream, stream_handle) = OutputStream::try_default().unwrap();
        let game_stream: Arc<Mutex<Vec<i16>>> = Default::default();
        let audio = Sink::try_new(&stream_handle).unwrap();
        audio.append(Stream::new(game_stream.clone()));

        Self {
            _output_stream: output_stream,
            _game_audio: audio,
            game_stream,
        }
    }

    pub fn get_game_stream(&mut self) -> Arc<Mutex<Vec<i16>>> {
        self.game_stream.clone()
    }
}

/// A continuously playing audio source from the NDS data.
/// Tries to play pre-buffered audio provided by the DS, otherwise
/// tries to extend the last played audio. If that still ends
/// before more audio is available, plays a frame of silence.
#[derive(Debug, Clone)]
pub struct Stream {
    position: usize,
    playback_mode: PlaybackMode,
    queue: Arc<Mutex<Vec<i16>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackMode {
    Regular(Vec<i16>),
    Extra([i16; Nds::AUDIO_CHANNELS as usize]),
    Empty,
}

impl Stream {
    pub fn new(queue: Arc<Mutex<Vec<i16>>>) -> Self {
        Self {
            position: 0,
            playback_mode: PlaybackMode::Empty,
            queue,
        }
    }

    fn change_mode(&mut self) {
        self.position = 0;

        match &mut self.queue.lock() {
            Ok(queue) => {
                if !queue.is_empty() {
                    let next_samples = std::mem::take(queue.as_mut());
                    self.playback_mode = PlaybackMode::Regular(next_samples);
                } else if let PlaybackMode::Regular(audio) = &self.playback_mode {
                    let extension = audio[audio.len() - Nds::AUDIO_CHANNELS as usize..]
                        .try_into()
                        .expect("slice must have correct length");
                    self.playback_mode = PlaybackMode::Extra(extension)
                } else {
                    self.playback_mode = PlaybackMode::Empty;
                }
            }
            Err(_) => self.playback_mode = PlaybackMode::Empty,
        }
    }
}

impl Iterator for Stream {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let mut read_position = self.position;
        self.position = read_position + 1;

        let (sample, change_mode) = match &self.playback_mode {
            PlaybackMode::Regular(audio) => {
                let len = audio.len();

                read_position = read_position.min(len - 1);
                (audio[read_position], self.position >= len)
            }
            PlaybackMode::Extra(sample) => {
                // play back the extra samples 10 times
                let len = Nds::AUDIO_CHANNELS as usize * 10;

                read_position = read_position.min(len - 1);
                (
                    sample[read_position % Nds::AUDIO_CHANNELS as usize],
                    self.position >= len,
                )
            }
            PlaybackMode::Empty => {
                let len = Nds::AUDIO_CHANNELS as usize * Nds::AUDIO_SAMPLE_RATE as usize / 60;

                (0, self.position >= len)
            }
        };

        if change_mode {
            self.change_mode()
        }

        Some(sample)
    }
}

impl Source for Stream {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        Nds::AUDIO_CHANNELS
    }

    fn sample_rate(&self) -> u32 {
        Nds::AUDIO_SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
