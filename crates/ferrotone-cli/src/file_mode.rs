use std::path::Path;

use ferrotone_core::audio::{CaptureConfig, CaptureEngine};
use ferrotone_core::config::{AudioSettings, NoiseCancellationSettings};
use ferrotone_core::music::{
    cents_off, hz_to_midi, midi_to_note_name, nearest_equal_tempered_freq, NoteEvent, NoteSegmenter,
};
use ferrotone_core::pitch::swipe::SwipeDetector;

use crate::note_log::{write_note_log, write_pitch_log, PitchLogEntry};

pub struct FileModeResult {
    pub note_events: Vec<NoteEvent>,
    pub pitch_frames: Vec<PitchLogEntry>,
    pub duration_secs: f64,
    pub sample_rate: u32,
}

pub fn run(
    path: &Path,
    note_log_path: Option<&Path>,
    pitch_log_path: Option<&Path>,
) -> Result<(), String> {
    let result = process_file(path)?;

    println!();
    println!("Processed: {}", path.display());
    println!(
        "Duration: {:.2}s  |  Sample rate: {} Hz",
        result.duration_secs, result.sample_rate
    );
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

fn process_file(path: &Path) -> Result<FileModeResult, String> {
    let mut reader =
        hound::WavReader::open(path).map_err(|e| format!("failed to open WAV file: {e}"))?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    let samples = read_samples(&mut reader, spec)?;

    let duration_secs = samples.len() as f64 / sample_rate as f64;

    let audio = AudioSettings {
        sample_rate,
        buffer_size: 1024,
        ..AudioSettings::default()
    };
    let noise = NoiseCancellationSettings::default();
    let config = CaptureConfig::from_settings(&audio, &noise);

    let detector = SwipeDetector::new(sample_rate, config.buffer_size, noise.confidence_threshold)
        .map_err(|e| format!("failed to create detector: {e}"))?;

    let buffer_size = config.buffer_size;
    let mut engine = CaptureEngine::new(Box::new(detector), config);
    let mut segmenter = NoteSegmenter::new();
    let mut note_events: Vec<NoteEvent> = Vec::new();
    let mut pitch_frames: Vec<PitchLogEntry> = Vec::new();

    for (chunk_idx, chunk) in samples.chunks(buffer_size).enumerate() {
        let frames = engine.feed_audio(chunk);

        let chunk_start_ms =
            ((chunk_idx * buffer_size) as f64 / sample_rate as f64 * 1000.0) as u64;
        let chunk_end_ms = (((chunk_idx + 1) * buffer_size).min(samples.len()) as f64
            / sample_rate as f64
            * 1000.0) as u64;

        for frame in &frames {
            let midi = hz_to_midi(frame.frequency_hz);
            let note_name = midi_to_note_name(midi);
            let cents = cents_off(
                frame.frequency_hz,
                nearest_equal_tempered_freq(frame.frequency_hz),
            );

            let ts = (chunk_start_ms + chunk_end_ms) / 2;

            pitch_frames.push(PitchLogEntry {
                timestamp_ms: ts,
                frequency_hz: frame.frequency_hz,
                note_name,
                midi,
                cents_deviation: cents,
                clarity: frame.clarity,
                voiced: frame.voiced,
            });

            for event in segmenter.process(frame.frequency_hz, frame.clarity, frame.voiced, ts) {
                note_events.push(event);
            }
        }
    }

    let final_ts = (duration_secs * 1000.0) as u64;
    for event in segmenter.flush(final_ts) {
        note_events.push(event);
    }

    Ok(FileModeResult {
        note_events,
        pitch_frames,
        duration_secs,
        sample_rate,
    })
}

fn read_samples(
    reader: &mut hound::WavReader<impl std::io::Read>,
    spec: hound::WavSpec,
) -> Result<Vec<f32>, String> {
    let channels = spec.channels as usize;

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .filter_map(|s| s.ok())
            .step_by(channels)
            .collect(),
        hound::SampleFormat::Int => {
            let max_val = 2_i32.pow(spec.bits_per_sample as u32 - 1) as f64;
            reader
                .samples::<i32>()
                .filter_map(|s| s.ok())
                .step_by(channels)
                .map(|s| (s as f64 / max_val) as f32)
                .collect()
        }
    };

    if samples.is_empty() {
        return Err("WAV file contains no samples".into());
    }

    Ok(samples)
}

fn print_note_summary(events: &[NoteEvent]) {
    use ferrotone_core::music::NoteEventType;

    let started: Vec<&NoteEvent> = events
        .iter()
        .filter(|e| matches!(e.event_type, NoteEventType::Started))
        .collect();

    if started.is_empty() {
        println!("No notes detected.");
        return;
    }

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
