use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use ferrotone_core::audio::{CaptureConfig, CaptureEngine};
use ferrotone_core::music::{
    cents_off, hz_to_midi, midi_to_note_name, nearest_equal_tempered_freq,
};
use ferrotone_core::pitch::swipe::SwipeDetector;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PitchFrameEvent {
    pub frequency_hz: f32,
    pub note_name: String,
    pub cents_deviation: f32,
    pub clarity: f32,
    pub timestamp_ms: u64,
}

#[tauri::command]
pub async fn start_capture(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    tracing::info!("start_capture command invoked");

    let mut engine = state.engine.lock().map_err(|e| {
        tracing::error!(error = %e, "failed to lock engine state");
        e.to_string()
    })?;

    if engine.is_some() {
        tracing::warn!("start_capture called but capture already running");
        return Err("capture already running".into());
    }

    let audio = &state.settings.audio;

    tracing::debug!(
        "creating SwipeDetector(sample_rate={}, buffer_size={})",
        audio.sample_rate,
        audio.buffer_size
    );
    let detector = SwipeDetector::new(audio.sample_rate, audio.buffer_size).map_err(|e| {
        tracing::error!(error = %e, "failed to create SwipeDetector");
        e.to_string()
    })?;

    let config = CaptureConfig {
        sample_rate: audio.sample_rate,
        buffer_size: audio.buffer_size,
        device_name: if audio.device_name.is_empty() {
            None
        } else {
            Some(audio.device_name.clone())
        },
    };

    let mut capture = CaptureEngine::new(Box::new(detector), config);
    let rx = capture.pitch_receiver().clone();

    tracing::info!("calling CaptureEngine::start()");
    capture.start().map_err(|e| {
        tracing::error!(error = %e, "capture engine failed to start");
        e.to_string()
    })?;

    let app = app_handle.clone();
    tracing::debug!("spawning event-forwarder thread");
    std::thread::spawn(move || {
        tracing::debug!("event-forwarder thread started");
        while let Ok(frame) = rx.recv() {
            let note = midi_to_note_name(hz_to_midi(frame.frequency_hz));
            let cents = cents_off(
                frame.frequency_hz,
                nearest_equal_tempered_freq(frame.frequency_hz),
            );
            let payload = PitchFrameEvent {
                frequency_hz: frame.frequency_hz,
                note_name: note,
                cents_deviation: cents,
                clarity: frame.clarity,
                timestamp_ms: frame.timestamp_ms,
            };
            if let Err(e) = app.emit("pitch-frame", payload) {
                tracing::warn!(error = %e, "failed to emit pitch-frame event");
            }
        }
        tracing::debug!("event-forwarder thread exiting");
    });

    *engine = Some(capture);
    tracing::info!("capture started successfully");
    Ok(())
}

#[tauri::command]
pub async fn stop_capture(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("stop_capture command invoked");
    let mut engine = state.engine.lock().map_err(|e| {
        tracing::error!(error = %e, "failed to lock engine state");
        e.to_string()
    })?;

    if let Some(mut capture) = engine.take() {
        tracing::debug!("stopping capture engine");
        capture.stop().map_err(|e| {
            tracing::error!(error = %e, "capture engine failed to stop");
            e.to_string()
        })?;
        tracing::info!("capture stopped successfully");
    } else {
        tracing::debug!("stop_capture called but no engine running");
    }
    Ok(())
}
