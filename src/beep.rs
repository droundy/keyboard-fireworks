use std::time::Duration;
use rodio::Source;

/// An infinite source that produces a sine.
///
/// Always has a rate of 48kHz and one channel.
#[derive(Clone, Debug)]
pub struct Chirp {
    time: Duration,
    inv_time: f32,
    num_sample: usize,
    up: bool,
}

const RATE: u32 = 48000;
const MAX_FREQ: f32 =  0.125 * RATE as f32;

impl Chirp {
    /// Create an up chirp
    #[inline]
    pub fn up(time: Duration) -> Chirp {
        Chirp {
            time: time,
            inv_time: 1e9/time.as_nanos() as f32,
            num_sample: 0,
            up: true,
        }
    }
    /// Create a down chirp
    #[inline]
    pub fn down(time: Duration) -> Chirp {
        Chirp {
            time: time,
            inv_time: 1e9/time.as_nanos() as f32,
            num_sample: 0,
            up: false,
        }
    }
}

impl Iterator for Chirp {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);

        let mut t = self.num_sample as f32 * (1./RATE as f32) * self.inv_time;
        if t > 1. {
            return None;
        }
        if !self.up {
            t = 1.-t;
        }
        let value = 2.0 * 3.14159265 * MAX_FREQ * t * t;
        Some(value.sin())
    }
}

impl Source for Chirp {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        RATE
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        Some(self.time)
    }
}
