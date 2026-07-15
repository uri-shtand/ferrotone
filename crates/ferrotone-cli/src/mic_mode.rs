use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ferrotone_core::audio::{CaptureConfig, CaptureEngine};
use ferrotone_core::config::Settings;
use ferrotone_core::music::{
    cents_off, hz_to_midi, midi_to_note_name, nearest_equal_tempered_freq, NoteEvent, NoteSegmenter,
};
use ferrotone_core::pitch::swipe::SwipeDetector;

use crate::note_log::{write_note_log, write_pitch_log, PitchLogEntry};

pub struct MicModeResult {
    pub note_events: Vec<NoteEvent>,
    pub pitch_frames: Vec<PitchLogEntry>,
    pub duration_secs: f64,
}

pub fn run(
    duration_secs: f64,
    settings: &Settings,
    note_log_path: Option<&Path>,
    pitch_log_path: Option<&Path>,
) -> Result<(), String> {
    let result = capture_mic(duration_secs, settings)?;

    println!();
    println!("Capture finished: {:.2}s", result.duration_secs);
    println!();
    print_note_summary(&result.note_events);

    if let Some(nl_path) = note_log_path {
        write_note_log(nl_path, &result.note_events)?;
    }
    if let Some(pl_path) = pitch_log_path {
        write_pitch_log(pl_path, &result.pitch_frames)?;
    }

    Ok(())
}

fn capture_mic(duration_secs: f64, settings: &Settings) -> Result<MicModeResult, String> {
    let audio = &settings.audio;
    let noise = &settings.noise_cancellation;

    let detector = SwipeDetector::new(
        audio.sample_rate,
        audio.buffer_size,
        noise.confidence_threshold,
    )
    .map_err(|e| format!("failed to create detector: {e}"))?;

    let config = CaptureConfig::from_settings(audio, noise);

    let mut engine = CaptureEngine::new(Box::new(detector), config);
    let rx = engine.pitch_receiver().clone();

    engine
        .start()
        .map_err(|e| format!("failed to start capture: {e}"))?;

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let segmenter_handle = std::thread::spawn(move || {
        let hold_ms = 80u64;
        let mut segmenter = NoteSegmenter::new();
        let mut note_events: Vec<NoteEvent> = Vec::new();
        let mut pitch_frames: Vec<PitchLogEntry> = Vec::new();

        loop {
            if !running_clone.load(Ordering::SeqCst) {
                break;
            }

            match rx.recv_timeout(Duration::from_millis(hold_ms)) {
                Ok(frame) => {
                    let midi = hz_to_midi(frame.frequency_hz);
                    let note_name = midi_to_note_name(midi);
                    let cents = cents_off(
                        frame.frequency_hz,
                        nearest_equal_tempered_freq(frame.frequency_hz),
                    );

                    pitch_frames.push(PitchLogEntry {
                        timestamp_ms: frame.timestamp_ms,
                        frequency_hz: frame.frequency_hz,
                        note_name,
                        midi,
                        cents_deviation: cents,
                        clarity: frame.clarity,
                        voiced: frame.voiced,
                    });

                    for event in segmenter.process(
                        frame.frequency_hz,
                        frame.clarity,
                        frame.voiced,
                        frame.timestamp_ms,
                    ) {
                        note_events.push(event);
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    for event in segmenter.flush(now) {
                        note_events.push(event);
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        for event in segmenter.flush(now) {
            note_events.push(event);
        }

        (note_events, pitch_frames)
    });

    let start = Instant::now();
    let total_dur = Duration::from_secs_f64(duration_secs);

    while start.elapsed() < total_dur {
        let elapsed = start.elapsed();
        let remaining = total_dur - elapsed;
        let wait = remaining.min(Duration::from_millis(100));
        std::thread::sleep(wait);

        let pct = elapsed.as_secs_f64() / duration_secs * 100.0;
        print!(
            "\rCapturing...  {:.1}s / {:.0}s  ({:.0}%)",
            elapsed.as_secs_f64(),
            duration_secs,
            pct
        );
    }

    println!(
        "\rCapturing...  {:.0}s / {:.0}s  (100%)",
        duration_secs, duration_secs
    );
    println!("Stopping capture...");

    running.store(false, Ordering::SeqCst);
    engine
        .stop()
        .map_err(|e| format!("failed to stop capture: {e}"))?;

    let (note_events, pitch_frames) = segmenter_handle
        .join()
        .map_err(|_| "segmenter thread panicked".to_string())?;

    Ok(MicModeResult {
        note_events,
        pitch_frames,
        duration_secs,
    })
}

fn print_note_summary(events: &[NoteEvent]) {
    use ferrotone_core::music::NoteEventType;

    let started: Vec<&NoteEvent> = events
        .iter()
        .filter(|e| matches!(e.event_type, NoteEventType::Started))
        .collect();

    if started.is_empty() {
        return;
    }

    println!("Detected Notes:");
    println!(
        "  {:<8} {:<8} {:<6} {:<5} {:<7} {:<7} {:<8}",
        "Started", "Ended", "Note", "MIDI", "Cents", "Clarity", "Duration"
    );

    for start in &started {
        let end = events
            .iter()
            .skip_while(|e| {
                e.timestamp_ms != start.timestamp_ms
                    || !matches!(e.event_type, NoteEventType::Started)
            })
            .skip(1)
            .find(|e| {
                matches!(e.event_type, NoteEventType::Ended) && e.note_name == start.note_name
            });

        let start_sec = start.timestamp_ms as f64 / 1000.0;

        if let Some(e) = end {
            let end_sec = e.timestamp_ms as f64 / 1000.0;
            println!(
                "  {:<8.3}s {:<8.3}s {:<6} {:<5} {:<+7.1}c {:<7.2} {}ms",
                start_sec,
                end_sec,
                start.note_name,
                start.midi,
                start.cents_deviation,
                start.clarity,
                e.duration_ms,
            );
        } else {
            println!(
                "  {:<8.3}s {:<8}  {:<6} {:<5} {:<+7.1}c {:<7.2} -",
                start_sec,
                "(ongoing)",
                start.note_name,
                start.midi,
                start.cents_deviation,
                start.clarity,
            );
        }
    }
}
