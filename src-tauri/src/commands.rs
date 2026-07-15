use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use ferrotone_core::audio::{device::AudioDeviceInfo, CaptureConfig, CaptureEngine};
use ferrotone_core::config::Settings;
use ferrotone_core::music::{
    cents_off, hz_to_midi, midi_to_note_name, nearest_equal_tempered_freq, NoteSegmenter,
};
use ferrotone_core::pitch::{swipe::SwipeDetector, PitchFrame};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PitchFrameEvent {
    pub frequency_hz: f32,
    pub note_name: String,
    pub cents_deviation: f32,
    pub clarity: f32,
    pub voiced: bool,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VolumeFrameEvent {
    pub rms_level: f32,
    pub timestamp_ms: u64,
}

fn spawn_pitch_forwarder(rx: Receiver<PitchFrame>, app: AppHandle) {
    std::thread::spawn(move || {
        tracing::debug!("pitch event-forwarder thread started");
        let mut segmenter = NoteSegmenter::new();
        let hold_ms = segmenter.hold_silence_ms;

        loop {
            match rx.recv_timeout(Duration::from_millis(hold_ms)) {
                Ok(frame) => {
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
                        voiced: frame.voiced,
                        timestamp_ms: frame.timestamp_ms,
                    };
                    if let Err(e) = app.emit("pitch-frame", payload) {
                        tracing::warn!(error = %e, "failed to emit pitch-frame event");
                    }

                    for event in segmenter.process(
                        frame.frequency_hz,
                        frame.clarity,
                        frame.voiced,
                        frame.timestamp_ms,
                    ) {
                        if let Err(e) = app.emit("note-event", &event) {
                            tracing::warn!(error = %e, "failed to emit note-event");
                        }
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    for event in segmenter.flush(now) {
                        if let Err(e) = app.emit("note-event", &event) {
                            tracing::warn!(error = %e, "failed to emit note-event");
                        }
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        for event in segmenter.flush(now) {
            if let Err(e) = app.emit("note-event", &event) {
                tracing::warn!(error = %e, "failed to emit note-event");
            }
        }
        tracing::debug!("pitch event-forwarder thread exiting");
    });
}

#[tauri::command]
pub async fn start_capture(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    tracing::info!("start_capture command invoked");

    let settings = state.settings.lock().map_err(|e| {
        tracing::error!(error = %e, "failed to lock settings");
        e.to_string()
    })?;

    let mut engine = state.engine.lock().map_err(|e| {
        tracing::error!(error = %e, "failed to lock engine state");
        e.to_string()
    })?;

    if engine.is_some() {
        tracing::warn!("start_capture called but capture already running");
        return Err("capture already running".into());
    }

    let audio = &settings.audio;
    let noise = &settings.noise_cancellation;

    tracing::debug!(
        "creating SwipeDetector(sample_rate={}, buffer_size={})",
        audio.sample_rate,
        audio.buffer_size
    );
    let detector = SwipeDetector::new(
        audio.sample_rate,
        audio.buffer_size,
        noise.confidence_threshold,
    )
    .map_err(|e| {
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
        noise_cancellation_enabled: noise.enabled,
        input_gain: noise.input_gain,
        rms_gate_enabled: noise.rms_gate_enabled,
        rms_threshold: noise.rms_threshold,
        confidence_gate_enabled: noise.confidence_gate_enabled,
        confidence_threshold: noise.confidence_threshold,
        bandpass_enabled: noise.bandpass_enabled,
        bandpass_low: noise.bandpass_low,
        bandpass_high: noise.bandpass_high,
        rnnoise_enabled: noise.rnnoise_enabled,
    };

    drop(settings);

    let mut capture = CaptureEngine::new(Box::new(detector), config);
    let rx = capture.pitch_receiver().clone();
    let vr = capture.volume_receiver().clone();

    tracing::info!("calling CaptureEngine::start()");
    capture.start().map_err(|e| {
        tracing::error!(error = %e, "capture engine failed to start");
        e.to_string()
    })?;

    let app_v = app_handle.clone();
    spawn_pitch_forwarder(rx, app_handle);
    std::thread::spawn(move || {
        tracing::debug!("volume event-forwarder thread started");
        while let Ok(frame) = vr.recv() {
            let payload = VolumeFrameEvent {
                rms_level: frame.rms_level,
                timestamp_ms: frame.timestamp_ms,
            };
            if let Err(e) = app_v.emit("volume-frame", payload) {
                tracing::warn!(error = %e, "failed to emit volume-frame event");
            }
        }
        tracing::debug!("volume event-forwarder thread exiting");
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

// ---------------------------------------------------------------------------
// Recording Control Widget commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    settings: Settings,
) -> Result<(), String> {
    tracing::info!("update_settings command invoked");

    let dirty = {
        let mut current = state.settings.lock().map_err(|e| e.to_string())?;
        if *current == settings {
            return Ok(());
        }
        *current = settings.clone();
        true
    };

    if dirty {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        if let Some(capture) = engine.take() {
            tracing::info!("restarting capture engine with new settings");
            let mut cap = capture;
            cap.stop().map_err(|e| e.to_string())?;
            drop(cap);

            let audio = &settings.audio;
            let noise = &settings.noise_cancellation;

            let detector = SwipeDetector::new(
                audio.sample_rate,
                audio.buffer_size,
                noise.confidence_threshold,
            )
            .map_err(|e| e.to_string())?;

            let config = CaptureConfig {
                sample_rate: audio.sample_rate,
                buffer_size: audio.buffer_size,
                device_name: if audio.device_name.is_empty() {
                    None
                } else {
                    Some(audio.device_name.clone())
                },
                noise_cancellation_enabled: noise.enabled,
                input_gain: noise.input_gain,
                rms_gate_enabled: noise.rms_gate_enabled,
                rms_threshold: noise.rms_threshold,
                confidence_gate_enabled: noise.confidence_gate_enabled,
                confidence_threshold: noise.confidence_threshold,
                bandpass_enabled: noise.bandpass_enabled,
                bandpass_low: noise.bandpass_low,
                bandpass_high: noise.bandpass_high,
                rnnoise_enabled: noise.rnnoise_enabled,
            };

            let mut capture = CaptureEngine::new(Box::new(detector), config);
            let rx = capture.pitch_receiver().clone();
            let vr = capture.volume_receiver().clone();
            capture.start().map_err(|e| e.to_string())?;

            let app_v = app_handle.clone();
            spawn_pitch_forwarder(rx, app_handle);
            std::thread::spawn(move || {
                while let Ok(frame) = vr.recv() {
                    let payload = VolumeFrameEvent {
                        rms_level: frame.rms_level,
                        timestamp_ms: frame.timestamp_ms,
                    };
                    if let Err(e) = app_v.emit("volume-frame", payload) {
                        tracing::warn!(error = %e, "failed to emit volume-frame event");
                    }
                }
            });

            *engine = Some(capture);
            tracing::info!("capture engine restarted successfully");
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("save_settings command invoked");
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.save().map_err(|e| e.to_string())?;
    tracing::info!("settings saved to disk");
    Ok(())
}

#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<Vec<AudioDeviceInfo>, String> {
    tracing::debug!("list_devices command invoked");

    let default_device_name = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.audio.device_name.clone()
    };

    ferrotone_core::audio::device::list_input_devices(&default_device_name)
}
