# BDL product upgrade context

## Product intent

BDL is a Chinese-first, local-first desktop workspace for turning Bilibili sources into reliable download tasks. The primary job is to paste one or more sources, choose content confidently, monitor transfers, recover failures, and find completed output without reading raw logs.

## Stable constraints

- Rust owns source resolution, task state, downloading, persistence, media finalization, secure credentials, and OS integration.
- Tauri commands remain a narrow adapter; Vue owns presentation, selection, and interaction state.
- The desktop UI must remain usable offline, at narrow Windows sizes, with keyboard navigation and reduced motion.
- Signed updater artifacts use the embedded Tauri public key; Windows Authenticode remains separate and optional.
- User credentials stay in operating-system secure storage and never enter ordinary files, SQLite, or diagnostics.

## Delivered product shape

- Primary navigation is Parse, Library, Transfer, Settings, and About; account state remains global.
- Initial parsing is metadata-first; selected parts hydrate media streams only when tasks are created.
- Transfer operations preserve resumability, explicit recovery actions, bounded progress updates, and durable scheduling/settings.
- Environment capability is checked once at startup, cached as application state, and rechecked only after an explicit repair or settings save.
- FFmpeg remains the external media engine but is launched directly by Rust with hidden Windows child-process flags.

## Deferred scope

Watch later, history, and follows are separate future account-library work. They are not unfinished requirements of this completed Work.
