# CLI Mode

## Goal

Add a command-line mode to FerroTone for batch processing (WAV files), timed microphone capture, and programmable testing — without requiring a GUI or Tauri runtime.

---

## Crate

New workspace member: `crates/ferrotone-cli/`

Dependencies:
- `ferrotone-core` (path)
- `clap` 4 (derive feature) — CLI argument parsing
- `hound` 3 — WAV file reading
- `serde` / `serde_json` — JSON output formatting
- `tracing` / `tracing-subscriber` — diagnostic logging (stderr)

The crate has **zero Tauri dependency** — it talks directly to `ferrotone-core`.

---

## CLI Interface

```
ferrotone-cli [OPTIONS] --file <WAV_PATH>
ferrotone-cli [OPTIONS] --duration <SECONDS>
```

| Argument | Short | Long | Description |
|----------|-------|------|-------------|
| Config path | `-c` | `--config` | Custom config TOML file instead of platform default |
| Note event log | `-l` | `--note-log` | Write note event log as JSON Lines to this file |
| Pitch frame log | `-p` | `--pitch-log` | Write per-frame pitch data as JSON Lines to this file |
| File input | `-f` | `--file` | WAV file to process offline (conflicts with `--duration`) |
| Duration | `-d` | `--duration` | Run microphone capture for N seconds then exit (conflicts with `--file`) |
| Help | `-h` | `--help` | Print help |

At least one of `--file` / `--duration` must be provided.

---

## Output Files

### Note Event Log (`--note-log`)

JSON Lines — one JSON object per line, each representing a note `Started` or `Ended` event:

```jsonl
{"event_type":"started","note_name":"A4","midi":69,"cents_deviation":2.3,"clarity":0.94,"duration_ms":0,"timestamp_ms":1234}
{"event_type":"ended","note_name":"A4","midi":69,"cents_deviation":1.8,"clarity":0.92,"duration_ms":856,"timestamp_ms":2090}
```

Fields correspond to `music::NoteEvent` in `ferrotone-core`.

### Pitch Frame Log (`--pitch-log`)

JSON Lines — one JSON object per line, each representing a single stabilized pitch estimate:

```jsonl
{"timestamp_ms":100,"frequency_hz":438.2,"note_name":"A4","midi":69.0,"cents_deviation":-7.0,"clarity":0.91,"voiced":true}
{"timestamp_ms":124,"frequency_hz":441.5,"note_name":"A4","midi":69.0,"cents_deviation":5.9,"clarity":0.93,"voiced":true}
```

---

## Stdout Output

### File mode

A tabular summary of detected notes is printed to stdout after processing:

```
Processed: samples/tenor_A4.wav
Duration: 3.42s  |  Sample rate: 44100 Hz  |  Channels: 1

Detected Notes:
  Started    Ended     Note    MIDI  Cents    Clarity  Duration
  0.100s     0.850s    A4      69    +2.3c    0.94     750ms
  0.950s     2.400s    A4      69    -1.8c    0.92     1450ms
  ...
```

### Mic mode

Live status updates (one line, refreshed in-place via `\r`) during capture:

```
Capturing...  3.2s / 10.0s  |  Current: A4  +2.3c  0.91 clarity
```

After capture, same tabular summary as file mode.

### Error handling

Errors are printed to stderr with a non-zero exit code. The JSON output files are only written on successful completion.

---

## Implementation

### Architecture

```
ferrotone-cli/src/
├── main.rs        # Entry point: parse args, load settings, dispatch
├── args.rs        # clap::Parser definition
├── file_mode.rs   # Offline WAV processing via CaptureEngine::feed_audio()
├── mic_mode.rs    # Timed microphone capture
└── note_log.rs    # JSON output writers
```

### File mode flow

1. Open WAV with `hound::WavReader`
2. Determine sample format (i16, i24, i32, f32), convert to mono `Vec<f32>` in [-1.0, 1.0]
3. Use the WAV file's sample rate (override config's sample_rate)
4. Create `SwipeDetector`, `CaptureConfig`, `CaptureEngine`
5. Feed audio in `buffer_size` chunks via `CaptureEngine::feed_audio()`
6. Run each returned `PitchFrame` through `NoteSegmenter`
7. Collect all pitch frames and note events
8. Write output files; print summary to stdout

### Mic mode flow

1. Create `SwipeDetector`, `CaptureConfig`, `CaptureEngine` from settings
2. Start capture engine
3. Spawn note segmenter thread that reads from pitch channel
4. Main thread waits for `duration` seconds (sleep)
5. Stop capture engine
6. Flush segmenter
7. Write output files; print summary to stdout

### Config loading

- `Settings::load()` with platform-default path (as today)
- If `--config` provided, load from that path instead via new `Settings::load_from(path)`

### Helper additions to `ferrotone-core`

- `Settings::load_from(path: &Path) -> Result<Self, ConfigError>` — load from arbitrary path
- `CaptureConfig::from_settings(settings: &Settings) -> Self` — avoids boilerplate repetition

---

## Testing

The CLI can be tested with the existing `samples/` directory (62 single-note WAV files):

```bash
# Process a sample file with note log
cargo run --bin ferrotone-cli -- -f samples/tenor_A4.wav -l notes.jsonl -p pitch.jsonl

# Process with custom config
cargo run --bin ferrotone-cli -- -c custom.toml -f samples/male_C3.wav

# Timed microphone capture for 5 seconds
cargo run --bin ferrotone-cli -- -d 5 -l notes.jsonl
```

The `samples/` directory provides ground truth (via `tone_map.json`) to verify per-file results.
