use super::{PitchDetector, PitchFrame};

pub struct DummyDetector {
    frequency_hz: f32,
    clarity: f32,
    voiced: bool,
    frame_count: u64,
}

impl DummyDetector {
    pub fn new(frequency_hz: f32, clarity: f32, voiced: bool) -> Self {
        Self {
            frequency_hz,
            clarity,
            voiced,
            frame_count: 0,
        }
    }
}

impl PitchDetector for DummyDetector {
    fn process(&mut self, _samples: &[f32]) -> Vec<PitchFrame> {
        self.frame_count += 1;
        vec![PitchFrame {
            frequency_hz: self.frequency_hz,
            clarity: self.clarity,
            voiced: self.voiced,
            timestamp_ms: self.frame_count * 32,
        }]
    }

    fn reset(&mut self) {
        self.frame_count = 0;
    }
}
