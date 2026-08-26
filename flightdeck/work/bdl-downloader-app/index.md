# BDL downloader app

## Goal

Build the original Rust/Tauri/Vue downloader foundation with normalized source resolution, persistent tasks, resilient downloads, media finalization, and a usable desktop interface.

## Status

Finished

## Current

The foundational downloader was completed and released before the later product-upgrade Work. Its durable architecture and product rules now live in project Knowledge.

## Next

None.

## Progress

- Implemented normalized resolvers for videos, uploaders, favorites, collections, series, bangumi, and courses.
- Added SQLite-backed task state, resumable fetching, queue coordination, FFmpeg media finalization, diagnostics, and secure account persistence.
- Delivered the Tauri command/event adapter and Vue workflows for parsing, selection, transfer management, settings, and account login.
- Added recovery, archive assets, pagination, URL refresh, and release QA foundations.
- Superseded the original UI and release backlog with the completed `bdl-product-upgrade` Work.

## References

- [Context](context.md)
- [Completion plan](plan.md)
- [Architecture knowledge](../../knowledge/bdl-downloader/architecture.md)
- [Product-flow knowledge](../../knowledge/bdl-downloader/product-flow.md)
