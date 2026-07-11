# FerroTone 🎸🎤

An open-source, high-performance cross-platform desktop application designed to help vocalists learn to sing, practice pitch accuracy, and train their ears using real-time audio visualization and AI-driven source separation.

Built with **Tauri**, **Rust**, **React/TypeScript**, and **Python**.

---

## 🚀 Overview

FerroTone is designed to bridge the gap between gamified singing experiences and professional vocal coaching. 
By utilizing a hybrid architecture, the application offloads heavy Digital Signal Processing (DSP) and native audio device integration
to a hyper-fast Rust core, while leveraging Python's robust machine learning ecosystem for isolating track components.

### Core Features

*   **Native System Audio Loopback:** Capture music streams globally (Spotify, YouTube) straight from your system speakers via Windows WASAPI Loopback / macOS ScreenCaptureKit with zero virtual routing setups.
*   **AI Source Separation:** Automatically break down any ingested audio file into a pure isolated `vocals.wav` stem and an instrumental `accompaniment.wav` stem.
*   **Visual Pitch Analytics:** A fluid, 60 FPS interactive HTML5 Canvas scroll grid mapping your live singing frequency directly against the perfect extracted target pitch timeline.
*   **Intelligent Pitch Detection:** Utilizes native implementations of the YIN/McLeod Pitch Method (MPM) algorithms to extract raw human vocal parameters with near-zero latency.
*   **Interactive Ear Training:** Gamified training screens focusing on scale intervals, hitting precise note progressions, and interval retention.

---

## 🏗️ Architecture

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

# Tech Stack & Open Source Libraries

Frontend Framework: React 18+ with TypeScript

Build Tool: Vite

Rendering: Native HTML5 Context2D Canvas for low-overhead audio wave/pitch rendering.

Desktop App Shell & Backend Core Framework: Tauri v2

Core Language: Rust

Audio I/O Handling: cpal (Cross-Platform Audio Library)

Audio Playback: rodio

DSP Engine: pitch-detection (Native Rust crate)

Machine Learning Engine (Sidecar)

Core Language: Python 3.10+

Vocal Splitting: deezer/spleeter or facebookresearch/demucs

Analysis Library: librosa (for pre-calculating foundational $f_0$ frequency matrices)


# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
