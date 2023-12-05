use std::sync::{Arc, Mutex};

use rodio::Source;

/// A continuously playing audio source from the NDS data.
/// Tries to play pre-buffered audio provided by the DS, otherwise
/// tries to extend the last played audio. If that still ends
/// before more audio is available, plays a frame of silence.
#[derive(Debug, Clone)]
pub struct NdsAudio {
    position: usize,
    playback_mode: PlaybackMode,
    queue: Arc<Mutex<Vec<i16>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackMode {
    Regular(Vec<i16>),
    Extra([i16; NdsAudio::CHANNELS as usize]),
    Empty,
}

impl NdsAudio {
    pub const CHANNELS: u16 = 2;
    pub const SAMPLE_RATE: u32 = 32824;

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
                    let extension = audio[audio.len() - NdsAudio::CHANNELS as usize..]
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

impl Iterator for NdsAudio {
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
                let len = Self::CHANNELS as usize * 10;

                read_position = read_position.min(len - 1);
                (
                    sample[read_position % Self::CHANNELS as usize],
                    self.position >= len,
                )
            }
            PlaybackMode::Empty => {
                let len = Self::CHANNELS as usize * Self::SAMPLE_RATE as usize / 60;

                (0, self.position >= len)
            }
        };

        if change_mode {
            self.change_mode()
        }

        Some(sample)
    }
}

impl Source for NdsAudio {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        NdsAudio::CHANNELS
    }

    fn sample_rate(&self) -> u32 {
        NdsAudio::SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
