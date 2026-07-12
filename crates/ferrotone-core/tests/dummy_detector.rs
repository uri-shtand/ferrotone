use ferrotone_core::pitch::dummy::DummyDetector;
use ferrotone_core::pitch::PitchDetector;

#[test]
fn dummy_returns_known_freq() {
    let mut detector = DummyDetector::new(440.0, 0.9, true);
    let frames = detector.process(&[0.0; 1024]);
    assert_eq!(frames.len(), 1);
    assert!((frames[0].frequency_hz - 440.0).abs() < 0.01);
    assert!((frames[0].clarity - 0.9).abs() < 0.01);
    assert!(frames[0].voiced);
}

#[test]
fn dummy_returns_unvoiced() {
    let mut detector = DummyDetector::new(220.0, 0.1, false);
    let frames = detector.process(&[0.0; 512]);
    assert_eq!(frames.len(), 1);
    assert!((frames[0].frequency_hz - 220.0).abs() < 0.01);
    assert!(!frames[0].voiced);
}

#[test]
fn dummy_reset_clears_frame_count() {
    let mut detector = DummyDetector::new(440.0, 0.9, true);
    let f1 = detector.process(&[0.0; 1024]);
    let f2 = detector.process(&[0.0; 1024]);
    assert!(f2[0].timestamp_ms > f1[0].timestamp_ms);
    detector.reset();
    let f3 = detector.process(&[0.0; 1024]);
    assert!((f3[0].timestamp_ms - 32) < 2);
}
