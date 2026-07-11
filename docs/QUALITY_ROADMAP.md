# BDL quality roadmap

BDL is moving from an early product implementation toward a maintainable open-source desktop application. This roadmap keeps behavior changes, structural refactors, and visual changes independently reviewable.

## Quality baseline

| Area | Current score | Evidence | Target |
| --- | ---: | --- | ---: |
| Accessibility | 3 / 4 | Shared controls expose labels and focus states; end-to-end keyboard and automated accessibility coverage are still missing. | 4 / 4 |
| Performance | 3 / 4 | Parsing is paged and media hydration is deferred; large lists still need measured rendering thresholds. | 4 / 4 |
| Theming | 3 / 4 | Tokenized light/dark themes exist; a few state treatments remain local to pages. | 4 / 4 |
| Responsive stability | 2 / 4 | Desktop widths are handled, but narrow-window and text-scaling contracts are not tested. | 3 / 4 |
| UI consistency | 3 / 4 | Shared primitives and workspace patterns exist; settings and some dense toolbars still diverge. | 4 / 4 |

Baseline: **14 / 20 — good, with structural risks that should be addressed before feature growth.**

## P1 — automated quality gates

- Keep Rust formatting, Clippy, tests, Vue type checking, style lint, and frontend tests mandatory in CI.
- Run Vue and TypeScript semantic lint with zero warnings.
- Add component-level accessibility tests for dialogs, menus, tabs, pagination, selection, and keyboard focus.
- Add a small set of deterministic screenshots for light/dark and narrow/standard desktop widths.

Exit condition: a pull request cannot merge with a type, semantic lint, style, unit-test, or stable visual-state regression.

## P1 — deepen backend modules

`crates/bdl-tauri/src/state.rs` and `commands.rs` are the main growth risks. Their public Tauri command interface should remain stable while implementations move behind deeper modules.

Progress:

- [x] Extract deterministic source paging, source identity parsing, page merging, and initial expansion rules into `parse_session`.
- [x] Move selection normalization and on-demand part hydration into `parse_session`.
- [x] Extract duplicate handling, startup recovery, and guarded queue transitions into `QueueCoordinator`.
- [x] Move diagnostic credential, URL, header, task, and log sanitization behind a tested export module.
- [x] Extract media attachment selection, completed-resource lookup, and NFO generation from the command worker.
- [ ] Reduce the remaining `commands.rs` worker/media processing to command adapters and dedicated modules.

Planned seams:

1. `ParseSession` — source lifecycle, paging, hydration, and normalized selection.
2. `QueueCoordinator` — scheduling, pause/resume/cancel, worker notification, and task-state transitions.
3. `AccountSession` — login snapshot, secure cookie storage, and account-library access.
4. `TaskMaintenance` — retry, media URL refresh, cleanup, and diagnostic export.

The Tauri commands remain thin adapters at these seams. Each module owns its invariants and tests; no additional pass-through layer should be introduced.

Exit condition: command adapters contain validation and translation only, and each state transition is testable through one module interface.

## P2 — frontend state and page structure

- [x] Centralize settings navigation and select-option catalogs with uniqueness tests.
- [x] Move settings bindings, validation, derived descriptions, initialization, save, reset, and restore behavior into `useSettingsForm` while the settings store remains the source of truth.
- [x] Extract download and media settings sections behind the shared `SettingsForm` interface.
- [x] Split naming, processing, archive, and maintenance sections without duplicating store ownership.
- Separate transfer query/filter state from task commands in `transferView.ts`.
- [x] Extract stable transfer sorting, search normalization, and bulk-action eligibility into tested pure queries.
- [x] Extract scheduling and per-task speed-limit dialog state behind an injected queue interface.
- [x] Extract context-menu positioning, lifecycle actions, task action composition, and dispatch behavior.
- [x] Extract bulk-action scope, status eligibility, and queue dispatch behind a narrow queue interface.
- [x] Extract task-detail selection, progress, logs, loading state, title, and refresh behavior.
- [x] Extract exhaustive single-task command dispatch and local configuration actions behind a narrow queue interface.
- [x] Centralize transfer-page refresh and window-listener ownership; keep global queue listeners at the app boundary.
- [x] Keep parse orchestration in the parse store while moving tree, filtering, and batch-selection transformations into tested pure modules.
- Define one page shell, one filter toolbar, one selection action bar, and one empty/error pattern for all five pages.

Exit condition: pages compose domain modules and shared UI primitives instead of owning repeated interaction behavior or large style blocks.

## P2 — list stability and performance

- Establish virtualization thresholds for parsed content and transfer history.
- Preserve selection, scroll position, and filters across route changes and paging.
- Lazy-load remote images and use fixed aspect-ratio placeholders to prevent layout shifts.
- Measure queue event frequency and coalesce presentation-only updates without delaying task commands.

Exit condition: 10,000-item sources and long-running queues remain responsive and memory use is bounded.

## P2 — contributor experience

- Document module ownership and invariants beside each major crate/module.
- Add issue and pull-request templates with reproduction, screenshots, and verification fields.
- Publish a supported-platform test matrix and release checklist.
- Keep generated frontend declarations out of semantic lint and review noise.

## Change discipline

- One concern per commit: behavior, refactor, visual polish, or tooling.
- Add characterization tests before moving stateful code.
- Prefer a deep module over a new shallow wrapper.
- Extend existing design tokens and primitives before adding page-local CSS.
- Every user-visible state must define default, hover, focus, disabled, loading, empty, error, and dark-theme behavior where applicable.
