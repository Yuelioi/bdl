# Index — BDL product upgrade

## State

The project-wide product and UI upgrade is active. Existing downloader flows work, but the current feature depth, interaction quality, visual language, and open-source presentation need a full audit before implementation.

## Next

Audit the current Rust/Tauri/Vue product, research mature desktop task-manager patterns, and write the staged upgrade plan before changing production UI.

## Read now

- ../../../knowledge/bdl-downloader/architecture.md
- ../../../knowledge/bdl-downloader/product-flow.md
- legacy-ui-followups.md

## Read if

- design.md — after the audit settles the new product and visual direction
- plan.md — once implementation begins

## Progress

Done:
- Loaded the existing design context and durable BDL architecture/product-flow constraints.
- Migrated the old standalone UI follow-up backlog into this topic package.

Current:
- Repository, product-flow, UI-system, and feature-gap audit.

Verified:
- Git was clean on `main` at preflight.

## Open questions

- Which improvements deliver the strongest first release without destabilizing the downloader core?

