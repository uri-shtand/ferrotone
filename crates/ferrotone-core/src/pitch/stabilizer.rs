use std::collections::VecDeque;

pub struct StageDStabilizer {
    pub median_window_size: usize,
    pub smoothing_alpha: f32,
    pub stable_alpha: f32,
    pub max_consecutive_octaves: usize,
    pub hold_silent_frames: usize,
    prev_valid_pitch: Option<f32>,
    prev_smoothed_pitch: Option<f32>,
    history_buffer: VecDeque<f32>,
    consecutive_octave_count: usize,
    silent_frames: usize,
}

impl StageDStabilizer {
    pub fn new() -> Self {
        Self {
            median_window_size: 7,
            smoothing_alpha: 0.55,
            stable_alpha: 0.12,
            max_consecutive_octaves: 4,
            hold_silent_frames: 5,
            prev_valid_pitch: None,
            prev_smoothed_pitch: None,
            history_buffer: VecDeque::with_capacity(8),
            consecutive_octave_count: 0,
            silent_frames: 0,
        }
    }

    pub fn new_with(
        median_window_size: usize,
        smoothing_alpha: f32,
        max_consecutive_octaves: usize,
    ) -> Self {
        Self {
            median_window_size,
            smoothing_alpha,
            stable_alpha: smoothing_alpha * 0.2,
            max_consecutive_octaves,
            hold_silent_frames: 5,
            prev_valid_pitch: None,
            prev_smoothed_pitch: None,
            history_buffer: VecDeque::with_capacity(median_window_size + 1),
            consecutive_octave_count: 0,
            silent_frames: 0,
        }
    }

    pub fn process(&mut self, input: Option<f32>) -> Option<f32> {
        let raw_hz = match input {
            Some(hz) => {
                self.silent_frames = 0;
                hz
            }
            None => {
                self.silent_frames += 1;
                if self.silent_frames > self.hold_silent_frames {
                    self.history_buffer.clear();
                    self.consecutive_octave_count = 0;
                    return None;
                }
                return self.prev_smoothed_pitch;
            }
        };

        let guarded = self.octave_jump_guard(raw_hz);
        let validated_hz = match guarded {
            Some(hz) => hz,
            None => return self.prev_smoothed_pitch,
        };

        let median_hz = self.median_filter(validated_hz);
        let alpha = self.compute_alpha(median_hz);
        let smoothed_hz = self.one_pole_smooth(median_hz, alpha);

        self.prev_smoothed_pitch = Some(smoothed_hz);
        self.prev_valid_pitch = Some(smoothed_hz);

        Some(smoothed_hz)
    }

    fn compute_alpha(&mut self, _median_hz: f32) -> f32 {
        if self.history_buffer.len() < 3 {
            return self.smoothing_alpha;
        }

        let min = self
            .history_buffer
            .iter()
            .copied()
            .fold(f32::MAX, f32::min);
        let max = self
            .history_buffer
            .iter()
            .copied()
            .fold(f32::MIN, f32::max);

        let cents_range = 1200.0 * (max / min.max(f32::EPSILON)).log2();

        if cents_range < 3.0 {
            self.stable_alpha
        } else if cents_range < 15.0 {
            let t = (cents_range - 3.0) / 12.0;
            self.stable_alpha + t * (self.smoothing_alpha - self.stable_alpha)
        } else {
            self.smoothing_alpha
        }
    }

    fn octave_jump_guard(&mut self, raw_hz: f32) -> Option<f32> {
        let prev = match self.prev_valid_pitch {
            Some(p) => p,
            None => return Some(raw_hz),
        };

        if prev <= 0.0 {
            return Some(raw_hz);
        }

        let ratio = raw_hz / prev;

        let is_octave_up = (1.85..=2.15).contains(&ratio);
        let is_octave_down = (0.45..=0.55).contains(&ratio);

        if is_octave_up || is_octave_down {
            self.consecutive_octave_count += 1;
            if self.consecutive_octave_count <= self.max_consecutive_octaves {
                return None;
            }
            self.consecutive_octave_count = 0;
            Some(raw_hz)
        } else {
            self.consecutive_octave_count = 0;
            Some(raw_hz)
        }
    }

    fn median_filter(&mut self, validated_hz: f32) -> f32 {
        self.history_buffer.push_back(validated_hz);

        while self.history_buffer.len() > self.median_window_size {
            self.history_buffer.pop_front();
        }

        if self.history_buffer.len() < 3 {
            return validated_hz;
        }

        let mut sorted: Vec<f32> = self.history_buffer.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        sorted[sorted.len() / 2]
    }

    fn one_pole_smooth(&mut self, median_hz: f32, alpha: f32) -> f32 {
        match self.prev_smoothed_pitch {
            Some(prev) => alpha * median_hz + (1.0 - alpha) * prev,
            None => median_hz,
        }
    }
}

impl Default for StageDStabilizer {
    fn default() -> Self {
        Self::new()
    }
}
