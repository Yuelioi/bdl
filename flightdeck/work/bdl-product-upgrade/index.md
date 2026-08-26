# BDL product upgrade

## Goal

Turn BDL into a polished, contributor-ready desktop downloader without weakening the Rust-owned download pipeline.

## Status

Finished

## Current

The product foundation, transfer reliability work, account-library entry point, signed updater, Windows packaging, and release surface are complete. `v0.2.0` was published on 2026-08-27 with signed updater metadata.

## Next

None.

## Progress

- Shipped the Quiet Control Room shell, semantic tokens, offline fonts, adaptive navigation, themes, keyboard behavior, and deterministic visual coverage.
- Deepened Parse, Transfer, Settings, scheduling, speed limits, duplicate handling, recovery, diagnostics, and account-library workflows.
- Moved metadata parsing and selected-part hydration to the correct seams for large sources and repeated downloads.
- Added repository documentation, security and contribution guidance, CI, signed updater automation, installer verification, and release operations.
- Incorporated real maintainer feedback for QR login, paid/legacy bangumi decoding, startup environment gating, and hidden Windows FFmpeg processes.
- Published [BDL v0.2.0](https://github.com/Yuelioi/bdl/releases/tag/v0.2.0); full CI and signed release workflows passed.

## References

- [Context](context.md)
- [Completion plan](plan.md)
- [Architecture knowledge](../../knowledge/bdl-downloader/architecture.md)
- [Product-flow knowledge](../../knowledge/bdl-downloader/product-flow.md)
- [Design-system knowledge](../../knowledge/bdl-downloader/design-system.md)
- [Release knowledge](../../knowledge/bdl-downloader/release-and-updates.md)
