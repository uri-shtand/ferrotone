use ferrotone_core::music::*;

#[test]
fn hz_to_midi_roundtrip() {
    let midi = hz_to_midi(440.0);
    assert!(
        (midi - 69.0).abs() < 0.01,
        "A4=440 Hz should be MIDI 69, got {}",
        midi
    );

    let midi = hz_to_midi(261.63);
    assert!(
        (midi - 60.0).abs() < 0.1,
        "C4=261.63 Hz should be MIDI ~60, got {}",
        midi
    );

    let midi = hz_to_midi(0.0);
    assert!(
        (midi - 0.0).abs() < 0.01,
        "0 Hz should return 0, got {}",
        midi
    );
}

#[test]
fn midi_to_note_names_golden() {
    assert_eq!(midi_to_note_name(69.0), "A4");
    assert_eq!(midi_to_note_name(60.0), "C4");
    assert_eq!(midi_to_note_name(0.0), "C-1");
    assert_eq!(midi_to_note_name(127.0), "G9");
    assert_eq!(midi_to_note_name(70.0), "A#4");
    assert_eq!(midi_to_note_name(61.0), "C#4");
    assert_eq!(midi_to_note_name(100.0), "E7");
    assert_eq!(midi_to_note_name(200.0), "?");
}

#[test]
fn cents_off_in_tune() {
    let cents = cents_off(440.0, 440.0);
    assert!(
        (cents - 0.0).abs() < 0.001,
        "440 vs 440 should be 0 cents, got {}",
        cents
    );
}

#[test]
fn cents_off_semitone() {
    let cents = cents_off(440.0, 466.16);
    assert!(
        (cents + 100.0).abs() < 2.0,
        "440 vs 466.16 should be ≈ -100 cents, got {}",
        cents
    );

    let cents = cents_off(466.16, 440.0);
    assert!(
        (cents - 100.0).abs() < 2.0,
        "466.16 vs 440 should be ≈ +100 cents, got {}",
        cents
    );
}

#[test]
fn cents_off_edge_cases() {
    assert!(
        (cents_off(0.0, 440.0) - 0.0).abs() < 0.001,
        "0 Hz should return 0"
    );
    assert!(
        (cents_off(440.0, 0.0) - 0.0).abs() < 0.001,
        "0 target should return 0"
    );
    assert!(
        (cents_off(-1.0, 440.0) - 0.0).abs() < 0.001,
        "negative should return 0"
    );
}

#[test]
fn nearest_equal_tempered() {
    let near = nearest_equal_tempered_freq(440.0);
    assert!(
        (near - 440.0).abs() < 0.01,
        "440 should map to 440, got {}",
        near
    );

    let near = nearest_equal_tempered_freq(445.0);
    assert!(
        (near - 440.0).abs() < 0.01,
        "445 should map to 440, got {}",
        near
    );

    let near = nearest_equal_tempered_freq(455.0);
    assert!(
        (near - 466.16).abs() < 0.1,
        "455 should map to ~466.16 (A#4), got {}",
        near
    );

    let near = nearest_equal_tempered_freq(0.0);
    assert!(
        (near - 0.0).abs() < 0.01,
        "0 Hz should map to 0, got {}",
        near
    );
}
