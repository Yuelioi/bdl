# Index — BDL product upgrade

## State

Audit and targeted external research are complete. The Quiet Control Room direction and staged execution plan are settled; Stage 1 implementation is next.

## Next

Implement Stage 1: product tokens, bundled typography, adaptive application shell, keyboard navigation, and bundle splitting.

## Read now

- ../../knowledge/bdl-downloader/architecture.md
- ../../knowledge/bdl-downloader/product-flow.md
- ../../knowledge/bdl-downloader/design-system.md
- design.md
- plan.md
- legacy-ui-followups.md

## Read if

- legacy-ui-followups.md — if a staged change intersects an older backlog item

## Progress

Done:
- Loaded the existing design context and durable BDL architecture/product-flow constraints.
- Migrated the old standalone UI follow-up backlog into this topic package.
- Audited the Rust/Tauri/Vue product, frontend structure, repository surface, and baseline build.
- Researched official desktop navigation, data-table, accessibility, and reduced-motion guidance plus mature open-source download managers.
- Settled the Quiet Control Room design direction and four-stage execution plan.

Current:
- Stage 1 product-foundation implementation.

Verified:
- Git was clean on `main` at preflight.
- `pnpm run build` passes before implementation; the initial JS chunk is about 590 KB and triggers Vite's chunk-size warning.

## Open questions

- Which license should the maintainer choose before public release?
