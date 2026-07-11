# AGENTS.md - Tauri & Rust App Guidelines

## Project Overview
This is a cross-platform desktop application built with the **Tauri v2** framework, leveraging a native **Rust** backend core and a high-performance **TypeScript / React** frontend.

## Development Commands
### Package Manager Focus
Always use `npm` for frontend management (or swap to `pnpm`/`bun` if explicit in package.json).

### Core Commands
* **Dev Environment**: `npm run tauri dev` (Runs Vite + Tauri development window)
* **Production Build**: `npm run tauri build` (Compiles release binary)
* **Frontend Only Dev**: `npm run dev`
* **Rust Linting**: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
* **Rust Formatting**: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
* **Frontend Linting**: `npm run lint`

## Architecture & Communication Flow
The project relies on a dual-layer, message-passing desktop architecture:
1. `src/` - Frontend layer. Uses React, Vite, and TypeScript.
2. `src-tauri/` - Backend layer. Native Rust runtime with system-level access.

### IPC (Inter-Process Communication) Rules
* **Commands**: Frontend initiates actions via `invoke('command_name', { args })`.
* **Events**: Backend pushes real-time updates using Tauri's event emitter emitting payload streams (`emit` / `listen`).
* **Isolation**: Never perform raw FFI, direct database mutations, or unsafe disk access in the frontend layer. Always route these through registered Tauri Rust commands.

## Code Style & Conventions

### Rust Backend (`src-tauri/`)
* **Error Handling**: Never use `unwrap()` or `expect()` in production paths. Always return a structured `Result<T, E>`.
* **Tauri Command Errors**: Return a `Result<T, String>` or a serialized custom error enum so Tauri can pass it to JavaScript's `catch` block.
* **State Management**: Persist global states using Tauri’s managed state system (`tauri::State<'_, MyState>`). Inject states directly into command signatures.
* **Imports & Modularity**: Group commands in a `commands/` or `services/` module. Keep `lib.rs` and `main.rs` strictly clean for setup, plugin registries, and invoke handlers.

### TypeScript / React Frontend (`src/`)
* **Strict Typing**: Maintain strict TypeScript patterns. Avoid using `any`. Write explicit interfaces for all incoming Tauri command responses.
* **Tauri Environment Safety**: Guard Tauri-specific APIs or checks with platform detection (`window.__TAURI__`) if fallback web previews are required.
* **Asynchronous Calls**: Wrap IPC `invoke` calls inside React hooks, `async/await` blocks, or data-fetching libraries like TanStack Query.

## Critical Security Boundaries
* **Capability Configurations**: Define individual plugin permissions explicitly within `src-tauri/capabilities/`.
* **Path Handlers**: Always use Tauri's scoped path resolvers (e.g., `$APPDATA`, `$DOCUMENT`) to access system file paths instead of raw hardcoded strings.
