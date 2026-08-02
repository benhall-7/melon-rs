use std::time::Duration;

use rodio::{OutputStream, Sink, Source};
use rtrb::{Consumer, Producer, RingBuffer};

use crate::melon::nds::Nds;

/// An open output device. Playback stops when this is dropped.
pub struct Playback {
    _output_stream: OutputStream,
    _sink: Sink,
}

impl Playback {
    /// Opens the default device, returning it alongside the producer handle.
    pub fn start() -> (Self, Audio) {
        let (output_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let (audio, stream) = Audio::new();

        sink.append(stream);

        let playback = Playback {
            _output_stream: output_stream,
            _sink: sink,
        };

        (playback, audio)
    }
}

/// The emulator's end of the audio path.
pub struct Audio {
    pairs: Producer<[i16; 2]>,
    pace: Pace,
}

impl Audio {
    const CAPACITY: usize = 2 * Pace::TARGET;

    /// Builds the ring, returning its two ends.
    fn new() -> (Self, Stream) {
        let (pairs, consumer) = RingBuffer::new(Self::CAPACITY);

        let audio = Audio {
            pairs,
            pace: Pace::new(),
        };
        let stream = Stream::new(consumer);

        (audio, stream)
    }

    /// Queues one emulated frame of audio, reporting the skew to resample with
    pub fn submit(&mut self, frame: &[[i16; 2]]) -> f64 {
        // Whatever will not fit is dropped, which is what bounds latency at the
        // ring's capacity.
        let _ = self.pairs.push_partial_slice(frame);

        self.pace.adjust(Self::CAPACITY - self.pairs.slots())
    }
}

/// The device's end of the ring, handing out one channel per call.
struct Stream {
    pairs: Consumer<[i16; 2]>,
    pair: [i16; 2],
    channel: usize,
}

impl Stream {
    fn new(pairs: Consumer<[i16; 2]>) -> Self {
        Stream {
            pairs,
            pair: [0, 0],
            channel: 0,
        }
    }
}

impl Iterator for Stream {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        if self.channel == 0 {
            // A dry ring keeps the last pair, as melonDS does. Nothing is
            // committed, so real audio resumes the moment it arrives.
            if let Ok(pair) = self.pairs.pop() {
                self.pair = pair;
            }
        }

        let sample = self.pair[self.channel];
        self.channel = (self.channel + 1) % Nds::AUDIO_CHANNELS as usize;

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

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Matches the console's audio rate to the device's.
///
/// The backlog is the integral of the difference between the two rates, so
/// stepping the skew while it sits outside the target range cancels a constant
/// clock offset and then holds still.
#[derive(Debug)]
struct Pace {
    skew: f64,
}

impl Pace {
    /// Backlog to hold: one emulated frame of audio, about 17ms.
    const TARGET: usize = Nds::AUDIO_SAMPLE_RATE as usize / 60;

    /// How far the backlog may stray before correcting. Whatever is left over
    /// after this is the margin that absorbs a late frame.
    const SLACK: usize = Self::TARGET / 8;

    /// Where one emulated frame is worth exactly a sixtieth of a second.
    const CALIBRATED: f64 = 60.0 / Nds::NATIVE_FRAME_RATE;

    /// One step is 0.01%, or 0.17 cents of pitch, and cancels 100ppm of error.
    const STEP: f64 = Self::CALIBRATED * 0.0001;

    const MIN: f64 = Self::CALIBRATED * 0.995;
    const MAX: f64 = Self::CALIBRATED * 1.005;

    fn new() -> Self {
        Pace {
            skew: Self::CALIBRATED,
        }
    }

    /// Reports the skew to resample with, for a ring holding `backlog` pairs.
    fn adjust(&mut self, backlog: usize) -> f64 {
        // Pairs per frame scale with the reciprocal of the skew.
        if backlog < Self::TARGET - Self::SLACK {
            self.skew -= Self::STEP;
        } else if backlog > Self::TARGET + Self::SLACK {
            self.skew += Self::STEP;
        }

        self.skew = self.skew.clamp(Self::MIN, Self::MAX);

        self.skew
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both ends of a ring, skipping the startup fill so the backlog is known.
    fn ring(pairs: &[[i16; 2]]) -> (Audio, Stream) {
        let (mut producer, consumer) = RingBuffer::new(Audio::CAPACITY);
        let _ = producer.push_partial_slice(pairs);

        let audio = Audio {
            pairs: producer,
            pace: Pace::new(),
        };

        (audio, Stream::new(consumer))
    }

    fn play(stream: &mut Stream, samples: usize) -> Vec<i16> {
        (0..samples).map(|_| stream.next().unwrap()).collect()
    }

    #[test]
    fn pairs_come_out_left_then_right() {
        let (_audio, mut stream) = ring(&[[1, 2], [3, 4]]);

        assert_eq!(play(&mut stream, 4), vec![1, 2, 3, 4]);
    }

    #[test]
    fn a_dry_ring_holds_the_last_pair() {
        let (_audio, mut stream) = ring(&[[1, 2]]);

        assert_eq!(play(&mut stream, 6), vec![1, 2, 1, 2, 1, 2]);
    }

    #[test]
    fn a_ring_that_never_filled_plays_silence() {
        let (_audio, mut stream) = ring(&[]);

        assert_eq!(play(&mut stream, 4), vec![0, 0, 0, 0]);
    }

    #[test]
    fn submitting_into_a_thin_ring_asks_for_more_audio() {
        let (mut audio, _stream) = ring(&[]);

        assert!(audio.submit(&[]) < Pace::CALIBRATED);
    }

    #[test]
    fn submitting_more_than_fits_stays_bounded() {
        let (mut audio, stream) = ring(&[]);
        let frame = vec![[1, 2]; Pace::TARGET];

        for _ in 0..4 {
            audio.submit(&frame);
        }

        assert_eq!(stream.pairs.slots(), Audio::CAPACITY);
    }

    #[test]
    fn a_thin_backlog_steps_the_skew_down() {
        assert!(Pace::new().adjust(0) < Pace::CALIBRATED);
    }

    #[test]
    fn a_deep_backlog_steps_the_skew_up() {
        assert!(Pace::new().adjust(Audio::CAPACITY) > Pace::CALIBRATED);
    }

    #[test]
    fn a_backlog_in_range_is_left_alone() {
        assert_eq!(Pace::new().adjust(Pace::TARGET), Pace::CALIBRATED);
    }

    /// The correction has to survive the backlog recovering, or the drift it
    /// cancels would simply resume.
    #[test]
    fn a_correction_outlives_the_backlog_that_caused_it() {
        let mut pace = Pace::new();
        let corrected = pace.adjust(0);

        assert_eq!(pace.adjust(Pace::TARGET), corrected);
    }

    #[test]
    fn corrections_are_bounded_either_way() {
        let mut thin = Pace::new();
        let mut deep = Pace::new();

        for _ in 0..1000 {
            thin.adjust(0);
            deep.adjust(Audio::CAPACITY);
        }

        assert_eq!(thin.adjust(0), Pace::MIN);
        assert_eq!(deep.adjust(Audio::CAPACITY), Pace::MAX);
    }
}
