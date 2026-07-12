pub mod dummy;
pub mod swipe;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchFrame {
    pub frequency_hz: f32,
    pub clarity: f32,
    pub voiced: bool,
    pub timestamp_ms: u64,
}

pub trait PitchDetector: Send {
    fn process(&mut self, samples: &[f32]) -> Vec<PitchFrame>;
    fn reset(&mut self);
}
