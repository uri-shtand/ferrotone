pub struct RmsGate {
    enabled: bool,
    threshold: f32,
}

impl RmsGate {
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

    pub fn process(&self, samples: &[f32]) -> bool {
        if !self.enabled {
            return true;
        }
        compute_rms(samples) >= self.threshold
    }
}

pub fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_below_threshold() {
        let gate = RmsGate::new(0.01);
        assert!(!gate.process(&[0.0; 1024]));
    }

    #[test]
    fn sine_above_threshold() {
        let gate = RmsGate::new(0.01);
        let samples: Vec<f32> = (0..1024)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
            .collect();
        assert!(gate.process(&samples));
    }

    #[test]
    fn disabled_gate_passes_all() {
        let gate = RmsGate::new(0.01).with_enabled(false);
        assert!(gate.process(&[0.0; 1024]));
    }

    #[test]
    fn custom_threshold() {
        let gate = RmsGate::new(0.5);
        let samples: Vec<f32> = (0..1024)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
            .collect();
        assert!(gate.process(&samples));
        let gate = RmsGate::new(0.8);
        assert!(!gate.process(&samples));
    }

    #[test]
    fn compute_rms_silence() {
        assert_eq!(compute_rms(&[0.0; 1024]), 0.0);
    }

    #[test]
    fn compute_rms_sine() {
        let samples: Vec<f32> = (0..1024)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
            .collect();
        let rms = compute_rms(&samples);
        // Sine at amplitude 1.0 has RMS of 1/√2 ≈ 0.707
        assert!((rms - 0.707).abs() < 0.01);
    }

    #[test]
    fn compute_rms_empty() {
        assert_eq!(compute_rms(&[]), 0.0);
    }
}
