# Real-Time Graphs

Two real-time canvas-based graphs display audio data over a sliding 60-second window, rendered below the main pitch display.

---

## Volume Graph

**File:** `src/components/VolumeGraph.tsx`
**Hook:** `src/hooks/useVolumeCapture.ts`

### Purpose

Shows the singer's loudness (volume) over time, providing visual feedback on dynamic control — whether the singer is too quiet, too loud, or maintaining consistent breath support.

### Data flow

```
Mic → DSP worker → compute_rms() → VolumeFrame { rms_level, timestamp_ms }
                                         ↓
                                  Tauri IPC "volume-frame" event
                                         ↓
                              useVolumeCapture hook (60s buffer)
                                         ↓
                               VolumeGraph canvas component
```

### Y-axis

- **Scale:** dB (decibels), range –60 dB to 0 dB
- **Conversion:** `RMS → dB` via `20 * log10(rms)`
- **Grid lines:** at –60, –40, –20, 0 dB
- **Label:** "dB"

### Visual style

- Dark background (`#1a1a2e`)
- Green line (`#4ade80`) with semi-transparent green gradient fill
- Line width: 1.5 px
- 120 px height

---

## Pitch Graph

**File:** `src/components/PitchGraph.tsx`
**Hook:** `src/hooks/usePitchGraphCapture.ts`

### Purpose

Shows the singer's pitch trajectory over time, revealing whether they are hitting notes accurately, sliding between notes, or drifting sharp/flat.

### Data flow

```
Mic → DSP worker → PitchDetector → PitchFrame { frequency_hz, cents_deviation, clarity, timestamp_ms }
                                         ↓
                             Tauri IPC "pitch-frame" event
                                         ↓
                          usePitchGraphCapture hook (60s buffer)
                                         ↓
                               PitchGraph canvas component
```

### Y-axis

- **Scale:** MIDI note number, range C2 (MIDI 36) to C6 (MIDI 84)
- **Conversion:** `Hz → MIDI` via `69 + 12 * log2(f / 440)`
- **Grid lines:** at octave boundaries (C2, C3, C4, C5, C6)
- **Labels:** Note names (e.g. "C4", "A4")
- **Label:** "MIDI"

### Line coloring

The graph line changes color based on cents deviation from the nearest equal-tempered note:

| Deviation | Color   | Hex       |
|-----------|---------|-----------|
| < 10¢     | Green   | `#4ade80` |
| 10–30¢    | Amber   | `#fbbf24` |
| > 30¢     | Red     | `#ef4444` |

### Visual style

- Same dark background as volume graph
- Per-segment stroke coloring based on average cents deviation between consecutive points
- Gradient fill uses the latest frame's cents deviation color
- Semi-transparent fill (`b3` / `4d` / `0d` alpha suffixes)

---

## Common attributes

| Property | Value |
|----------|-------|
| Window   | 60-second sliding window |
| Render   | `requestAnimationFrame` (continuous) |
| Canvas   | HiDPI-aware via `devicePixelRatio` |
| Padding  | Left 40 px (labels), right 8 px, top 24 px, bottom 16 px |
| Empty state | Shows title text ("Volume" / "Pitch") centered |
| Clear on stop | Buffer emptied when capture stops |

---

## Component interface

Both graphs accept the same props:

```typescript
interface GraphProps {
  bufferRef: React.MutableRefObject<GraphFrame[]>;
  isCapturing: boolean;
}
```

Where `GraphFrame` is `VolumeFrame` for the volume graph and `PitchFrame` for the pitch graph.

---

## Future enhancements

- Target pitch overlay (ghost note line from a reference track)
- Zoom / pan controls
- Scrollable history beyond 60 seconds
- Click-to-inspect tooltip showing exact values at a point
- Auto-scaling Y-axis to match sung range
