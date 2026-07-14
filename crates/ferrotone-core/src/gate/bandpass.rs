use std::f32::consts::PI;

pub struct BandpassFilter {
    enabled: bool,
    low_hz: f32,
    high_hz: f32,
    sample_rate: u32,
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
    needs_recalc: bool,
}

impl BandpassFilter {
    pub fn new(low_hz: f32, high_hz: f32, sample_rate: u32) -> Self {
        let mut f = Self {
            enabled: true,
            low_hz,
            high_hz,
            sample_rate,
            b0: 0.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
            needs_recalc: true,
        };
        f.recalc_coeffs();
        f
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn set_low_hz(&mut self, low_hz: f32) {
        if (self.low_hz - low_hz).abs() > f32::EPSILON {
            self.low_hz = low_hz;
            self.needs_recalc = true;
        }
    }

    pub fn set_high_hz(&mut self, high_hz: f32) {
        if (self.high_hz - high_hz).abs() > f32::EPSILON {
            self.high_hz = high_hz;
            self.needs_recalc = true;
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.needs_recalc = true;
        }
    }

    fn recalc_coeffs(&mut self) {
        if !self.needs_recalc {
            return;
        }
        self.needs_recalc = false;

        let fs = self.sample_rate as f32;
        let fc = ((self.low_hz + self.high_hz) / 2.0).max(1.0).min(fs / 2.0 - 1.0);
        let bw = (self.high_hz - self.low_hz).max(10.0);
        let q = fc / bw;
        let w0 = 2.0 * PI * fc / fs;
        let alpha = w0.sin() / (2.0 * q);

        let cos_w0 = w0.cos();
        let a0 = 1.0 + alpha;

        self.b0 = alpha / a0;
        self.b1 = 0.0;
        self.b2 = -alpha / a0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    pub fn process(&mut self, samples: &mut [f32]) -> bool {
        if !self.enabled {
            return true;
        }
        if self.needs_recalc {
            self.recalc_coeffs();
        }
        for sample in samples.iter_mut() {
            let x = *sample;
            let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
                - self.a1 * self.y1 - self.a2 * self.y2;
            self.x2 = self.x1;
            self.x1 = x;
            self.y2 = self.y1;
            self.y1 = y;
            *sample = y;
        }
        true
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine_wave(freq: f32, sample_rate: u32, duration_samples: usize) -> Vec<f32> {
        (0..duration_samples)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin())
            .collect()
    }

    fn compute_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    #[test]
    fn disabled_passes_through() {
        let mut filter = BandpassFilter::new(80.0, 1600.0, 48000).with_enabled(false);
        let mut input = sine_wave(440.0, 48000, 1024);
        let original = input.clone();
        filter.process(&mut input);
        for (a, b) in input.iter().zip(original.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn attenuates_out_of_band() {
        let mut filter = BandpassFilter::new(200.0, 400.0, 48000);
        let mut low = sine_wave(50.0, 48000, 4096);
        let mut high = sine_wave(2000.0, 48000, 4096);
        let mut mid = sine_wave(300.0, 48000, 4096);

        filter.process(&mut low);
        filter.reset();
        filter.process(&mut high);
        filter.reset();
        filter.process(&mut mid);

        let low_rms = compute_rms(&low);
        let high_rms = compute_rms(&high);
        let mid_rms = compute_rms(&mid);

        assert!(mid_rms > low_rms * 2.0, "mid should pass better than low (mid={mid_rms}, low={low_rms})");
        assert!(mid_rms > high_rms * 2.0, "mid should pass better than high (mid={mid_rms}, high={high_rms})");
    }

    #[test]
    fn in_band_passes() {
        let mut filter = BandpassFilter::new(80.0, 1600.0, 48000);
        let mut input = sine_wave(440.0, 48000, 4096);
        filter.process(&mut input);
        let rms = compute_rms(&input);
        assert!(rms > 0.1, "440Hz within 80-1000 bandpass should pass (rms={rms})");
    }

    #[test]
    fn recalc_on_param_change() {
        let mut filter = BandpassFilter::new(80.0, 1600.0, 48000);
        let mut input = sine_wave(440.0, 48000, 1024);
        filter.process(&mut input);
        let rms_before = compute_rms(&input);

        filter.set_low_hz(500.0);
        let mut input2 = sine_wave(440.0, 48000, 1024);
        filter.reset();
        filter.process(&mut input2);
        let rms_after = compute_rms(&input2);

        assert!(rms_after < rms_before, "raising low cut to 500 should attenuate 440Hz more");
    }

    #[test]
    fn empty_buffer_no_panic() {
        let mut filter = BandpassFilter::new(80.0, 1600.0, 48000);
        let mut empty: Vec<f32> = vec![];
        filter.process(&mut empty);
    }
}
