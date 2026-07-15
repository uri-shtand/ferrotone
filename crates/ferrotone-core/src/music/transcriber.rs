use serde::{Deserialize, Serialize};

use super::note::{cents_off, hz_to_midi, midi_to_note_name, nearest_equal_tempered_freq};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteEventType {
    Started,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NoteEvent {
    pub event_type: NoteEventType,
    pub note_name: String,
    pub midi: u8,
    pub cents_deviation: f32,
    pub clarity: f32,
    pub duration_ms: u64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone)]
struct NoteSegmentData {
    note_name: String,
    midi: u8,
    cents_sum: f32,
    clarity_sum: f32,
    sample_count: u32,
    start_timestamp_ms: u64,
    last_timestamp_ms: u64,
    confirmed: bool,
    silence_start_ms: Option<u64>,
}

enum SegmenterState {
    Silence,
    InNote(NoteSegmentData),
}

pub struct NoteSegmenter {
    pub min_note_duration_ms: u64,
    pub hold_silence_ms: u64,
    pub note_transition_cents: f32,
    state: SegmenterState,
}

impl NoteSegmenter {
    pub fn new() -> Self {
        Self {
            min_note_duration_ms: 120,
            hold_silence_ms: 80,
            note_transition_cents: 40.0,
            state: SegmenterState::Silence,
        }
    }

    pub fn process(
        &mut self,
        frequency_hz: f32,
        clarity: f32,
        voiced: bool,
        timestamp_ms: u64,
    ) -> Vec<NoteEvent> {
        let midi_float = hz_to_midi(frequency_hz);
        let midi = midi_float.round() as u8;
        let note_name = midi_to_note_name(midi_float);
        let cents = cents_off(frequency_hz, nearest_equal_tempered_freq(frequency_hz));

        let current_state = std::mem::replace(&mut self.state, SegmenterState::Silence);

        match current_state {
            SegmenterState::Silence => {
                if !voiced || clarity <= 0.0 {
                    return Vec::new();
                }
                self.state = SegmenterState::InNote(NoteSegmentData {
                    note_name,
                    midi,
                    cents_sum: cents,
                    clarity_sum: clarity,
                    sample_count: 1,
                    start_timestamp_ms: timestamp_ms,
                    last_timestamp_ms: timestamp_ms,
                    confirmed: false,
                    silence_start_ms: None,
                });
                Vec::new()
            }

            SegmenterState::InNote(mut data) => {
                if !voiced {
                    let silence_start = *data.silence_start_ms.get_or_insert(timestamp_ms);
                    let silence_elapsed = timestamp_ms - silence_start;
                    if silence_elapsed >= self.hold_silence_ms {
                        return self.finalize_impl(&data, timestamp_ms);
                    }
                    data.last_timestamp_ms = timestamp_ms;
                    data.silence_start_ms = Some(silence_start);
                    self.state = SegmenterState::InNote(data);
                    return Vec::new();
                }

                data.silence_start_ms = None;

                let is_new_note = note_name != data.note_name
                    || (cents - (data.cents_sum / data.sample_count.max(1) as f32)).abs()
                        > self.note_transition_cents;

                if is_new_note {
                    let duration = data.last_timestamp_ms - data.start_timestamp_ms;
                    if data.confirmed && duration >= self.min_note_duration_ms {
                        let ended_note_name = data.note_name.clone();
                        let ended_midi = data.midi;
                        let count = data.sample_count.max(1);
                        let ended_event = NoteEvent {
                            event_type: NoteEventType::Ended,
                            note_name: ended_note_name,
                            midi: ended_midi,
                            cents_deviation: data.cents_sum / count as f32,
                            clarity: data.clarity_sum / count as f32,
                            duration_ms: duration,
                            timestamp_ms,
                        };

                        data.note_name = note_name.clone();
                        data.midi = midi;
                        data.cents_sum = cents;
                        data.clarity_sum = clarity;
                        data.sample_count = 1;
                        data.start_timestamp_ms = timestamp_ms;
                        data.last_timestamp_ms = timestamp_ms;
                        data.confirmed = false;
                        data.silence_start_ms = None;

                        self.state = SegmenterState::InNote(data);
                        vec![ended_event]
                    } else {
                        data.note_name = note_name.clone();
                        data.midi = midi;
                        data.cents_sum = cents;
                        data.clarity_sum = clarity;
                        data.sample_count = 1;
                        data.start_timestamp_ms = timestamp_ms;
                        data.last_timestamp_ms = timestamp_ms;
                        data.silence_start_ms = None;

                        self.state = SegmenterState::InNote(data);
                        Vec::new()
                    }
                } else {
                    data.cents_sum += cents;
                    data.clarity_sum += clarity;
                    data.sample_count += 1;
                    data.last_timestamp_ms = timestamp_ms;

                    // Check if this note should now be confirmed
                    let mut events = Vec::new();
                    if !data.confirmed {
                        let duration = data.last_timestamp_ms - data.start_timestamp_ms;
                        if duration >= self.min_note_duration_ms {
                            data.confirmed = true;
                            events.push(NoteEvent {
                                event_type: NoteEventType::Started,
                                note_name: data.note_name.clone(),
                                midi: data.midi,
                                cents_deviation: data.cents_sum / data.sample_count as f32,
                                clarity: data.clarity_sum / data.sample_count as f32,
                                duration_ms: 0,
                                timestamp_ms,
                            });
                        }
                    }

                    self.state = SegmenterState::InNote(data);
                    events
                }
            }
        }
    }

    fn finalize_impl(&mut self, data: &NoteSegmentData, timestamp_ms: u64) -> Vec<NoteEvent> {
        let duration = data.last_timestamp_ms - data.start_timestamp_ms;
        if data.confirmed && duration >= self.min_note_duration_ms {
            let count = data.sample_count.max(1);
            vec![NoteEvent {
                event_type: NoteEventType::Ended,
                note_name: data.note_name.clone(),
                midi: data.midi,
                cents_deviation: data.cents_sum / count as f32,
                clarity: data.clarity_sum / count as f32,
                duration_ms: duration,
                timestamp_ms,
            }]
        } else {
            Vec::new()
        }
    }

    pub fn flush(&mut self, timestamp_ms: u64) -> Vec<NoteEvent> {
        let state = std::mem::replace(&mut self.state, SegmenterState::Silence);
        match state {
            SegmenterState::InNote(data) => self.finalize_impl(&data, timestamp_ms),
            SegmenterState::Silence => Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state = SegmenterState::Silence;
    }
}

impl Default for NoteSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_note_held_long_enough() {
        let mut s = NoteSegmenter::new();
        s.min_note_duration_ms = 100;

        let events = s.process(440.0, 0.9, true, 0);
        assert!(events.is_empty());

        let events = s.process(440.1, 0.9, true, 150);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Started));
        assert_eq!(events[0].note_name, "A4");

        let events = s.process(0.0, 0.0, false, 250);
        assert!(events.is_empty());

        let events = s.process(0.0, 0.0, false, 400);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Ended));
        assert_eq!(events[0].note_name, "A4");
        assert!(events[0].duration_ms >= 100);
    }

    #[test]
    fn note_transition_emits_ended_and_started() {
        let mut s = NoteSegmenter::new();
        s.min_note_duration_ms = 50;

        s.process(440.0, 0.9, true, 0);

        let events = s.process(440.0, 0.9, true, 100);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Started));

        let events = s.process(523.25, 0.9, true, 200);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Ended));
        assert_eq!(events[0].note_name, "A4");

        let events = s.process(523.25, 0.9, true, 300);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Started));
        assert_eq!(events[0].note_name, "C5");
    }

    #[test]
    fn brief_glissando_absorbed_then_final_note_confirms() {
        let mut s = NoteSegmenter::new();
        s.min_note_duration_ms = 100;

        s.process(440.0, 0.9, true, 0);

        // A#4 glissando frame replaces A4 silently
        let events = s.process(466.16, 0.9, true, 60);
        assert!(events.is_empty());

        // C5 is the landing note
        let events = s.process(523.25, 0.9, true, 120);
        assert!(events.is_empty());

        // C5 held long enough → confirmed
        let events = s.process(523.25, 0.9, true, 250);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Started));
        assert_eq!(events[0].note_name, "C5");
    }

    #[test]
    fn short_note_discarded() {
        let mut s = NoteSegmenter::new();
        s.min_note_duration_ms = 200;

        s.process(440.0, 0.9, true, 0);

        let events = s.flush(150);
        assert!(events.is_empty());
    }

    #[test]
    fn silence_held_briefly_resumes_note() {
        let mut s = NoteSegmenter::new();
        s.hold_silence_ms = 80;
        s.min_note_duration_ms = 50;

        s.process(440.0, 0.9, true, 0);

        let events = s.process(440.0, 0.9, true, 100);
        assert_eq!(events.len(), 1);

        let events = s.process(0.0, 0.0, false, 150);
        assert!(events.is_empty());

        let events = s.process(440.0, 0.9, true, 180);
        assert!(events.is_empty());

        let events = s.process(440.0, 0.9, true, 300);
        assert!(events.is_empty());

        let events = s.process(0.0, 0.0, false, 400);
        assert!(events.is_empty());

        let events = s.process(0.0, 0.0, false, 500);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Ended));
    }

    #[test]
    fn flush_ends_active_note() {
        let mut s = NoteSegmenter::new();
        s.min_note_duration_ms = 50;

        s.process(440.0, 0.9, true, 0);
        s.process(440.0, 0.9, true, 100);

        let events = s.flush(200);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Ended));
        assert_eq!(events[0].note_name, "A4");
    }

    #[test]
    fn reset_clears_state() {
        let mut s = NoteSegmenter::new();
        s.min_note_duration_ms = 50;

        s.process(440.0, 0.9, true, 0);
        s.process(440.0, 0.9, true, 100);

        s.reset();

        let events = s.process(523.25, 0.9, true, 200);
        assert!(events.is_empty());

        let events = s.process(523.25, 0.9, true, 300);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Started));
        assert_eq!(events[0].note_name, "C5");
    }

    #[test]
    fn unvoiced_start_no_event() {
        let mut s = NoteSegmenter::new();
        let events = s.process(440.0, 0.0, false, 0);
        assert!(events.is_empty());
    }

    #[test]
    fn silence_duration_accumulates() {
        let mut s = NoteSegmenter::new();
        s.hold_silence_ms = 100;
        s.min_note_duration_ms = 50;

        s.process(440.0, 0.9, true, 0);
        s.process(440.0, 0.9, true, 100);

        let events = s.process(0.0, 0.0, false, 180);
        assert!(events.is_empty());

        let events = s.process(0.0, 0.0, false, 300);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, NoteEventType::Ended));
        assert_eq!(events[0].note_name, "A4");
    }
}
