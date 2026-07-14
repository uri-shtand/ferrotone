# Technical Specification: Stage D — Post-Processing Stabilization (The Jitter Killer)

This document is a technical specification for implementing Stage D (Post-Processing Stabilization) in the FerroTone Engine (Robust). 

It is designed to ingest raw, gated frequency inputs ($f_0$ from the solver) and output a stable, 
continuous pitch value optimized for real-time canvas rendering.

## 1. State Definition (Rust Struct)

To maintain context across consecutive audio frames, the processing struct must maintain local state. 
Initialize this struct in your Rust backend:

```rust
use std::collections::VecDeque;

pub struct StageDStabilizer {
    // Configuration Parameters
    pub median_window_size: usize,      // Default: 5 (must be odd: 3 or 5)
    pub smoothing_alpha: f32,           // Default: 0.75 (range: 0.1 to 1.0)
    pub max_consecutive_octaves: usize, // Default: 3 (failsafe for real leaps)

    // Running State
    pub prev_valid_pitch: Option<f32>,     // Last verified pitch
    pub prev_smoothed_pitch: Option<f32>,  // Last smoothed output
    pub history_buffer: VecDeque<f32>,     // Rolling buffer for Median Filter
    pub consecutive_octave_count: usize,   // Tracks sequential octave jumps
}
```

## 2. The Step-by-Step Algorithm

Every time a raw frequency is calculated (approx. every 16ms to 32ms), execute the following pipeline:

```
[ Raw Input: Option<f32> ]
   │
   ▼
   [ Step 1: Input & Reset Check ]
   │
   ├─► If None ──► Clear Buffers ──► Return None
   │
   ▼ (Input is Some(f32))
   [ Step 2: Octave Jump Guard ]
   │
   ▼
   [ Step 3: Median Filtering ]
   │
   ▼
   [ Step 4: One-Pole Smoothing ]
   │
   ▼
   [ Output: Option<f32> ]
```

### Step 1: Input Validation & Reset Logic

**Action:** Check if the incoming value is `Some(f32)` or `None` (gated out by Stage B/C).

**Logic:**
- If the input is `None`:
  - Clear `history_buffer`.
  - Reset `consecutive_octave_count` to 0.
  - Do not clear `prev_valid_pitch` or `prev_smoothed_pitch` immediately (this allows a brief hangover/decay if needed, though they can be set to `None` if the silence lasts longer than 150ms).
  - Return `None`.
- If the input is `Some(raw_hz)`: Proceed to Step 2.

### Step 2: Octave Jump Guard

This step filters out sudden, single-frame octave doubling ($2\times$) or halving ($0.5\times$) errors common in periodic frequency solvers.

**Logic:**
- If `prev_valid_pitch` is `None`, skip this guard. Set `validated_hz = raw_hz` and proceed to Step 3.
- If `prev_valid_pitch` is `Some(prev_hz)`:
  - Calculate the ratio: $R = \frac{\text{raw\_hz}}{\text{prev\_hz}}$.
  - Check if $R$ is an octave-up jump ($R \in [1.85, 2.15]$) OR an octave-down jump ($R \in [0.45, 0.55]$).
  - If an octave jump is detected:
    - Increment `consecutive_octave_count`.
    - If `consecutive_octave_count <= max_consecutive_octaves`:
      - Discard `raw_hz`. Substitute it: `validated_hz = prev_hz` (hold the line).
    - Else:
      - The user made a real octave leap. Accept `raw_hz`: `validated_hz = raw_hz` and reset `consecutive_octave_count = 0`.
  - If no octave jump is detected:
    - `validated_hz = raw_hz`
    - Reset `consecutive_octave_count = 0`.

### Step 3: Sliding Window Median Filter

This step removes sharp, single-frame noise spikes (outliers) without adding the phase delay of a moving average.

**Logic:**
- Push `validated_hz` to the back of `history_buffer`.
- If `history_buffer.len() > median_window_size`, pop the front element.
- If `history_buffer.len() < 3`:
  - `median_hz = validated_hz` (not enough history yet; skip sorting).
- If `history_buffer.len() >= 3`:
  - Create a temporary clone of `history_buffer`.
  - Sort the cloned list in ascending order.
  - Extract the middle element:
    $$\text{median\_hz} = \text{sorted\_buffer}\left[\frac{\text{len}}{2}\right]$$

### Step 4: One-Pole Low-Pass Smoothing

This step softens sub-semitone micro-jitter (tremor) to create a clean visual tracking path.

**Logic:**
- If `prev_smoothed_pitch` is `None`:
  - `smoothed_hz = median_hz`
- If `prev_smoothed_pitch` is `Some(prev_smooth)`:
  - Apply the exponential smoothing formula:
    $$\text{smoothed\_hz} = (\text{smoothing\_alpha} \cdot \text{median\_hz}) + ((1.0 - \text{smoothing\_alpha}) \cdot \text{prev\_smooth})$$
- **Update States:**
  - `prev_smoothed_pitch = Some(smoothed_hz)`
  - `prev_valid_pitch = Some(smoothed_hz)`
- **Return:** `Some(smoothed_hz)`.

## 3. Configuration Profiles

To integrate seamlessly with your existing configuration framework, expose these presets as JSON/Rust properties:

```json
{
  "pitch_engine": "V2_Robust",
  "stabilizer_settings": {
    "responsive": {
      "median_window_size": 3,
      "smoothing_alpha": 0.85,
      "max_consecutive_octaves": 2
    },
    "balanced": {
      "median_window_size": 5,
      "smoothing_alpha": 0.70,
      "max_consecutive_octaves": 3
    },
    "ultra_smooth": {
      "median_window_size": 5,
      "smoothing_alpha": 0.45,
      "max_consecutive_octaves": 4
    }
  }
}
```

- **Responsive Profile:** Best for rapid scale exercises and high-tempo songs.
- **Balanced Profile (Default Recommended):** Best for standard pop/rock vocal tracking.
- **Ultra Smooth Profile:** Best for slow ballads and long, sustained operatic notes.
