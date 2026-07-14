use ferrotone_core::audio::{CaptureConfig, CaptureEngine};
use ferrotone_core::pitch::dummy::DummyDetector;

#[test]
fn rms_gates_silence_in_feed_audio() {
    let mut engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.9, true)),
        CaptureConfig {
            noise_cancellation_enabled: true,
            rms_gate_enabled: true,
            rms_threshold: 0.01,
            ..CaptureConfig::default()
        },
    );
    let frames = engine.feed_audio(&[0.0; 1024]);
    assert!(frames.is_empty(), "silence should be gated by RMS gate");
}

#[test]
fn rms_passes_loud_signal() {
    let mut engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.9, true)),
        CaptureConfig {
            noise_cancellation_enabled: true,
            rms_gate_enabled: true,
            rms_threshold: 0.01,
            ..CaptureConfig::default()
        },
    );
    let samples: Vec<f32> = (0..1024)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
        .collect();
    let frames = engine.feed_audio(&samples);
    assert!(!frames.is_empty(), "loud signal should pass RMS gate");
}

#[test]
fn rms_gate_works_without_noise_cancellation_master() {
    let mut engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.9, true)),
        CaptureConfig {
            noise_cancellation_enabled: false,
            rms_gate_enabled: true,
            rms_threshold: 0.01,
            ..CaptureConfig::default()
        },
    );
    let frames = engine.feed_audio(&[0.0; 1024]);
    assert!(
        frames.is_empty(),
        "RMS gate should gate silence even when noise_cancellation is off"
    );
}

#[test]
fn rms_gate_disabled_passes_silence() {
    let config = CaptureConfig {
        noise_cancellation_enabled: true,
        rms_gate_enabled: false,
        ..CaptureConfig::default()
    };
    let mut engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.9, true)),
        config,
    );
    let frames = engine.feed_audio(&[0.0; 1024]);
    assert_eq!(frames.len(), 1, "disabled RMS gate should pass silence");
}