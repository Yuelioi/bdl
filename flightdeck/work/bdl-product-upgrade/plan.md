# BDL Product Upgrade — Execution Plan

## Stage 1 — Product foundation

- Replace ad hoc color/spacing values with the Quiet Control Room token system.
- Bundle the selected display and Chinese UI fonts for offline use.
- Redesign the app shell with icon navigation, live queue health, adaptive compact mode, and keyboard shortcuts.
- Add route-level lazy loading/manual chunks so the initial bundle no longer ships as one monolith.
- Add reduced-motion and strong focus-visible behavior.

Verification: frontend type check/build; no large-entry-chunk warning; shell remains usable at 720–1440 px widths.

## Stage 2 — Core workflow uplift

- Parse: strengthen the input command surface, teach supported inputs in the empty state, clarify source/history state, and simplify the result action hierarchy.
- Transfer: add a compact queue health strip, preserve scan-stable columns, improve empty states, and add row context-menu parity using existing state-aware actions.
- Settings: add section navigation, visible dirty/saved feedback, and clearer advanced-group boundaries.

Verification: all existing commands and store contracts remain unchanged; keyboard access and touch-sized narrow-window controls are present.

## Stage 3 — Open-source release surface

- Add a truthful bilingual-ready README structure with feature map, architecture, development, verification, security, and roadmap sections.
- Add CONTRIBUTING and SECURITY policies plus a CI workflow for Rust checks and frontend build.
- Do not add a license until the maintainer selects one.

Verification: documented commands match repository scripts and workspace paths.

## Stage 4 — Functional depth after the UI foundation

### Stage 4.1 — Transfer reliability (complete)

- Make pause wake stalled metadata, request, and response-stream waits immediately.
- Treat successful muxing as a commit point that cannot be overwritten by pause/cancel races.
- Recover legacy tasks when a non-empty final output exists and cleaned raw media inputs are gone.
- Replace adjacent-event speed estimates with a stable rolling byte window.
- Coalesce chunk-level progress into a bounded UI event cadence while preserving the final update.
- Use four resumable range segments by default so CDN single-connection throttling does not cap throughput.
- Keep pause/cancel actions unavailable during the non-interruptible mux commit phase.

Verification: deterministic stalled-request cancellation test; task-state transition regression; fetcher resume suite; Tauri state tests; frontend type check/build; full repository check.

### Stage 4.2 — Product depth

- Duplicate-task policy (complete): backend `ask`/`skip`/`create`, atomic confirmation, safe copy IDs, queue-path reservation, and a status-aware confirmation dialog.
- Environment health (complete): writable-directory probe, real FFmpeg version probe, shared Settings/download-dialog health panel, inline directory creation, and persisted FFmpeg repair routing.
- Frontend component/store tests and keyboard-navigation tests (complete): Vitest repository gate, environment health/repair regressions, and roving-focus parsed-tree navigation.
- Evaluate scheduler, speed limits, updater, dark theme, and account asset entry points as separate vertical slices.

Verification: narrow Rust tests per feature, full core/Tauri clippy, frontend tests/build, and release QA.
