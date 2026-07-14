# Configuration

FerroTone uses a TOML configuration file for all user and application settings.

## File Location

The config file is resolved at startup in this order:

1. **Current working directory** — `config.toml` is always checked first
2. **Debug mode** (`cargo run`, `npm run tauri dev`) — always uses CWD; creates the file there if missing
3. **Release mode** — falls back to the platform config directory if no file exists in CWD:
   - Windows: `%APPDATA%/ferrotone/config.toml`
   - macOS: `~/Library/Application Support/ferrotone/config.toml`
   - Linux: `~/.config/ferrotone/config.toml`

On first run the app creates a default `config.toml` at the resolved location.

## Default Contents

```toml
[audio]
sample_rate = 48000
buffer_size = 1024
device_name = ""
algorithm = "swipe"

[noise_cancellation]
enabled = false
input_gain = 1.0
rms_gate_enabled = true
rms_threshold = 0.01
confidence_gate_enabled = true
confidence_threshold = 0.3
bandpass_enabled = true
bandpass_low = 80.0
bandpass_high = 1600.0

[user]
cache_folder = ""
active_profile = "default"
```

## Settings Reference

### `[audio]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `sample_rate` | integer | `48000` | Audio capture sample rate in Hz |
| `buffer_size` | integer | `1024` | Samples per DSP processing chunk |
| `device_name` | string | `""` | Specific input device name; empty = system default |
| `algorithm` | string | `"swipe"` | Pitch detection algorithm (`swipe` or `dummy`; future: `yin`, `pyin`) |

### `[noise_cancellation]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `false` | Master noise cancellation toggle |
| `input_gain` | float | `1.0` | Input gain multiplier (0.0–2.0) |
| `rms_gate_enabled` | bool | `true` | Enable RMS-based silence gate |
| `rms_threshold` | float | `0.01` | RMS threshold below which input is silenced |
| `confidence_gate_enabled` | bool | `true` | Enable confidence-based pitch frame filter |
| `confidence_threshold` | float | `0.3` | Minimum clarity score (0.0–1.0) to pass a pitch frame |
| `bandpass_enabled` | bool | `true` | Enable bandpass pre-filtering |
| `bandpass_low` | float | `80.0` | Bandpass low cutoff (Hz) |
| `bandpass_high` | float | `1600.0` | Bandpass high cutoff (Hz) |

### `[user]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `cache_folder` | string | `""` | Cache directory; empty = platform default |
| `active_profile` | string | `"default"` | Currently active player profile |

## Architecture

The configuration module lives in `ferrotone-core` (`crates/ferrotone-core/src/config.rs`) with zero Tauri dependency. It can be unit-tested and reused independently.

- `Settings::load()` reads the file on startup, creating defaults if absent
- `Settings::save()` writes the current settings back to disk
- The `serde(default)` attribute on each section ensures missing keys in an old config file do not cause errors
- File handles are closed immediately after read/write (`std::fs::read_to_string` / `std::fs::write`)
