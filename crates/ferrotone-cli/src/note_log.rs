use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use ferrotone_core::music::{NoteEvent, NoteEventType};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NoteLogEntry {
    pub event_type: String,
    pub note_name: String,
    pub midi: u8,
    pub cents_deviation: f32,
    pub clarity: f32,
    pub duration_ms: u64,
    pub timestamp_ms: u64,
}

impl From<&NoteEvent> for NoteLogEntry {
    fn from(e: &NoteEvent) -> Self {
        Self {
            event_type: match e.event_type {
                NoteEventType::Started => "started".into(),
                NoteEventType::Ended => "ended".into(),
            },
            note_name: e.note_name.clone(),
            midi: e.midi,
            cents_deviation: e.cents_deviation,
            clarity: e.clarity,
            duration_ms: e.duration_ms,
            timestamp_ms: e.timestamp_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PitchLogEntry {
    pub timestamp_ms: u64,
    pub frequency_hz: f32,
    pub note_name: String,
    pub midi: f32,
    pub cents_deviation: f32,
    pub clarity: f32,
    pub voiced: bool,
}

pub fn write_note_log(path: &Path, events: &[NoteEvent]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("failed to create note log: {e}"))?;
    let mut writer = BufWriter::new(file);
    for event in events {
        let entry = NoteLogEntry::from(event);
        let line = serde_json::to_string(&entry)
            .map_err(|e| format!("failed to serialize note event: {e}"))?;
        writeln!(writer, "{line}").map_err(|e| format!("failed to write note log: {e}"))?;
    }
    tracing::info!("wrote {} note events to {}", events.len(), path.display());
    Ok(())
}

pub fn write_pitch_log(path: &Path, frames: &[PitchLogEntry]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("failed to create pitch log: {e}"))?;
    let mut writer = BufWriter::new(file);
    for entry in frames {
        let line = serde_json::to_string(entry)
            .map_err(|e| format!("failed to serialize pitch frame: {e}"))?;
        writeln!(writer, "{line}").map_err(|e| format!("failed to write pitch log: {e}"))?;
    }
    tracing::info!("wrote {} pitch frames to {}", frames.len(), path.display());
    Ok(())
}
