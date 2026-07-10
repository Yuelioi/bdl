# Index — BDL product upgrade

## State

Stages 1–3 are complete: the product foundation, core-workflow UI uplift, and open-source release surface are implemented and verified. Stage 4 functional-depth slices remain available as follow-up work.

## Next

Start Stage 4 with backend duplicate-task detection and its skip/create/ask UI, then add environment health checks as a separate vertical slice.

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

Current:
- Stage 4 is not started; the recommended first slice is duplicate-task policy.

Verified:
- Git was clean on `main` at preflight.
- `pnpm run build` passes before implementation; the initial JS chunk is about 590 KB and triggers Vite's chunk-size warning.
- `pnpm run build` passes after implementation on Vite 8.1.4; page chunks are emitted and no chunk-size warning remains.
- `./scripts/check.ps1` passes: Rustfmt, strict workspace Clippy, 151 Rust tests, frontend type/build, and whitespace checks.
- `pnpm peers check` reports no peer dependency issues.

## Open questions

- None. The workspace already declared MIT, so the repository license file now matches that metadata.
