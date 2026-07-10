# Index — BDL product upgrade

## State

Stages 1–3, Stage 4.1 transfer reliability, duplicate-task policy, environment health, and the first frontend test/keyboard-navigation slice are complete.

## Next

Have the maintainer exercise pause/resume, duplicate confirmation, and environment repair on real downloads. Then evaluate scheduler controls, speed limits, updater, dark theme, and account asset entry points as separate vertical slices.

## Read now

- ../../knowledge/bdl-downloader/architecture.md
- ../../knowledge/bdl-downloader/product-flow.md
- ../../knowledge/bdl-downloader/design-system.md
- design.md
- plan.md
- legacy-ui-followups.md

## Read if

- legacy-ui-followups.md — if a staged change intersects an older backlog item
- ../../knowledge/frontend/typescript-7-vue-tsc.md — if changing TypeScript/vue-tsc or the frontend build fails before type checking
- ../../knowledge/frontend/nuxt-ui-select-portal.md — if a select popup shifts or clips form content
- ../../knowledge/frontend/dropdown-dismissal.md — before adding or changing any transient popup menu

## Progress

Done:
- Loaded the existing design context and durable BDL architecture/product-flow constraints.
- Migrated the old standalone UI follow-up backlog into this topic package.
- Audited the Rust/Tauri/Vue product, frontend structure, repository surface, and baseline build.
- Researched official desktop navigation, data-table, accessibility, and reduced-motion guidance plus mature open-source download managers.
- Settled the Quiet Control Room design direction and four-stage execution plan.
- Added an OKLCH semantic token system and bundled Archivo + Noto Sans SC variable fonts for offline use.
- Rebuilt the app shell with adaptive expanded/compact navigation, queue health, page identity, reduced-motion support, and Ctrl+1/2/3 plus Ctrl+L shortcuts.
- Added the Parse command surface and instructional empty states.
- Added Transfer queue health, stable task operations, and state-aware right-click context-menu parity.
- Added Settings section navigation and persistent dirty-state feedback.
- Split pages into async chunks; the largest JS chunk is now about 269 KB instead of a 590 KB monolith.
- Kept the concurrent Pinia/Vite/plugin/vue-tsc upgrades and pinned TypeScript 5.9.3 to restore vue-tsc compatibility.
- Added README, CONTRIBUTING, SECURITY, MIT license, and a Windows CI workflow backed by the repository check script.
- Refactored the planner's internal output-path interface so strict Clippy passes without a lint suppression.
- Removed duplicate queue health surfaces; Transfer tabs now own counts and the sidebar footer owns aggregate speed.
- Unified field label/control geometry and icon-button centering.
- Re-enabled the Nuxt UI select portal so menus overlay without changing form layout.
- Reduced header, panel, decorative-gradient, shadow, and motion intensity after live visual review.
- Replaced the custom account/source popovers and native-details overflow menu with accessible Nuxt UI dropdowns.
- Made fetch cancellation wake stalled metadata, request, and response-stream waits immediately.
- Coalesced chunk progress to five UI updates per second and changed speed display to a five-second rolling byte window.
- Migrated the legacy single-segment default to four resumable range segments once, while preserving an explicit post-migration choice of one segment; partial segment files survive pause/resume.
- Made successful muxing an irreversible completion commit and prevented muxing/completed tasks from accepting late pause/cancel transitions.
- Added recovery for the exact `missing media input` case where the final MP4 exists and raw media inputs were already cleaned.
- Replaced silent duplicate-task skipping with an atomic ask/skip/create contract and status-aware confirmation dialog.
- Reserved active queue output paths and generated safe `:copy:<n>` task/resource identities for create-anyway.
- Added a shared environment health panel backed by writable-directory probes and a real `ffmpeg -version` check.
- Blocked task creation on unhealthy environments, with inline directory creation and Settings routing for persisted FFmpeg repair.

Current:
- Frontend component/store tests and parsed-tree keyboard navigation are complete; select the next product-depth slice from scheduler controls, speed limits, updater, dark theme, or account asset entry points.

Verified:
- Git was clean on `main` at preflight.
- `pnpm run build` passes before implementation; the initial JS chunk is about 590 KB and triggers Vite's chunk-size warning.
- `pnpm run build` passes after implementation on Vite 8.1.4; page chunks are emitted and no chunk-size warning remains.
- `./scripts/check.ps1` passes: Rustfmt, strict workspace Clippy, 151 Rust tests, frontend type/build, and whitespace checks.
- `pnpm peers check` reports no peer dependency issues.
- `pnpm run build` passes after the visual-review corrections.
- Captured and inspected live Tauri screenshots for Parse, Transfer, Settings, and an open audio-quality menu; the menu no longer moves following fields.
- Reproduced account and Transfer overflow dismissal in the live Tauri app; outside clicks now close both menus.
- Added deterministic regressions for cancellation of a stalled GET, resumable segment parts, terminal task transitions, and completed-output recovery.
- `./scripts/check.ps1` passes after the transfer reliability changes: Rustfmt, strict workspace Clippy, all Rust tests, frontend type/build, and whitespace checks.
- Restarted and inspected the live Tauri app after the backend changes; the existing 100%-progress failed task remains available for the maintainer to retry into the new completed-output recovery path.
- Added atomic AppState regressions for ask-without-partial-insertion and create-anyway path/ID reservation; the full repository check passes.
- Added FFmpeg probe and directory-state regressions; captured the live Settings health panel showing the writable project download directory and system FFmpeg 7.1.1.
- Review-hardened environment repair: non-FFmpeg executables are rejected, edited batch paths cannot reuse stale health, and older async checks cannot overwrite newer results.
- Added Vitest to the repository gate with Store race regressions, environment-health interaction/announcement coverage, and roving-focus tree keyboard tests.

## Open questions

- None. The workspace already declared MIT, so the repository license file now matches that metadata.
