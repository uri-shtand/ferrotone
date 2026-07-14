use ferrotone_core::pitch::swipe::SwipeDetector;
use ferrotone_core::pitch::PitchDetector;

fn generate_sine(frequency_hz: f32, sample_rate: u32, num_samples: usize) -> Vec<f32> {
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * std::f32::consts::PI * frequency_hz * t).sin()
        })
        .collect()
}

#[test]
fn sine_a4_440hz() {
    let mut detector = SwipeDetector::new(48000, 1024, 0.3).unwrap();
    // Feed plenty of audio for the tracker to stabilize
    let samples = generate_sine(440.0, 48000, 96000);
    let frames = detector.process(&samples);
    assert!(!frames.is_empty(), "should produce pitch frames");

    // Check last half of frames (after warmup)
    let steady_frames = &frames[frames.len() / 2..];
    let avg_hz: f32 = steady_frames.iter().map(|f| f.frequency_hz).sum::<f32>()
        / steady_frames.len() as f32;
    assert!(
        (avg_hz - 440.0).abs() < 15.0,
        "A4 sine should be ~440 Hz, got avg={} from {} steady frames (total {})",
        avg_hz,
        steady_frames.len(),
        frames.len()
    );
}

#[test]
fn sine_c4_261hz() {
    let mut detector = SwipeDetector::new(48000, 1024, 0.3).unwrap();
    let samples = generate_sine(261.63, 48000, 96000);
    let frames = detector.process(&samples);
    assert!(!frames.is_empty(), "should produce pitch frames");

    let steady_frames = &frames[frames.len() / 2..];
    let avg_hz: f32 = steady_frames.iter().map(|f| f.frequency_hz).sum::<f32>()
        / steady_frames.len() as f32;
    assert!(
        (avg_hz - 261.63).abs() < 15.0,
        "C4 sine should be ~261.63 Hz, got avg={} from {} steady frames (total {})",
        avg_hz,
        steady_frames.len(),
        frames.len()
    );
}

#[test]
fn sine_silence_returns_low_confidence() {
    let mut detector = SwipeDetector::new(48000, 1024, 0.3).unwrap();
    let samples = vec![0.0; 48000];
    let frames = detector.process(&samples);
    if !frames.is_empty() {
        for frame in &frames {
            assert!(
                frame.clarity < 0.5,
                "silence should have low clarity, got {}",
                frame.clarity
            );
        }
    }
}

#[test]
fn swipe_detector_reset() {
    let mut detector = SwipeDetector::new(48000, 1024, 0.3).unwrap();
    let samples = generate_sine(440.0, 48000, 48000);
    let frames_before = detector.process(&samples);
    detector.reset();
    let frames_after = detector.process(&samples);
    assert!(!frames_before.is_empty());
    assert!(!frames_after.is_empty());
}

#[test]
fn swipe_detector_reasonable_values() {
    let mut detector = SwipeDetector::new(48000, 1024, 0.3).unwrap();
    let samples = generate_sine(440.0, 48000, 96000);
    let frames = detector.process(&samples);
    assert!(!frames.is_empty());

    // All frequencies should be reasonable
    for frame in &frames {
        assert!(frame.frequency_hz > 0.0, "frequency should be positive");
        assert!(frame.frequency_hz < 2000.0, "frequency should be reasonable");
        assert!(frame.clarity >= 0.0 && frame.clarity <= 1.0, "clarity in [0,1]");
    }
}
