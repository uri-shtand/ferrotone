# Vocal Samples

Sustained single-note "ah" vowel recordings for testing and playback in FerroTone.

## Range

| Voice  | Notes         | MIDI      | Frequency   | Count |
|--------|---------------|-----------|-------------|-------|
| Male   | C2 – C5       | 36 – 72   | 65.4–523.3 Hz | 37    |
| Female | C3 – C5       | 48 – 72   | 130.8–523.3 Hz | 25    |

## Format

- Mono WAV, 44100 Hz, 16-bit
- ~2.5 seconds sustained "ah" vowel
- Gentle attack (50 ms) and release (150 ms)

## Source

All samples are **synthetically generated** using formant synthesis (sawtooth + triangle oscillator shaped through three bandpass biquad filters at "ah" vowel formant frequencies). They are original works by the FerroTone project.

- Male formants: ~730 Hz, ~1090 Hz, ~2440 Hz
- Female formants: ~854 Hz, ~1275 Hz, ~2855 Hz (scaled 1.17×)

## License

All files in this directory are original works released under the same license as the FerroTone project (MIT / Apache-2.0). No attribution is required but appreciated.

## Generator

The `crates/sample-gen/` binary regenerates all WAV files. Run from the workspace root:

```
cargo run -p sample-gen
```

## Metadata

`tone_map.json` provides machine-readable metadata for every sample:

```json
{
  "male": {
    "C4": { "file": "male/C4.wav", "hz": 261.63, "midi": 60, "source": "synthetic" },
    ...
  },
  "female": {
    "C4": { "file": "female/C4.wav", "hz": 261.63, "midi": 60, "source": "synthetic" },
    ...
  }
}
```
