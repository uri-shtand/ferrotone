use ferrotone_core::audio::{CaptureConfig, CaptureEngine};
use ferrotone_core::pitch::dummy::DummyDetector;

#[test]
fn confidence_gates_low_clarity() {
    let mut engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.1, false)),
        CaptureConfig {
            noise_cancellation_enabled: true,
            confidence_gate_enabled: true,
            confidence_threshold: 0.3,
            ..CaptureConfig::default()
        },
    );
    let samples: Vec<f32> = (0..1024)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
        .collect();
    let frames = engine.feed_audio(&samples);
    assert!(
        frames.is_empty(),
        "low clarity frames should be gated by confidence gate"
    );
}

#[test]
fn confidence_passes_high_clarity() {
    let mut engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.9, true)),
        CaptureConfig {
            noise_cancellation_enabled: true,
            confidence_gate_enabled: true,
            confidence_threshold: 0.3,
            ..CaptureConfig::default()
        },
    );
    let samples: Vec<f32> = (0..1024)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
        .collect();
    let frames = engine.feed_audio(&samples);
    assert!(!frames.is_empty(), "high clarity should pass confidence gate");
    assert!((frames[0].frequency_hz - 440.0).abs() < 0.01);
}

#[test]
fn confidence_gate_disabled_passes_low_clarity() {
    let config = CaptureConfig {
        noise_cancellation_enabled: true,
        confidence_gate_enabled: false,
        ..CaptureConfig::default()
    };
    let mut engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.1, false)),
        config,
    );
    let samples: Vec<f32> = (0..1024)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
        .collect();
    let frames = engine.feed_audio(&samples);
    assert_eq!(
        frames.len(),
        1,
        "disabled confidence gate should pass low clarity"
    );
}