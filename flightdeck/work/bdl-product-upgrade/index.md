# Index — BDL product upgrade

## State

The product foundation and core-workflow UI uplift are implemented and pass the frontend production build. Open-source release documentation and final workspace verification remain.

## Next

Implement Stage 3: README, contribution/security guidance, CI, then run final frontend and Rust verification.

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

Current:
- Stage 3 open-source release surface.

Verified:
- Git was clean on `main` at preflight.
- `pnpm run build` passes before implementation; the initial JS chunk is about 590 KB and triggers Vite's chunk-size warning.
- `pnpm run build` passes after implementation on Vite 8.1.4; page chunks are emitted and no chunk-size warning remains.

## Open questions

- Which license should the maintainer choose before public release?
