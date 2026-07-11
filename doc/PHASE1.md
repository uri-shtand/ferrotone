# Phase 1: Native Core & Real-Time Pitch

**Goal:** Build a stable, ultra-low-latency pitch tracking pipeline with SWIPE' algorithm, 
exposed through a Tauri IPC layer with a live React frontend display. Fully tested with unit tests and integration tests.

---

## 1. Architecture Overview

```
┌──────────────────────────────────────────────┐
│              FRONTEND (React/TS)              │
│  PitchDisplay component + usePitchCapture hook │
│  Shows: note name, Hz, cents, clarity, status │
└──────────────────┬───────────────────────────┘
                   │ Tauri IPC (Commands + Events)
┌──────────────────▼───────────────────────────┐
│            TAURI SHELL (src-tauri)            │
│  commands.rs: start_capture / stop_capture    │
│  state.rs: AppState { CaptureEngine }         │
│  Polls pitch channel, emits "pitch-frame"     │
│  events to frontend at ~30-63 Hz              │
└──────────────────┬───────────────────────────┘
                   │ depends on (workspace)
┌──────────────────▼───────────────────────────┐
│          ferrotone-core (pure Rust)           │
│  ┌────────────┐  ┌──────────────────┐        │
│  │ PitchDetec │  │ CaptureEngine     │        │
│  │ tor trait  │  │ (cpal + channels) │        │
│  │  ─ SWIPE   │  │                  │        │
│  │  ─ Dummy   │  │                  │        │
│  └────────────┘  └──────────────────┘        │
│  ┌──────────────────────────────────┐        │
│  │ Music utilities (Hz, MIDI, note) │        │
│  └──────────────────────────────────┘        │
└──────────────────────────────────────────────┘
```

### Thread Model

| Thread | Role | Technology |
|--------|------|------------|
| Main (Tauri) | Window events, command routing | `tauri::Builder` |
| Audio I/O | Capture mic audio, push raw `f32` samples | `cpal` callback |
| DSP Worker | Pull samples, run `PitchDetector::process()`, push `PitchFrame` | `crossbeam::Channel` |
| Event Emitter | Poll pitch channel, emit Tauri events to frontend | Non-blocking loop |

### Data Flow

```
Mic → cpal callback → crossbeam SPSC (raw f32) → DSP Worker
  → PitchDetector::process() → crossbeam SPSC (PitchFrame)
  → Tauri shell → emit("pitch-frame", {hz, note, cents, clarity})
  → Frontend listen() → React state → PitchDisplay
```

---

## 2. Crate Structure

```
ferrotone/
├── Cargo.toml                  # Workspace root
├── crates/
│   └── ferrotone-core/
│       ├── Cargo.toml          # Pure Rust deps: pitch-core, cpal, crossbeam
│       └── src/
│           ├── lib.rs          # Re-exports public API
│           ├── error.rs        # Unified error type
│           ├── audio/
│           │   ├── mod.rs      # CaptureEngine, CaptureConfig
│           │   └── capture.rs  # cpal integration, thread management
│           ├── pitch/
│           │   ├── mod.rs      # PitchDetector trait + PitchFrame
│           │   ├── swipe.rs    # SWIPE' backend via pitch-core
│           │   └── dummy.rs    # Test double (emits known frequencies)
│           └── music/
│               ├── mod.rs
│               └── note.rs     # hz_to_midi, midi_to_note_name, cents_off
├── src-tauri/
│   ├── Cargo.toml              # Depends on ferrotone-core + tauri
│   └── src/
│       ├── main.rs
│       ├── lib.rs              # Builder setup, plugin registration
│       ├── commands.rs         # start_capture, stop_capture
│       ├── state.rs            # AppState
│       └── tests/              # Integration tests
│           ├── capture_commands.rs
│           └── pitch_events.rs
├── src/                        # Frontend
│   ├── components/
│   │   └── PitchDisplay.tsx    # Live pitch readout
│   ├── hooks/
│   │   └── usePitchCapture.ts  # Tauri event listener
│   ├── types.ts                # TypeScript interfaces
│   ├── App.tsx                 # Wired UI
│   ├── App.css
│   └── main.tsx
└── doc/
    ├── PLAN.md
    └── PHASE1.md               # This file
```

**Why two crates:** The `ferrotone-core` crate has zero Tauri dependency — it can be unit-tested in isolation, reused from WASM, 
and iterated on without compiling the Tauri shell. The `src-tauri` crate is the thin shell that wires core to the UI via IPC.

---

## 3. Core Crate Design (`ferrotone-core`)

### 3.1 `PitchDetector` Trait (`src/pitch/mod.rs`)

```rust
/// Algorithm-agnostic pitch detection interface.
///
/// Phase 1 ships SWIPE' via pitch-core.
/// Future phases add YIN, pYIN, or neural backends without changing callers.
pub trait PitchDetector: Send {
    /// Process a mono audio buffer, return zero or more pitch frames.
    fn process(&mut self, samples: &[f32]) -> Vec<PitchFrame>;

    /// Reset internal state (e.g., between capture sessions or after
    /// device sample rate change).
    fn reset(&mut self);
}

/// A single pitch estimate from the detector.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PitchFrame {
    pub frequency_hz: f32,
    pub clarity: f32,          // 0.0–1.0 confidence
    pub voiced: bool,          // false = unvoiced/silence
    pub timestamp_ms: u64,     // cumulative ms from stream start
}
```

### 3.2 SWIPE' Implementation (`src/pitch/swipe.rs`)

Wraps `pitch_core::PitchTracker` with a `SwipeEstimator`. 
The `pitch-core` crate handles resampling to 48 kHz internally via its linear resampler. 
The implementation converts `PitchFrame` from pitch-core's format into FerroTone's uniform `PitchFrame`.

```rust
pub struct SwipeDetector {
    tracker: pitch_core::PitchTracker,
    frame_count: u64,
}

impl SwipeDetector {
    pub fn new(sample_rate: u32, buffer_size: usize) -> Result<Self, DetectionError> {
        let estimator = pitch_core::SwipeEstimator::new(
            pitch_core::SwipeMode::Balanced,
        );
        let tracker = pitch_core::PitchTracker::new(
            estimator, sample_rate, buffer_size,
        ).map_err(|e| DetectionError::InitFailed(e.to_string()))?;
        Ok(Self { tracker, frame_count: 0 })
    }
}
```

### 3.3 Test Double (`src/pitch/dummy.rs`)

```rust
/// Emits a fixed frequency regardless of input. Used in tests
/// and as a development stand-in before real mic capture works.
pub struct DummyDetector {
    frequency_hz: f32,
    clarity: f32,
    voiced: bool,
}
```

### 3.4 Capture Engine (`src/audio/capture.rs`)

```rust
pub struct CaptureConfig {
    pub sample_rate: u32,        // e.g., 48000
    pub buffer_size: usize,      // e.g., 1024
    pub device_name: Option<String>,  // None = default
}

pub struct CaptureEngine {
    detector: Box<dyn PitchDetector>,
    config: CaptureConfig,
    pitch_tx: crossbeam_channel::Sender<PitchFrame>,
    pitch_rx: crossbeam_channel::Receiver<PitchFrame>,
    control_tx: crossbeam_channel::Sender<ControlSignal>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl CaptureEngine {
    pub fn new(
        detector: Box<dyn PitchDetector>,
        config: CaptureConfig,
    ) -> Self { /* ... */ }

    /// Start capture. Spawns DSP worker thread. Returns immediately.
    pub fn start(&mut self) -> Result<(), CaptureError> { /* ... */ }

    /// Stop capture. Signals worker thread to shut down.
    pub fn stop(&mut self) -> Result<(), CaptureError> { /* ... */ }

    /// Non-blocking receiver for pitch frames.
    pub fn pitch_receiver(&self) -> &crossbeam_channel::Receiver<PitchFrame> { /* ... */ }
}
```

**Threading detail:**

1. `start()` queries available audio devices via `cpal::default_input_device()`
2. Configures `cpal::StreamConfig` with `CaptureConfig` parameters
3. Spawns a build callback that pushes `f32` samples into a raw audio channel
4. Spawns the DSP worker thread that reads raw samples, feeds `PitchDetector::process()`, pushes `PitchFrame` results to `pitch_tx`
5. The main thread (Tauri) polls `pitch_rx` at its own pace

### 3.5 Music Utilities (`src/music/note.rs`)

```rust
/// A4 = 440 Hz = MIDI 69
pub const A4_HZ: f32 = 440.0;
pub const A4_MIDI: f32 = 69.0;
pub const SEMITONES_PER_OCTAVE: f32 = 12.0;

/// Convert frequency in Hz to MIDI note number (may be fractional).
pub fn hz_to_midi(frequency_hz: f32) -> f32 { /* ... */ }

/// Convert MIDI note number to nearest note name ("C4", "A#4", etc.).
pub fn midi_to_note_name(midi: f32) -> &'static str { /* ... */ }

/// Compute cents deviation between actual and target frequency.
/// Positive = sharp, negative = flat.
pub fn cents_off(actual_hz: f32, target_hz: f32) -> f32 {
    1200.0 * (actual_hz / target_hz).log2()
}

/// Return the nearest equal-tempered frequency for a given frequency.
pub fn nearest_equal_tempered_freq(frequency_hz: f32) -> f32 { /* ... */ }
```

### 3.6 Error Types (`src/error.rs`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("pitch detection init failed: {0}")]
    InitFailed(String),
    #[error("no audio device available")]
    NoDevice,
    #[error("unsupported sample rate: {0}")]
    UnsupportedSampleRate(u32),
    #[error("stream error: {0}")]
    StreamError(String),
}
```

---

## 4. Tauri Shell (`src-tauri`)

### 4.1 Commands (`commands.rs`)

```rust
#[tauri::command]
async fn start_capture(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    if engine.is_some() {
        return Err("capture already running".into());
    }

    let detector = SwipeDetector::new(48000, 1024)
        .map_err(|e| e.to_string())?;
    let config = CaptureConfig {
        sample_rate: 48000,
        buffer_size: 1024,
        device_name: None,
    };

    let mut capture = CaptureEngine::new(Box::new(detector), config);
    let rx = capture.pitch_receiver().clone();
    capture.start().map_err(|e| e.to_string())?;

    // Spawn event emitter
    let app = app_handle.clone();
    std::thread::spawn(move || {
        while let Ok(frame) = rx.recv() {
            let note = music::midi_to_note_name(hz_to_midi(frame.frequency_hz));
            let cents = music::cents_off(
                frame.frequency_hz,
                music::nearest_equal_tempered_freq(frame.frequency_hz),
            );
            let payload = PitchFrameEvent {
                frequency_hz: frame.frequency_hz,
                note_name: note,
                cents_deviation: cents,
                clarity: frame.clarity,
                timestamp_ms: frame.timestamp_ms,
            };
            let _ = app.emit("pitch-frame", payload);
        }
    });

    *engine = Some(capture);
    Ok(())
}

#[tauri::command]
async fn stop_capture(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    if let Some(mut capture) = engine.take() {
        capture.stop().map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

### 4.2 State (`state.rs`)

```rust
use std::sync::Mutex;
use ferrotone_core::audio::CaptureEngine;

pub struct AppState {
    pub engine: Mutex<Option<CaptureEngine>>,
}

impl AppState {
    pub fn new() -> Self {
        Self { engine: Mutex::new(None) }
    }
}
```

### 4.3 Application Setup (`lib.rs`)

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::start_capture,
            commands::stop_capture,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 4.4 Capabilities (`capabilities/default.json`)

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:event:default",
    "opener:default"
  ]
}
```

### 4.5 Event Payload Schema

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PitchFrameEvent {
    pub frequency_hz: f32,
    pub note_name: String,
    pub cents_deviation: f32,
    pub clarity: f32,
    pub timestamp_ms: u64,
}
```

---

## 5. Frontend

### 5.1 TypeScript Types (`src/types.ts`)

```typescript
export interface PitchFrame {
  frequency_hz: number;
  note_name: string;
  cents_deviation: number;
  clarity: number;
  timestamp_ms: number;
}

export interface CaptureStatus {
  isCapturing: boolean;
  error: string | null;
}
```

### 5.2 Hook (`src/hooks/usePitchCapture.ts`)

```typescript
import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { PitchFrame } from '../types';

interface UsePitchCaptureReturn {
  isCapturing: boolean;
  error: string | null;
  latestFrame: PitchFrame | null;
  history: PitchFrame[];
  start: () => Promise<void>;
  stop: () => Promise<void>;
}

const MAX_HISTORY = 100;

export function usePitchCapture(): UsePitchCaptureReturn {
  const [isCapturing, setIsCapturing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [latestFrame, setLatestFrame] = useState<PitchFrame | null>(null);
  const [history, setHistory] = useState<PitchFrame[]>([]);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    const setup = async () => {
      unlistenRef.current = await listen<PitchFrame>('pitch-frame', (event) => {
        setLatestFrame(event.payload);
        setHistory((prev) => {
          const next = [...prev, event.payload];
          return next.length > MAX_HISTORY ? next.slice(-MAX_HISTORY) : next;
        });
      });
    };
    setup();
    return () => {
      unlistenRef.current?.();
    };
  }, []);

  const start = useCallback(async () => {
    try {
      setError(null);
      await invoke('start_capture');
      setIsCapturing(true);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const stop = useCallback(async () => {
    try {
      await invoke('stop_capture');
      setIsCapturing(false);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  return { isCapturing, error, latestFrame, history, start, stop };
}
```

### 5.3 Component (`src/components/PitchDisplay.tsx`)

```
┌──────────────────────────────────────┐
│  ┌──────────┐                        │
│  │   A4     │  ← note name (large)   │
│  │          │  ← color: green/amber/ │
│  │  +2.3c   │    white based on      │
│  └──────────┘    cents deviation     │
│                                    │
│  440.0 Hz      clarity: ████░ 94%  │
│                                    │
│  ────┬──────────────┬────         │
│      │  ◉           │  ← cents bar│
│  ────┴──────────────┴────         │
│  -50    0        +50             │
│                                    │
│  [ ● Start ]  [ ■ Stop ]          │
│         Status: Listening...       │
└──────────────────────────────────────┘
```

```typescript
interface PitchDisplayProps {
  isCapturing: boolean;
  latestFrame: PitchFrame | null;
  onStart: () => void;
  onStop: () => void;
}
```

**Color rules:**
- `|cents| < 5` → bright green (in tune)
- `5 ≤ |cents| < 25` → amber (close)
- `|cents| ≥ 25` → red (off)
- `clarity < 0.5` → dimmed white (low confidence)

### 5.4 App Wiring (`src/App.tsx`)

Replace the default Tauri greet template with:
- `PitchDisplay` component
- Start/Stop buttons wired to `usePitchCapture`
- Status indicator (idle, listening, error)

---

## 6. Testing Strategy

### 6.1 Unit Tests (Rust — `ferrotone-core`)

| Test File | Test | What It Validates |
|-----------|------|-------------------|
| `tests/note_conversion.rs` | `hz_to_midi_roundtrip` | A4=440 → MIDI 69.0, C4=261.63 → MIDI 60.0 |
| `tests/note_conversion.rs` | `midi_to_note_names_golden` | Golden table for notes 0..127 matches reference |
| `tests/note_conversion.rs` | `cents_off_in_tune` | 440 Hz vs 440 Hz = 0 cents |
| `tests/note_conversion.rs` | `cents_off_semitone` | 440 Hz vs 466.16 Hz ≈ +100 cents |
| `tests/note_conversion.rs` | `nearest_equal_tempered` | Known frequencies map to correct ET target |
| `tests/swipe_detector.rs` | `sine_a4_440hz` | SWIPE' on synthesized A4 sine returns 440 ± 2 Hz |
| `tests/swipe_detector.rs` | `sine_c4_261hz` | SWIPE' on synthesized C4 sine returns 261.63 ± 2 Hz |
| `tests/swipe_detector.rs` | `sine_silence_returns_unvoiced` | Zero-signal buffer yields `voiced: false` |
| `tests/dummy_detector.rs` | `dummy_returns_known_freq` | DummyDetector returns configured frequency |
| `tests/capture_engine.rs` | `dummy_through_engine` | CaptureEngine with DummyDetector produces expected PitchFrames |
| `tests/capture_engine.rs` | `start_stop_cycle` | Engine can start and stop without errors |

### 6.2 Integration Tests (Rust — `src-tauri/tests/`)

Uses Tauri's `test` module with `MockRuntime`.

| Test File | Test | What It Validates |
|-----------|------|-------------------|
| `tests/capture_commands.rs` | `start_capture_returns_ok` | `start_capture` command routes correctly |
| `tests/capture_commands.rs` | `start_capture_twice_returns_error` | Double start returns error |
| `tests/capture_commands.rs` | `stop_without_start_returns_ok` | Stopping idle capture is safe |
| `tests/pitch_events.rs` | `start_emits_pitch_frames` | Events are emitted with correct shape after start |
| `tests/pitch_events.rs` | `stop_halts_events` | No events emitted after stop |

**Mock strategy:** Integration tests do not use a real microphone. Instead, `ferrotone-core` provides the `DummyDetector` that emits known frequencies. The Tauri shell wires this behind a feature flag during tests.

### 6.3 Frontend Tests (Vitest + `mockIPC`)

| Test File | Test | What It Validates |
|-----------|------|-------------------|
| `src/hooks/usePitchCapture.test.ts` | `listens_to_pitch_frame_event` | Hook starts listener on mount |
| `src/hooks/usePitchCapture.test.ts` | `updates_latest_frame_on_event` | State updates when event fires |
| `src/hooks/usePitchCapture.test.ts` | `maintains_history_buffer` | History caps at MAX_HISTORY |
| `src/hooks/usePitchCapture.test.ts` | `invokes_start_capture_command` | `start()` calls `invoke('start_capture')` |
| `src/hooks/usePitchCapture.test.ts` | `invokes_stop_capture_command` | `stop()` calls `invoke('stop_capture')` |
| `src/components/PitchDisplay.test.tsx` | `renders_idle_state` | Shows "Idle" when not capturing |
| `src/components/PitchDisplay.test.tsx` | `renders_pitch_data` | Shows note name, Hz, cents |
| `src/components/PitchDisplay.test.tsx` | `colors_by_cents_deviation` | Green/amber/red color logic |
| `src/components/PitchDisplay.test.tsx` | `calls_onstart_on_click` | Start button fires callback |

### 6.4 Test Commands

```bash
# All Rust tests (core + shell)
cargo test --workspace

# Rust tests with stdout (to see pitch values)
cargo test --workspace -- --nocapture

# Frontend tests
npx vitest run

# Frontend tests with watch mode
npx vitest

# All tests
cargo test --workspace && npx vitest run
```

---

## 7. Implementation Sequence (Ordered Steps)

### Step 1: Workspace & Core Crate Scaffolding

- Create root `Cargo.toml` workspace
- Create `crates/ferrotone-core/` with `Cargo.toml`
- Add dependencies: `pitch-core`, `cpal`, `crossbeam-channel`, `serde`, `serde_json`, `thiserror`
- Create module structure (`lib.rs`, `audio/`, `pitch/`, `music/`, `error.rs`)
- Verify `cargo build` passes

### Step 2: Music Utilities + Tests

- Implement `hz_to_midi`, `midi_to_note_name`, `cents_off`, `nearest_equal_tempered_freq`
- Write golden table tests in `tests/note_conversion.rs`
- Run `cargo test` — all pass

### Step 3: PitchDetector Trait + SWIPE' Implementation

- Define `PitchDetector` trait and `PitchFrame` struct
- Implement `SwipeDetector` wrapping `pitch_core::PitchTracker + SwipeEstimator`
- Implement `DummyDetector` for testing
- Write tests in `tests/swipe_detector.rs` with synthetic sine waves
- Write tests in `tests/dummy_detector.rs`
- Run `cargo test` — all pass

### Step 4: CaptureEngine + cpal Integration

- Implement `CaptureConfig` and `CaptureEngine`
- Integrate `cpal` input stream callback
- Wire `crossbeam::channel` between cpal callback and DSP worker
- Wire PitchDetector into DSP worker thread
- Write tests in `tests/capture_engine.rs` with DummyDetector
- Run `cargo test` — all pass

### Step 5: Tauri Shell Wiring

- Add `ferrotone-core` dependency to `src-tauri/Cargo.toml`
- Implement `AppState` with `Mutex<Option<CaptureEngine>>`
- Implement `start_capture` / `stop_capture` commands
- Implement event emitter thread
- Update `capabilities/default.json` permissions
- Write integration tests in `src-tauri/tests/`
- Run `cargo test --workspace` — all pass

### Step 6: Frontend

- Create `src/types.ts` with TypeScript interfaces
- Implement `usePitchCapture` hook
- Implement `PitchDisplay` component
- Wire `App.tsx` with start/stop controls
- Add Vitest + `@tauri-apps/api/mocks` dev dependencies
- Write frontend tests
- Run `npx vitest run` — all pass

### Step 7: Manual Verification

- `npm run tauri dev`
- Click Start → see pitch display showing live values while singing
- Click Stop → capture halts, display shows idle
- Verify no crashes on repeated start/stop cycles
- Verify error handling when no mic is available

---

## 8. Dependencies

### Rust (`ferrotone-core/Cargo.toml`)

| Crate | Version | Purpose |
|-------|---------|---------|
| `pitch-core` | 0.1 | SWIPE' pitch detector with streaming tracker |
| `cpal` | 0.15 | Cross-platform audio capture |
| `crossbeam-channel` | 0.5 | Lock-free SPSC channels for audio data transfer |
| `serde` | 1 | Serialization (with `derive` feature) |
| `serde_json` | 1 | JSON for debug/testing |
| `thiserror` | 2 | Ergonomic error types |

### Rust (`src-tauri/Cargo.toml` — additions)

| Crate | Version | Purpose |
|-------|---------|---------|
| `ferrotone-core` | workspace | Core crate |
| `tauri` | 2 | Already present |
| `tauri-plugin-opener` | 2 | Already present |

### Frontend (`package.json` — additions)

| Package | Version | Purpose |
|---------|---------|---------|
| `vitest` | ^3 | Test runner |
| `@testing-library/react` | ^16 | Component testing |
| `jsdom` | ^25 | DOM environment for tests |

---

## 9. Open Source Projects Used

| Project | License | How We Use It |
|---------|---------|---------------|
| [pitch-core](https://github.com/gzivdo/pitch-core) | MIT/Apache-2.0 | SWIPE' pitch detection with streaming `PitchTracker` API |
| [cpal](https://github.com/RustAudio/cpal) | MIT/Apache-2.0 | Cross-platform microphone capture |
| [crossbeam](https://github.com/crossbeam-rs/crossbeam) | MIT/Apache-2.0 | Lock-free channels for audio data transfer between threads |
| [serde](https://github.com/serde-rs/serde) | MIT/Apache-2.0 | Serialize Rust structs to JSON for Tauri IPC |
| [thiserror](https://github.com/dtolnay/thiserror) | MIT/Apache-2.0 | Derive macro for `std::error::Error` |
| [Tauri v2](https://github.com/tauri-apps/tauri) | MIT/Apache-2.0 | Desktop app shell, IPC runtime, test utilities |
| [React](https://github.com/facebook/react) | MIT | Frontend UI framework |
| [Vite](https://github.com/vitejs/vite) | MIT | Frontend bundler and dev server |
| [Vitest](https://github.com/vitest-dev/vitest) | MIT | Frontend test runner |
| [Testing Library](https://github.com/testing-library/react-testing-library) | MIT | React component testing utilities |

---

## 10. Open Source Projects for Inspiration

| Project | Stack | Why It's Relevant |
|---------|-------|-------------------|
| [neural-pitch](https://github.com/derekkinzo/neural-pitch) | Tauri 2 + Rust + React/TS | Same stack, same domain. Clean architecture with `neural-pitch-core` crate, `commands.rs`, `state.rs`. Has real integration tests using Tauri's mock runtime. Phase 1 of FerroTone mirrors its crate separation pattern. |
| [sing-attune](https://github.com/leonarduk/sing-attune) | Electron + Python (FastAPI) + TypeScript | Choir practice tool with real-time pitch overlay, per-note coloring (green/amber/red), MusicXML score loading. Good reference for pitch visualization UX (cents tolerance coloring) and phrase-based scoring. |
| [intonavio](https://github.com/pawelgawliczek/intonavio) | SwiftUI + Python + NestJS | iOS singing practice app with AI stem separation, real-time pitch on piano roll, phrase scoring. Reference for how to structure the full song-analysis pipeline (YouTube → stem separation → pitch profile → live overlay). |
| [MercuryPitch](https://github.com/mercurypitch/mercurypitch) | SolidJS + TypeScript + Vite | Browser-based vocal pitch practice. Uses YIN via `pitchfinder` JS library. React-like framework, Canvas-based piano roll, session tracking, and ADSR envelope. Good frontend UX reference. |
| [PratiCanto](https://github.com/douglas125/PratiCanto) | C++ (desktop, now web) | Long-running vocal training tool with spectrograms, pitch training, MIDI file playback, and voice extension analysis. Good reference for feature scope and professional use cases. |
| [RUST_Tuner](https://github.com/LenaVcht/RUST_Tuner) | Rust + cpal + YIN | Musical instrument tuner using same core stack (cpal + YIN). Demonstrates thread-safe cpal integration, circular buffer pattern, and console-based pitch output. |
| [rsac](https://github.com/Codeseys-Labs/rust-crossplat-audio-capture) | Rust | Per-app audio capture via WASAPI/CoreAudio/PipeWire. Not used in Phase 1, but critical reference for Phase 3 (system audio loopback). |

---

## 11. Commercial Products for Inspiration

| Product | Platform | Why It's Relevant |
|---------|----------|-------------------|
| [SingScope](http://www.singscope.com) | iOS | Clean real-time pitch display with scoring. The "gold standard" for mobile vocal training. Phase 2's Canvas rendering should aim for this level of visual clarity. |
| [Yousician](https://yousician.com) | iOS/Android/Desktop | Gamified music learning with note-highway scrolling. Reference for Phase 4 gamification design. |
| [Rocksmith](https://www.ubisoft.com/en-us/game/rocksmith) | Desktop | Real-time note tracking with scrolling highway. Similar visual metaphor to what Phase 2-4 will build. |
| [Smule](https://www.smule.com) | iOS/Android | Social karaoke with pitch visualization and scoring. Good reference for note accuracy scoring algorithms and real-time pitch coloring. |
| [Vocal Pitch Monitor](https://play.google.com/store/apps/details?id=com.tadaoyamaoka.vocalpitchmonitor) | Android | Simple, effective pitch display showing Hz and note name in real time. Reference for Phase 1 frontend simplicity. |

---

## 12. Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| SWIPE' over YIN | SWIPE' is the strongest pure-DSP option in pitch-core. Better accuracy for singing voice than basic YIN. Can swap later by implementing `PitchDetector` for another backend. |
| pitch-core over pitch-detection crate | pitch-core provides a streaming `PitchTracker` with built-in resampling. The older `pitch-detection` crate requires manual window management. pitch-core supports SWIPE', pYIN, and Praat-AC behind one trait. |
| Separate `ferrotone-core` crate | Zero Tauri dependency enables fast compilation, unit tests without a window, potential WASM reuse, and cleaner CI. |
| crossbeam channels over std::sync::mpsc | crossbeam's SPSC channels are multi-consumer capable and don't require `&mut` receiver, making them easier to share across threads. They also support `try_recv` for non-blocking polling. |
| DummyDetector test pattern | Allows full pipeline testing without a microphone. The dummy emits known frequencies, so test assertions are deterministic. |
| Event-driven frontend | The Rust core emits `PitchFrame` events at ~30-63 Hz. The React hook `listen()`s to these and updates state. No polling, no invoke() overhead per frame. |
| No Canvas in Phase 1 | Canvas-based pitch scrolling is Phase 2. Phase 1 keeps the frontend simple — a numeric/visual readout is sufficient to verify the pipeline works. |

---

## 13. Success Criteria

- [ ] `cargo test --workspace` passes with ≥15 tests (unit + integration)
- [ ] `npx vitest run` passes with ≥8 tests (frontend unit + component)
- [ ] `npm run tauri dev` launches without errors
- [ ] Clicking Start displays live note name (e.g., "A4") while singing
- [ ] Display updates 30-60 times per second in real-time
- [ ] Cents deviation indicator accurately reflects pitch accuracy
- [ ] Clarity meter drops during silence / low-volume input
- [ ] Clicking Stop halts capture and resets display to idle
- [ ] Rapid start/stop cycles do not crash the app
- [ ] App gracefully handles missing microphone (shows error, no crash)
- [ ] `cargo clippy --all-targets -- -D warnings` passes with no warnings
- [ ] `cargo fmt -- --check` passes with no formatting issues
