# pitch-core 0.1.0 API Summary

## SwipeEstimator
- `SwipeEstimator::new() -> Result<Self>` — no SwipeMode param needed
- `SwipeEstimator::with_max_window(max_window: usize) -> Result<Self>`
- Implements `PitchEstimator` trait

## PitchTracker
- `PitchTracker::new<E: PitchEstimator + 'static>(estimator: E, input_sample_rate: u32, resample_chunk: usize) -> Result<Self>`
- `PitchTracker::from_boxed(estimator: Box<dyn PitchEstimator>, input_sample_rate: u32, resample_chunk: usize) -> Result<Self>`
- `process(&mut self, audio: &[f32]) -> Result<Vec<PitchFrame>>`
- `reset(&mut self)`
- `input_sample_rate(&self) -> u32`
- `target_sample_rate(&self) -> u32`
- `algorithm(&self) -> &str`
- Not Sync, but Send

## PitchFrame (pitch_core::estimator)
- Fields: `frame_index: u64`, `time_s: f32`, `pitch_hz: f32`, `confidence: f32`, `is_preliminary: bool`
- Implements Clone, Copy, Debug

## Re-exports
- `pitch_core::SwipeEstimator`, `pitch_core::PitchTracker`, `pitch_core::PitchFrame`, `pitch_core::PitchEstimator`
- `pitch_core::EstimatorError`, `pitch_core::calibrate_confidence`
