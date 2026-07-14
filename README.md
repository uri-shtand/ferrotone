# FerroTone

An open-source, high-performance cross-platform desktop application designed to help vocalists learn to sing, practice pitch accuracy, and train their ears using real-time audio visualization and AI-driven source separation.

Built with **Tauri**, **Rust**, **React/TypeScript**, and **Python**.

---

##  Overview

FerroTone is designed to bridge the gap between gamified singing experiences and professional vocal coaching. 
By utilizing a hybrid architecture, the application offloads heavy Digital Signal Processing (DSP) and native audio device integration
to a hyper-fast Rust core, while leveraging Python's robust machine learning ecosystem for isolating track components.

### Core Features

*   **Native System Audio Loopback:** Capture music streams globally (Spotify, YouTube) straight from your system speakers via Windows WASAPI Loopback / macOS ScreenCaptureKit with zero virtual routing setups.
*   **AI Source Separation:** Automatically break down any ingested audio file into a pure isolated `vocals.wav` stem and an instrumental `accompaniment.wav` stem.
*   **Visual Pitch Analytics:** A fluid, 60 FPS interactive HTML5 Canvas scroll grid mapping your live singing frequency directly against the perfect extracted target pitch timeline.
*   **Intelligent Pitch Detection:** Utilizes native implementations of the YIN/McLeod Pitch Method (MPM) algorithms to extract raw human vocal parameters with near-zero latency.
*   **Noise Gating:** RMS volume gate and confidence score gate filter out background noise and low-confidence pitch frames in real-time.
*   **Volume Graph:** Real-time waveform showing voice loudness over the last 60 seconds (dB scale).
*   **Pitch Graph:** Real-time pitch trajectory over the last 60 seconds (MIDI note scale, cents-based coloring).
*   **RNNoise Suppression:** Neural-network-based real-time noise reduction via nnnoiseless.
*   **Bandpass Pre-Filtering:** Second-order IIR bandpass filter restricts processing to the vocal frequency range (80–1000 Hz default).
*   **Recording Controls Widget:** Live-adjustable panel for input gain, volume gate, confidence gate, bandpass filter, and device selection with one-click save to disk.
*   **Interactive Ear Training:** Gamified training screens focusing on scale intervals, hitting precise note progressions, and interval retention.
*   **Vocal Sample Library:** 62 single-note sustained "ah" vowel samples (C2–C5 male + C3–C5 female) for testing and playback, generated via formant synthesis in `samples/`.

---

##  Architecture

FerroTone utilizes a decoupled, three-tier architecture to maximize computational efficiency while keeping the binary payload small:

```text
┌────────────────────────────────────────────────────────┐
│                   FRONTEND (UI LAYER)                  │
│                React / TypeScript / Vite               │
│  • 60 FPS Scrolling Visual Target Grid (Canvas)        │
│  • Interactive Training Modules & Dashboard            │
└───────────────────────────┬────────────────────────────┘
                            │ Tauri IPC (Commands/Events)
┌───────────────────────────▼────────────────────────────┐
│                    TAURI CORE (RUST)                   │
│  • Native Mic Capture Engine (via `cpal`)              │
│  • Real-Time Pitch Analysis DSP (YIN/MPM algorithms)    │
│  • Asynchronous Thread Management & Sidecar Controller │
└───────────────────────────┬────────────────────────────┘
                            │ Local Localhost Socket / IPC
┌───────────────────────────▼────────────────────────────┐
│                  AI ENGINE (PYTHON SIDECAR)            │
│  • Audio Demixing Models (Spleeter / Demucs)           │
│  • Static Track Target Pitch Profile Analysis          │
└────────────────────────────────────────────────────────┘

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19, TypeScript 5.8, Vite 7 |
| Desktop Shell | Tauri v2 |
| Core (DSP/Audio) | Rust (edition 2021) with `cpal`, `rodio`, `pitch-detection` |
| AI Sidecar (planned) | Python 3.10+ with Spleeter/Demucs, librosa |
| Rendering | Native HTML5 Canvas (Context2D) — 60 FPS pitch grid |

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) 1.85+ (nightly)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (platform-specific)

### Install dependencies

```sh
npm install
```

### Run in development mode

Launches Vite dev server on `localhost:1420` + Tauri desktop window with hot-reload:

```sh
npm run tauri dev
```

To work on the frontend only (no Tauri window):

```sh
npm run dev
```

### Build for production

Frontend-only build (`tsc && vite build`):

```sh
npm run build
```

Full desktop binary:

```sh
npm run tauri build
```

### Run tests

Frontend (Vitest):

```sh
npm test
```

All Rust crates (`ferrotone-core` + Tauri shell):

```sh
cargo test --workspace
```

### Lint & format

```sh
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

---

## Project Structure

```
ferrotone/
├── src/                     # React / TypeScript frontend
├── src-tauri/               # Tauri shell (thin IPC layer)
│   ├── src/                 #   Rust backend (commands, events)
│   ├── tests/               #   Integration tests (MockRuntime)
│   └── capabilities/        #   Tauri capability permissions
├── crates/
│   └── ferrotone-core/      # Pure Rust crate (no Tauri dep)
│       ├── src/
│       │   ├── audio/       #   cpal capture engine
│       │   ├── pitch/       #   YIN/MPM/SWIPE' detectors
│       │   └── music/       #   hz_to_midi, note naming, cents
│       └── tests/           #   Unit tests
├── samples/                 # Vocal sample library (62 WAV files + tone_map.json)
├── doc/                     # Architecture & design docs
├── package.json
└── README.md
```

---

## Architecture

FerroTone uses a decoupled three-tier design:

```text
┌────────────────────────────────────────────────────────┐
│                   FRONTEND (UI LAYER)                  │
│                React / TypeScript / Vite               │
│  • 60 FPS Scrolling Visual Target Grid (Canvas)        │
│  • Interactive Training Modules & Dashboard            │
└───────────────────────────┬────────────────────────────┘
                            │ Tauri IPC (Commands/Events)
┌───────────────────────────▼────────────────────────────┐
│                    TAURI CORE (RUST)                   │
│  • Native Mic Capture Engine (via `cpal`)              │
│  • Real-Time Pitch Analysis DSP (YIN/MPM algorithms)    │
│  • Asynchronous Thread Management & Sidecar Controller │
└───────────────────────────┬────────────────────────────┘
                            │ Local Localhost Socket / IPC
┌───────────────────────────▼────────────────────────────┐
│                  AI ENGINE (PYTHON SIDECAR)            │
│  • Audio Demixing Models (Spleeter / Demucs)           │
│  • Static Track Target Pitch Profile Analysis          │
└────────────────────────────────────────────────────────┘
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
