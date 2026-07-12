use ferrotone_core::audio::{CaptureConfig, CaptureEngine};
use ferrotone_core::pitch::dummy::DummyDetector;

#[test]
fn dummy_through_engine() {
    let engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.9, true)),
        CaptureConfig::default(),
    );
    let rx = engine.pitch_receiver().clone();

    // Feed samples manually through the detector pipeline
    let mut eng = engine;
    let frames = eng.feed_audio(&[0.0; 1024]);
    assert_eq!(frames.len(), 1);
    assert!((frames[0].frequency_hz - 440.0).abs() < 0.01);

    // Also check the channel received the frames
    let received: Vec<_> = rx.try_iter().collect();
    assert_eq!(received.len(), 1);
    assert!((received[0].frequency_hz - 440.0).abs() < 0.01);
}

#[test]
fn start_stop_cycle() {
    let mut engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.9, true)),
        CaptureConfig::default(),
    );
    // Stopping an unstarted engine should be safe
    assert!(engine.stop().is_ok());
    assert!(!engine.is_running());
}

#[test]
fn config_defaults() {
    let config = CaptureConfig::default();
    assert_eq!(config.sample_rate, 48000);
    assert_eq!(config.buffer_size, 1024);
    assert!(config.device_name.is_none());
}

#[test]
fn capture_engine_new_not_running() {
    let engine = CaptureEngine::new(
        Box::new(DummyDetector::new(440.0, 0.9, true)),
        CaptureConfig::default(),
    );
    assert!(!engine.is_running());
}

#[test]
fn dummy_through_engine_multiple_batches() {
    let engine = CaptureEngine::new(
        Box::new(DummyDetector::new(220.0, 0.5, false)),
        CaptureConfig::default(),
    );
    let rx = engine.pitch_receiver().clone();

    let mut eng = engine;
    let f1 = eng.feed_audio(&[0.0; 512]);
    let f2 = eng.feed_audio(&[0.0; 512]);

    assert_eq!(f1.len(), 1);
    assert_eq!(f2.len(), 1);
    assert!((f1[0].frequency_hz - 220.0).abs() < 0.01);
    assert!((f2[0].frequency_hz - 220.0).abs() < 0.01);

    let received: Vec<_> = rx.try_iter().collect();
    assert_eq!(received.len(), 2);
}
