use crate::pitch::PitchFrame;

pub struct ConfidenceGate {
    enabled: bool,
    threshold: f32,
}

impl ConfidenceGate {
    pub fn new(threshold: f32) -> Self {
        Self {
            enabled: true,
            threshold,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn process(&self, frames: Vec<PitchFrame>) -> Vec<PitchFrame> {
        if !self.enabled {
            return frames;
        }
        frames
            .into_iter()
            .filter(|f| f.clarity >= self.threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(clarity: f32) -> PitchFrame {
        PitchFrame {
            frequency_hz: 440.0,
            clarity,
            voiced: clarity > 0.3,
            timestamp_ms: 0,
        }
    }

    #[test]
    fn low_clarity_filtered() {
        let gate = ConfidenceGate::new(0.3);
        let frames = vec![make_frame(0.1), make_frame(0.2)];
        let result = gate.process(frames);
        assert!(result.is_empty());
    }

    #[test]
    fn high_clarity_passes() {
        let gate = ConfidenceGate::new(0.3);
        let frames = vec![make_frame(0.5), make_frame(0.9)];
        let result = gate.process(frames);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn disabled_passes_all() {
        let gate = ConfidenceGate::new(0.3).with_enabled(false);
        let frames = vec![make_frame(0.1), make_frame(0.9)];
        let result = gate.process(frames);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn mixed_clarity() {
        let gate = ConfidenceGate::new(0.3);
        let frames = vec![make_frame(0.1), make_frame(0.5), make_frame(0.2), make_frame(0.9)];
        let result = gate.process(frames);
        assert_eq!(result.len(), 2);
        for frame in &result {
            assert!(frame.clarity >= 0.3);
        }
    }
}
