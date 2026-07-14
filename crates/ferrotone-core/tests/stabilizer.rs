use ferrotone_core::pitch::stabilizer::StageDStabilizer;

#[test]
fn none_input_clears_buffer() {
    let mut s = StageDStabilizer::new();
    // Prime with a few values
    assert!(s.process(Some(440.0)).is_some());
    assert!(s.process(Some(441.0)).is_some());
    // First few None calls hold the line (hold_silent_frames = 5)
    for _ in 0..5 {
        assert!(s.process(None).is_some(), "should hold during brief silence");
    }
    // After hold_silent_frames exceeded, it clears and returns None
    assert!(s.process(None).is_none());
    // Should start fresh
    let result = s.process(Some(440.0));
    assert!(result.is_some());
}

#[test]
fn first_value_passes_through() {
    let mut s = StageDStabilizer::new();
    let result = s.process(Some(440.0));
    assert_eq!(result, Some(440.0));
}

#[test]
fn small_deviation_passes() {
    let mut s = StageDStabilizer::new();
    s.process(Some(440.0));
    // 2 Hz deviation is not an octave jump
    let result = s.process(Some(442.0));
    assert!(result.is_some());
    let hz = result.unwrap();
    assert!(hz > 440.0);
    assert!(hz < 445.0);
}

#[test]
fn single_octave_jump_is_suppressed() {
    let mut s = StageDStabilizer::new();
    s.process(Some(440.0));
    // Octave up (~2x) — should be suppressed (return previous)
    let result = s.process(Some(880.0));
    assert_eq!(result, Some(440.0));
}

#[test]
fn single_octave_down_jump_is_suppressed() {
    let mut s = StageDStabilizer::new();
    s.process(Some(440.0));
    // Octave down (~0.5x) — should be suppressed
    let result = s.process(Some(220.0));
    assert_eq!(result, Some(440.0));
}

#[test]
fn repeated_consecutive_octave_jumps_break_through() {
    let mut s = StageDStabilizer::new_with(3, 1.0, 3);
    s.process(Some(440.0));
    // Three suppressed octave-up attempts
    for _ in 0..3 {
        assert_eq!(s.process(Some(880.0)), Some(440.0));
    }
    // After the guard breaks through the output converges to 880
    // within a couple of frames (since suppressed values no longer
    // pollute the median buffer).
    assert_eq!(s.process(Some(880.0)), Some(880.0));
}

#[test]
fn non_octave_leap_between_consecutive_jumps_resets_counter() {
    let mut s = StageDStabilizer::new_with(3, 1.0, 3);
    s.process(Some(440.0));
    assert_eq!(s.process(Some(880.0)), Some(440.0)); // suppressed (1)
    // Push enough values for median to converge on 441
    s.process(Some(441.0));
    let result = s.process(Some(441.0));
    assert!(result.is_some());
    assert!((result.unwrap() - 441.0).abs() < 1.0);
    // Now a new octave jump should be treated as first attempt
    let suppressed = s.process(Some(880.0)).unwrap();
    assert!((suppressed - 441.0).abs() < 1.0, "expected ~441, got {suppressed}"); // suppressed again
}

#[test]
fn median_filter_smooths_spike() {
    let mut s = StageDStabilizer::new();
    s.process(Some(440.0));
    s.process(Some(441.0));
    s.process(Some(442.0));
    // A spike should be pulled toward the median
    let result = s.process(Some(500.0));
    assert!(result.is_some());
    let hz = result.unwrap();
    // With window=5 and values [440, 441, 442, 500], median is 441.5
    assert!(hz < 490.0);
}

#[test]
fn smoothing_converges_to_input() {
    let mut s = StageDStabilizer::new_with(3, 0.9, 3);
    // After enough steady input, output should approach input
    for _ in 0..20 {
        s.process(Some(440.0));
    }
    let result = s.process(Some(440.0));
    assert!(result.is_some());
    let hz = result.unwrap();
    assert!((hz - 440.0).abs() < 1.0);
}
