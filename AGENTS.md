# FerroTone — AGENTS.md

## Project

Tauri v2 desktop app for real-time vocal pitch training. Currently at starter-template stage — the `doc/` directory contains the planned architecture but nothing beyond the scaffold is implemented yet.

## Stack

- **Frontend**: React 19, TypeScript 5.8, Vite 7 (`src/`)
- **Backend**: Tauri v2, Rust (edition 2021) (`src-tauri/`)
- **Planned**: `crates/ferrotone-core/` (pure Rust crate, no Tauri dep) — not yet created

## Commands

| Command | Purpose |
|---------|---------|
| `npm run tauri dev` | Dev mode — Vite on port 1420 + Tauri window |
| `npm run dev` | Frontend-only Vite dev server (no Tauri) |
| `npm run build` | `tsc && vite build` (frontend only) |
| `npm run tauri build` | Production binary |
| `cargo test --workspace` | All Rust tests (core + shell) |
| `npx vitest run` | Frontend tests |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Rust lint |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Rust formatting |

No frontend lint config exists yet (no eslint/prettier). `npm run lint` is not wired.

## Architecture (planned)

Three-tier design documented in `doc/PHASE1.md`:

```
src/                          # React/TS frontend
src-tauri/                    # Tauri shell (thin IPC layer)
crates/ferrotone-core/        # Pure Rust crate (not yet created)
  ├── audio/                  #   cpal capture engine
  ├── pitch/                  #   PitchDetector trait, SWIPE' impl, DummyDetector
  └── music/                  #   hz_to_midi, midi_to_note_name, cents_off
```

Key design rule: `ferrotone-core` has zero Tauri dependency — testable in isolation, reusable from WASM. The `src-tauri` crate is a thin shell that wires core to UI via IPC.

## IPC pattern

- Frontend calls Rust via `invoke('command_name', { args })` from `@tauri-apps/api/core`
- Rust emits real-time events (e.g. `"pitch-frame"`) via `app_handle.emit()`, frontend listens with `listen()` from `@tauri-apps/api/event`
- All audio/DSP lives in `ferrotone-core`; `src-tauri` only routes commands and events

## Testing

- Rust: `cargo test --workspace` (unit tests in `ferrotone-core`, integration tests in `src-tauri/tests/` using Tauri's `MockRuntime`)
- Frontend: `npx vitest run` (Vitest with `@tauri-apps/api/mocks`)
- Integration tests use `DummyDetector` (no real mic needed)

## Key conventions

- No `unwrap()`/`expect()` in production Rust code — return `Result<T, String>` for Tauri commands
- Tauri managed state via `tauri::State<'_, AppState>` with `Mutex<Option<CaptureEngine>>`
- Capability permissions in `src-tauri/capabilities/default.json`
- Vite dev server on port 1420 (strict), ignores `src-tauri/` changes
- `src-tauri/src/main.rs` has `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` — do not remove
- Every feature added must be reflected with a single short line in `README.md` under the relevant section
- Classes (and Rust structs/traits) should be small and own a single responsibility — follow the Single Responsibility Principle. When planning code changes, first consider whether an existing type has grown too broad; if so, split it before adding new functionality.
