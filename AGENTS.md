# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2024 terminal music player. The entry point is `src/main.rs`. Application state, events, and interactive components live under `src/app/`; audio decoding and playback are in `src/audio/`; Ratatui rendering, drawers, and theme definitions are in `src/ui/`; and filesystem or input helpers belong in `src/utils/`. Static project media is stored in `assets/images/`. Keep new modules close to the layer they extend and expose them through that directory's `mod.rs`.

## Build, Test, and Development Commands

- `cargo run` builds a debug binary and launches the player in the current directory.
- `cargo build --release` creates the optimized binary at `target/release/tui-music-player`; use this when checking playback performance.
- `cargo check` performs a fast type and compile check without producing a runnable binary.
- `cargo test` runs all unit and integration tests.
- `cargo fmt --all -- --check` verifies standard Rust formatting.
- `cargo clippy --all-targets --all-features -- -D warnings` catches common Rust mistakes and treats warnings as failures.

The UI uses Nerd Font glyphs, so use a terminal configured with a Nerd Font when manually testing.

## Coding Style & Naming Conventions

Use `rustfmt` defaults (four-space indentation) and keep code Clippy-clean. Follow Rust conventions: `snake_case` for functions, variables, files, and modules; `PascalCase` for structs, enums, and traits; and `SCREAMING_SNAKE_CASE` for constants. Keep UI drawing separate from application state transitions and audio processing. Prefer explicit error propagation over new `unwrap()` calls in filesystem and playback paths.

## Testing Guidelines

The repository currently has no committed automated tests. Add focused `#[cfg(test)]` unit modules beside the code under test; place cross-module scenarios in `tests/` with descriptive names such as `playlist_filter.rs`. Before submitting, run `cargo test`, `cargo fmt --all -- --check`, and Clippy. Manually verify affected keyboard controls and audio formats documented in `README.md` (MP3, MP4/AAC, WAV, and FLAC where applicable).

## Commit & Pull Request Guidelines

Recent history uses short, imperative Conventional Commit-style subjects such as `feat: implement filter playlist`, `bug: update ...`, and `doc: prepare ...`. Continue that pattern, keeping each commit focused. Pull requests should explain the user-visible change, list validation commands, and link relevant issues. Include a terminal screenshot or recording for UI changes, and call out platform or audio-device assumptions for playback changes.
