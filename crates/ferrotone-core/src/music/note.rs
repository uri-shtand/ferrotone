pub const A4_HZ: f32 = 440.0;
pub const A4_MIDI: f32 = 69.0;
pub const SEMITONES_PER_OCTAVE: f32 = 12.0;

const NOTE_LABELS: &[&str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Convert frequency in Hz to MIDI note number (may be fractional).
pub fn hz_to_midi(frequency_hz: f32) -> f32 {
    if frequency_hz <= 0.0 {
        return 0.0;
    }
    A4_MIDI + SEMITONES_PER_OCTAVE * (frequency_hz / A4_HZ).log2()
}

/// Convert MIDI note number to nearest note name ("C4", "A#4", etc.).
pub fn midi_to_note_name(midi: f32) -> String {
    let idx = midi.round() as i32;
    if !(0..=127).contains(&idx) {
        return "?".to_string();
    }
    let label = NOTE_LABELS[idx as usize % 12];
    let octave = (idx / 12) - 1;
    format!("{}{}", label, octave)
}

/// Compute cents deviation between actual and target frequency.
/// Positive = sharp, negative = flat.
pub fn cents_off(actual_hz: f32, target_hz: f32) -> f32 {
    if actual_hz <= 0.0 || target_hz <= 0.0 {
        return 0.0;
    }
    1200.0 * (actual_hz / target_hz).log2()
}

/// Return the nearest equal-tempered frequency for a given frequency.
pub fn nearest_equal_tempered_freq(frequency_hz: f32) -> f32 {
    if frequency_hz <= 0.0 {
        return 0.0;
    }
    let midi = hz_to_midi(frequency_hz);
    let rounded_midi = midi.round();
    A4_HZ * 2.0_f32.powf((rounded_midi - A4_MIDI) / SEMITONES_PER_OCTAVE)
}
