use std::time::Duration;
use std::vec::Drain;

use rodio::Source;

pub struct NdsAudio {
    position: usize,
    data: Vec<i16>,
}

impl NdsAudio {
    pub fn new(data: Vec<i16>) -> Self {
        Self { position: 0, data }
    }
}

impl Iterator for NdsAudio {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.position;
        if position < self.data.len() {
            let val = self.data[position];
            self.position += 1;
            Some(val)
        } else {
            None
        }
    }
}

impl Source for NdsAudio {
    fn current_frame_len(&self) -> Option<usize> {
        Some((self.data.len() - self.position) / 2)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        32824
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(Duration::from_secs_f32(1.0 / 60.0))
    }
}
