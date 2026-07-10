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

- Add backend duplicate-task detection with skip/create/ask policy.
- Add a download-directory/FFmpeg environment health command and inline repair actions.
- Add frontend component/store tests and keyboard-navigation tests.
- Evaluate scheduler, speed limits, updater, dark theme, and account asset entry points as separate vertical slices.

Verification: narrow Rust tests per feature, full core/Tauri clippy, frontend tests/build, and release QA.

