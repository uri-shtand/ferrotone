# Recording Control Widget

A frontend panel that exposes every audio pipeline parameter to the user with real-time feedback and a Save-to-disk button.

---

## 1. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  FRONTEND (React/TS)                     │
│                                                         │
│  PitchDisplay                           ┌─────────────┐ │
│  (pitch readout)                        │ AudioWidget │ │
│                                         │ ─────────── │ │
│  ┌──────────────────────┐               │ Gain:  ◉── │ │
│  │  A4                  │               │ RMS:  ◉── │ │
│  │  +2.3c  440.0 Hz     │               │ BPF:  ◉── │ │
│  │  [● Start] [■ Stop]  │               │ [Save]    │ │
│  └──────────────────────┘               └─────────────┘ │
│                                              ▲          │
│  invoke('get_settings')    invoke('save_settings')      │
│  invoke('update_settings', { ... })                     │
│  invoke('list_devices')                                  │
└──────────────────────────┬──────────────────────────────┘
                           │ Tauri IPC
┌──────────────────────────▼──────────────────────────────┐
│                  TAURI SHELL (src-tauri)                  │
│                                                          │
│  commands.rs: get_settings, update_settings,             │
│               save_settings, list_devices                │
│               start_capture, stop_capture                │
│                                                          │
│  state.rs: AppState { engine, settings (Mutex) }         │
└──────────────────────────┬──────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│              ferrotone-core (Rust)                       │
│                                                          │
│  Settings → CaptureConfig → CaptureEngine                │
│                              ├─ InputGain (scalar)       │
│                              ├─ BandpassFilter (biquad)  │
│                              ├─ RmsGate                  │
│                              ├─ PitchDetector            │
│                              └─ ConfidenceGate           │
└─────────────────────────────────────────────────────────┘
```

## 2. Data Flow

**Live editing** (user tweaks slider):
```
invoke('update_settings', { path: "rms_threshold", value: 0.02 })
  → AppState.settings updated
  → If engine.running && capture-relevant param:
      engine.stop()
      engine = CaptureEngine::new(detector, CaptureConfig::from(settings))
      engine.start()
      event emitter re-spawned
  → Ok
```

**Save to disk** (user clicks Save):
```
invoke('save_settings')
  → AppState.settings.save() → config.toml
  → Ok
```

**List devices** (widget mounts / refresh):
```
invoke('list_devices')
  → Vec<DeviceInfo> { name, channels, default }
```

## 3. Backend Changes

### 3.1 DSP Pipeline (new processing order)

```
Raw samples from cpal callback
  → InputGain (scale by gain factor)
  → BandpassFilter (biquad IIR, if enabled)
  → RmsGate (if enabled)
  → PitchDetector
  → ConfidenceGate (if enabled)
  → PitchFrame → channel → Tauri event → frontend
```

### 3.2 BandpassFilter (`gate/bandpass.rs`)

A second-order IIR bandpass using RBJ biquad topology:

```rust
pub struct BandpassFilter {
    enabled: bool,
    low_hz: f32,
    high_hz: f32,
    sample_rate: u32,
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}
```

`process(samples: &mut [f32])` filters in-place. Coeffs recalculated when `low_hz` / `high_hz` / `sample_rate` change via RBJ formulae.

### 3.3 Input Gain

Applied as a simple scalar multiplier on raw samples before any filtering:
```rust
pub fn process(samples: &mut [f32], gain: f32) {
    for s in samples.iter_mut() {
        *s *= gain;
    }
}
```

### 3.4 Settings & Config additions

**`Settings`** (unchanged — schema already has everything). Only additions:
- `NoiseCancellationSettings` gets: `input_gain: f32` (default 1.0)

**`CaptureConfig`** additions:
```rust
pub struct CaptureConfig {
    // existing
    pub noise_cancellation_enabled: bool,
    pub input_gain: f32,
    pub bandpass_enabled: bool,
    pub bandpass_low: f32,
    pub bandpass_high: f32,
}
```

**`NoiseCancellationSettings`** additions in `config.rs`:
```rust
pub struct NoiseCancellationSettings {
    // existing...
    pub input_gain: f32,  // 0.0–2.0, default 1.0
}
```

### 3.5 New Tauri Commands (`commands.rs`)

| Command | Args | Returns | Side Effects |
|---------|------|---------|-------------|
| `get_settings` | none | `Settings` | Reads from AppState |
| `update_settings` | `settings: Settings` | `()` | Replaces AppState.settings; restarts engine if running |
| `save_settings` | none | `()` | Writes config.toml via Settings::save() |
| `list_devices` | none | `Vec<DeviceInfo>` | Enumerates cpal input devices |

### 3.6 Engine restart logic

`update_settings` determines if restart is needed:
```rust
fn is_capture_relevant(old: &Settings, new: &Settings) -> bool {
    old.audio != new.audio
        || old.noise_cancellation.enabled != new.noise_cancellation.enabled
        || old.noise_cancellation.rms_gate_enabled != new.noise_cancellation.rms_gate_enabled
        || (old.noise_cancellation.rms_threshold - new.noise_cancellation.rms_threshold).abs() > f32::EPSILON
        || ...
}
```

If engine is running and settings changed, stop and restart.

## 4. Frontend

### 4.1 Types (`src/types.ts`)

```typescript
export interface Settings {
  audio: AudioSettings;
  noise_cancellation: NoiseCancellationSettings;
  user: UserSettings;
}

export interface AudioSettings {
  sample_rate: number;
  buffer_size: number;
  device_name: string;
  algorithm: string;
}

export interface NoiseCancellationSettings {
  enabled: boolean;
  input_gain: number;
  rms_gate_enabled: boolean;
  rms_threshold: number;
  confidence_gate_enabled: boolean;
  confidence_threshold: number;
  bandpass_enabled: boolean;
  bandpass_low: number;
  bandpass_high: number;
}

export interface DeviceInfo {
  name: string;
  is_default: boolean;
}
```

### 4.2 Hook: `useAudioSettings`

```typescript
interface UseAudioSettingsReturn {
  settings: Settings | null;
  loading: boolean;
  dirty: boolean;
  updateAudio: (partial: Partial<AudioSettings>) => Promise<void>;
  updateNoiseCancellation: (partial: Partial<NoiseCancellationSettings>) => Promise<void>;
  save: () => Promise<void>;
  devices: DeviceInfo[];
  refreshDevices: () => Promise<void>;
}
```

### 4.3 Component: `AudioControls`

Collapsible panel below PitchDisplay:

```
┌─ ⚙ Audio Controls ────────────────────┐
│ [▼] Input                              │
│  Gain       ──────◉──────────  1.0x    │
│  Device     [ Default Mic        ▼ ]   │
│                                        │
│ [▼] Volume Gate                        │
│  Enabled    [■■■] ON                   │
│  Threshold  ────◉──────────  0.01      │
│                                        │
│ [▼] Confidence Gate                    │
│  Enabled    [■■■] ON                   │
│  Threshold  ────◉──────────  0.30      │
│                                        │
│ [▼] Bandpass Filter                    │
│  Enabled    [■■■] ON                   │
│  Low Cut    ──◉────────────  80 Hz     │
│  High Cut   ────────◉──────  1000 Hz   │
│                                        │
│ ┌──────────────────────────────────┐   │
│ │          [ Save Settings ]       │   │
│ └──────────────────────────────────┘   │
│   (changes apply live; save to disk)   │
└────────────────────────────────────────┘
```

- Collapsible sections with arrow toggles
- Range sliders show current value as number
- Section toggles are styled switches
- Save button disabled when `!dirty`
- Dirty state: unsaved changes exist since last save

### 4.4 App Wiring

- `AudioControls` toggle button (gear icon) in the header
- `PitchDisplay` + `AudioControls` both read from `usePitchCapture` and `useAudioSettings`

## 5. Implementation Sequence

1. `BandpassFilter` in Rust (`gate/bandpass.rs`)
2. `input_gain` scalar in `gate/gain.rs`
3. Expand `CaptureConfig` with new fields
4. Wire new DSP chain in `spawn_dsp_worker`
5. Add `list_devices` command
6. Add `get_settings`, `update_settings`, `save_settings` commands
7. `AppState.settings` → `Mutex`, engine restart logic
8. Register commands in `lib.rs`
9. Frontend types, hook, component, styles
10. Wire into `App.tsx`
11. Tests (Rust + frontend)
12. Update README.md
