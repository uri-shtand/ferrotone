use crate::pitch::PitchFrame;

pub struct ConfidenceGate {
    enabled: bool,
    threshold: f32,
    hysteresis: f32,
    was_above_threshold: bool,
}

impl ConfidenceGate {
    pub fn new(threshold: f32) -> Self {
        Self {
            enabled: true,
            threshold,
            hysteresis: 0.05,
            was_above_threshold: false,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn process(&mut self, frames: Vec<PitchFrame>) -> Vec<PitchFrame> {
        if !self.enabled {
            return frames;
        }
        let effective_threshold = if self.was_above_threshold {
            self.threshold - self.hysteresis
        } else {
            self.threshold
        };
        let mut any_passed = false;
        let result: Vec<PitchFrame> = frames
            .into_iter()
            .filter(|f| {
                let pass = f.clarity >= effective_threshold;
                if pass {
                    any_passed = true;
                }
                pass
            })
            .collect();
        self.was_above_threshold = any_passed;
        result
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
        let mut gate = ConfidenceGate::new(0.3);
        let frames = vec![make_frame(0.1), make_frame(0.2)];
        let result = gate.process(frames);
        assert!(result.is_empty());
    }

    #[test]
    fn high_clarity_passes() {
        let mut gate = ConfidenceGate::new(0.3);
        let frames = vec![make_frame(0.5), make_frame(0.9)];
        let result = gate.process(frames);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn disabled_passes_all() {
        let mut gate = ConfidenceGate::new(0.3).with_enabled(false);
        let frames = vec![make_frame(0.1), make_frame(0.9)];
        let result = gate.process(frames);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn mixed_clarity() {
        let mut gate = ConfidenceGate::new(0.3);
        let frames = vec![make_frame(0.1), make_frame(0.5), make_frame(0.2), make_frame(0.9)];
        let result = gate.process(frames);
        assert_eq!(result.len(), 2);
        for frame in &result {
            assert!(frame.clarity >= 0.3);
        }
    }

    #[test]
    fn hysteresis_holds_near_threshold() {
        let mut gate = ConfidenceGate::new(0.3);
        // First, pass a high-clarity frame to set was_above_threshold
        let result = gate.process(vec![make_frame(0.9)]);
        assert_eq!(result.len(), 1);
        // Now a frame just below threshold should still pass
        // (hysteresis of 0.05 means effective threshold is 0.25)
        let result = gate.process(vec![make_frame(0.29)]);
        assert_eq!(result.len(), 1, "hysteresis should let 0.29 pass after 0.9");
        // A frame well below threshold should fail
        let result = gate.process(vec![make_frame(0.20)]);
        assert!(result.is_empty(), "0.20 should fail even with hysteresis");
        // After failing, was_above_threshold is false, so 0.29 should fail
        let result = gate.process(vec![make_frame(0.29)]);
        assert!(result.is_empty(), "0.29 should fail without hysteresis");
    }
}
