# Briefing — bdl

## Conventions

<!-- Project house rules + AI-maintenance preferences, in plain prose.
     e.g. "publishing surface is English", "ask before force-pushing". -->

- Durable project design lives in `flightdeck/knowledge/`, not root `docs/`.
- Active implementation handoff lives under `flightdeck/work/<topic>/index.md`.
- Completed implementation handoff packages live under `flightdeck/archive/<topic>/index.md`.
- Keep downloader business logic in Rust core crates; Tauri commands should remain glue.

## Subscriptions

<!-- one ~/.flightdeck-relative path per line; empty = subscribe to nothing global -->
