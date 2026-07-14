use pitch_core::{PitchTracker, SwipeEstimator};

use super::{PitchDetector, PitchFrame};
use crate::error::DetectionError;

pub struct SwipeDetector {
    tracker: PitchTracker,
    frame_count: u64,
    sample_rate: u32,
    buffer_size: usize,
    confidence_threshold: f32,
}

impl SwipeDetector {
    pub fn new(
        sample_rate: u32,
        buffer_size: usize,
        confidence_threshold: f32,
    ) -> Result<Self, DetectionError> {
        let estimator = SwipeEstimator::new()?;
        let tracker = PitchTracker::new(estimator, sample_rate, buffer_size)?;
        Ok(Self {
            tracker,
            frame_count: 0,
            sample_rate,
            buffer_size,
            confidence_threshold,
        })
    }
}

impl PitchDetector for SwipeDetector {
    fn process(&mut self, samples: &[f32]) -> Vec<PitchFrame> {
        let frames = match self.tracker.process(samples) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let result: Vec<PitchFrame> = frames
            .into_iter()
            .filter(|pf| !pf.is_preliminary)
            .map(|pf| {
                self.frame_count += 1;
                PitchFrame {
                    frequency_hz: pf.pitch_hz,
                    clarity: pf.confidence,
                    voiced: pf.confidence > self.confidence_threshold,
                    timestamp_ms: (self.frame_count * self.buffer_size as u64 * 1000
                        / self.sample_rate as u64)
                        .max(1),
                }
            })
            .collect();

        result
    }

    fn reset(&mut self) {
        self.tracker.reset();
        self.frame_count = 0;
    }
}
