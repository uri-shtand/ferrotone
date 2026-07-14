use nnnoiseless::DenoiseState;

const SCALE: f32 = 32768.0;

pub struct NoiseSuppressor {
    enabled: bool,
    state: Box<DenoiseState<'static>>,
    frame_size: usize,
    leftover: Vec<f32>,
    output_buf: Vec<f32>,
}

impl Default for NoiseSuppressor {
    fn default() -> Self {
        Self::new()
    }
}

impl NoiseSuppressor {
    pub fn new() -> Self {
        let frame_size = DenoiseState::FRAME_SIZE;
        Self {
            enabled: true,
            state: DenoiseState::new(),
            frame_size,
            leftover: Vec::with_capacity(frame_size),
            output_buf: vec![0.0; frame_size],
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn process(&mut self, samples: &mut Vec<f32>) {
        if !self.enabled || samples.is_empty() {
            return;
        }

        if !self.leftover.is_empty() {
            let mut combined = Vec::with_capacity(self.leftover.len() + samples.len());
            combined.extend_from_slice(&self.leftover);
            combined.extend_from_slice(samples);
            self.leftover.clear();
            *samples = combined;
        }

        let num_frames = samples.len() / self.frame_size;
        let processed = num_frames * self.frame_size;

        for frame_idx in 0..num_frames {
            let start = frame_idx * self.frame_size;
            let end = start + self.frame_size;
            let frame = &mut samples[start..end];

            for s in frame.iter_mut() {
                *s *= SCALE;
            }

            self.state
                .process_frame(&mut self.output_buf, frame);

            frame.copy_from_slice(&self.output_buf);

            for s in frame.iter_mut() {
                *s /= SCALE;
            }
        }

        if processed < samples.len() {
            self.leftover
                .extend_from_slice(&samples[processed..]);
        }

        samples.truncate(processed);
    }

    pub fn reset(&mut self) {
        self.leftover.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_is_noop() {
        let mut suppressor = NoiseSuppressor::new().with_enabled(false);
        let mut samples = vec![0.5, -0.5, 0.25, -0.25];
        let original = samples.clone();
        suppressor.process(&mut samples);
        assert_eq!(samples, original);
    }

    #[test]
    fn empty_buffer_no_panic() {
        let mut suppressor = NoiseSuppressor::new();
        let mut empty: Vec<f32> = vec![];
        suppressor.process(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn reset_clears_leftover() {
        let mut suppressor = NoiseSuppressor::new().with_enabled(false);
        let mut samples = vec![0.5; 100];
        suppressor.process(&mut samples);
        // with disabled, leftover stays empty because we return early
        assert!(suppressor.leftover.is_empty());
    }

    #[test]
    fn full_frame_processed() {
        let mut suppressor = NoiseSuppressor::new();
        let mut samples = vec![0.1; 480];
        let len_before = samples.len();
        suppressor.process(&mut samples);
        assert_eq!(samples.len(), len_before);
        assert!(suppressor.leftover.is_empty());
    }

    #[test]
    fn partial_frame_buffered() {
        let mut suppressor = NoiseSuppressor::new();
        let mut samples = vec![0.1; 500];
        suppressor.process(&mut samples);
        // 500 = 1 frame (480) + 20 leftover
        assert_eq!(samples.len(), 480);
        assert_eq!(suppressor.leftover.len(), 20);
    }

    #[test]
    fn leftover_prepended() {
        let mut suppressor = NoiseSuppressor::new();
        // First call: 500 samples → 480 processed, 20 leftover
        let mut s1 = vec![0.1; 500];
        suppressor.process(&mut s1);
        assert_eq!(s1.len(), 480);
        assert_eq!(suppressor.leftover.len(), 20);

        // Second call: 460 + 20 leftover = 480 → 1 full frame
        let mut s2 = vec![0.1; 460];
        suppressor.process(&mut s2);
        assert_eq!(s2.len(), 480);
        assert!(suppressor.leftover.is_empty());
    }
}
