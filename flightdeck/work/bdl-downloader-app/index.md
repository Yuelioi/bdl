# Index - bdl-downloader-app

## State

Task 14 is code-complete on branch `bdl-downloader-app`. The workspace has a Rust/Tauri/Vue scaffold, frontend-safe normalized DTOs, a pure Bilibili input classifier, `bpi-rs` backed video/favorite/uploader/collection/series/bangumi/cheese resolvers, a planner that turns selected normalized parts into backend-owned download tasks/resources, a resumable `reqwest` fetcher, an ffmpeg muxer, a CLI parse/download path, registered Tauri commands/events, a custom Vue UI kit, parse/transfer pages, SQLite-backed queue persistence, and top-right account login through Cookie import or QR scan.

## Next

Continue Task 15 derived assets, naming, and history/settings cleanup. Transfer manual Tauri-window visual QA remains a product-polish follow-up if the Transfer page is revisited.

## Read now

- flightdeck/knowledge/bdl-downloader/product-flow.md
- flightdeck/knowledge/bdl-downloader/architecture.md
- flightdeck/work/bdl-downloader-app/transfer-product-refactor-plan.md
- flightdeck/work/bdl-downloader-app/plan.md

## Read if

- E:/projects/tools/bpi-rs/docs/api-index.md - when mapping a supported source type to existing `bpi-rs` APIs.


## Progress

Current:
- Created the initial flightdeck deck.
- Captured BDL product flow and architecture decisions.
- Moved durable design material into `flightdeck/knowledge/bdl-downloader/`.
- Wrote the phased implementation plan in `flightdeck/work/bdl-downloader-app/plan.md`.
- Completed Task 1 scaffold in commits `c3cac74` and `b482e22`.
- Task 1 verification passed: `cargo check --workspace`, `pnpm --dir apps/desktop install`, and `pnpm --dir apps/desktop build`.
- Code review required removing the absolute `bpi-rs` dependency from Task 1 and replacing the placeholder icon with a generated Tauri icon set; both are done.
- Completed Task 2 DTOs in commits `b5d10e5` and `347a1ba`.
- Task 2 verification passed: `cargo test -p bdl-core --test normalization`, `cargo fmt --all --check`, and `cargo check --workspace`.
- DTO quality review required tagged `StreamQuality`, transparent ID wrappers used in DTO fields, object-shaped `HeaderPair`, and full JSON shape tests; all are done.
- Non-blocking hardening noted for later: add deserialization/round-trip tests for transparent IDs and tagged `StreamQuality`.
- Completed Task 3 input classifier in commits `b1a076c` and `40ac013`.
- Task 3 verification passed: `cargo test -p bdl-core --test input_classifier`, `cargo fmt --all --check`, and `cargo check --workspace`.
- Classifier quality review required short URLs to remain `Unknown`, exact BVID parsing, AV URL parsing, host/path-aware URL handling, plain uploader space support, and broader edge tests; all are done.
- Non-blocking hardening noted for later: add more host spoofing regression tests such as `bilibili.com.evil.test` and `b23.tv.evil.test`.
- Completed Task 4 single-video resolver in commit `f2ab9cc`.
- Task 4 verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core`, and `cargo check --workspace`.
- The resolver uses portable `bpi-rs = 0.2.3` with only the `video` feature enabled, keeps `bpi-rs` usage inside `crates/bdl-core/src/resolver/video.rs`, and uses a fake `VideoApi` adapter for non-live tests.
- Execution note from the user: continue locally without subagents unless explicitly requested again.
- Fixed the Task 4 source/test mismatch in commit `8907a25`.
- Completed Task 5 planner and task model in commit `61bbb0b`.
- Task 5 verification passed: `cargo test -p bdl-core --test planner --test queue_state`, `cargo test -p bdl-core`, `cargo fmt --all --check`, and `cargo check --workspace`.
- Planner creates stable task/resource IDs, plans only selected loaded parts, chooses video/audio media streams in fast mode, adds cover/subtitle/danmaku/NFO intents for complete archive mode, and returns actionable errors when required streams are missing.
- Completed Task 6 resumable fetcher in commit `cb85078`.
- Task 6 verification passed: `cargo test -p bdl-core --test fetcher_resume`, `cargo test -p bdl-core`, `cargo fmt --all --check`, and `cargo check --workspace`.
- Fetcher writes to temp `.bdlpart` files first, stores progress in `<temp>.state`, resumes with `Range`, restarts when content length changes, forwards resource headers, retries failed GETs up to the configured limit, and emits progress through a channel.
- Completed Task 7 ffmpeg muxer and CLI vertical slice in commit `f6656cf`.
- Task 7 verification passed: `cargo test -p bdl-core --test muxer`, `cargo test --workspace`, `cargo check --workspace`, `cargo fmt --all --check`, `cargo run -p bdl-cli -- ffmpeg-check`, and `cargo run -p bdl-cli -- parse BV1xx411c7mD --json`.
- `ffmpeg-check` found `D:\scoop\shims\ffmpeg.exe` on this machine; CLI live parse returned normalized JSON for `BV1xx411c7mD`.
- CLI commands now include `parse`, `download`, `verify-cookie`, and `ffmpeg-check`; the initial `download` command supports the single-video pipeline using resolver, planner, fetcher, and muxer.
- Completed Task 8 Tauri command and event bridge in commit `8a33ac7`.
- Task 8 verification passed: `cargo check -p bdl-desktop`, `cargo test --workspace`, and `cargo fmt --all --check`.
- Registered canonical Tauri commands in `apps/desktop/src-tauri/src/main.rs`. `parse_create_source`, `parse_close_source`, `queue_list`, `settings_get`, `settings_update`, and `account_get` have real state-backed behavior; not-yet-implemented commands return typed `unsupported` errors.
- Defined canonical event names and emit paths for `parse://source-updated` and `settings://updated` in this phase.
- Completed Task 9 UI kit and app shell in commit `58078ea`.
- Task 9 verification passed: `pnpm --dir apps/desktop build`.
- The desktop shell now has custom shared controls, nav tabs `解析` / `传输` / `历史` / `设置`, a top-right login/account split button with logout dropdown, QR/Cookie login dialog surface, help drawer, toast host, source tree, transfer tabs, task row, and settings download controls.
- Completed Task 10 parse page in commit `45d4816`.
- Task 10 verification passed: `cargo check -p bdl-desktop`, `cargo test --workspace`, `cargo fmt --all --check`, and `pnpm --dir apps/desktop build`.
- The parse page now has frontend DTOs matching Rust serde output, typed Tauri wrappers, a Pinia parse store, source list, selectable normalized tree, `解析更多` / `解析全部` placeholders, and `下载已选择` creating backend queue tasks without auto-navigation. The toast action `查看传输` is user-triggered.
- `selection_create_tasks` is now implemented in the Tauri bridge for loaded parsed sources so the parse page can add tasks to the in-memory queue.
- Completed Task 11 transfer queue UI in commit `700b067`.
- Task 11 verification passed: `cargo check -p bdl-desktop`, `cargo test --workspace`, `cargo fmt --all --check`, and `pnpm --dir apps/desktop build`.
- The transfer page now has filters `正在下载`, `队列中`, `已暂停`, `失败`, `已完成`, and `全部`, a Pinia queue store listening for queue events, task detail/log panels, and task row actions for pause/resume/cancel/retry/remove/open-file/open-dir.
- Queue pause/resume/cancel/retry/remove now perform in-memory Tauri state transitions and emit `queue://task-updated`; file open commands still return typed `unsupported` until file actions are implemented.
- Completed Task 12 persistent storage and recovery in commits `36981b3`, `7649de6`, and `b434c16`.
- Task 12 verification passed: `cargo test --workspace`, `cargo check -p bdl-desktop`, `cargo fmt --all --check`, and `pnpm --dir apps/desktop build`.
- `bdl-core::storage::TaskStorage` now creates `tasks.sqlite` with `tasks`, `resources`, `segments`, `history`, and `task_logs`, saves task/resource snapshots, and reloads them after reopening.
- `AppState` now loads persisted queue tasks from `.bdl/tasks.sqlite` on startup and persists queue snapshots after create/pause/resume/cancel/retry/remove. It does not auto-resume tasks.
- Settings snapshot moved into `bdl-core::settings::AppSettings`; Tauri keeps `SettingsSnapshot` as an alias.
- Completed Task 13 account persistence and login in commits `1b3f3a4`, `01c6e50`, `31427dc`, and `93884dc`.
- Task 13 verification passed: `cargo test -p bdl-core --test account`, `cargo test -p bdl-tauri secure_store`, `cargo check -p bdl-desktop`, `cargo fmt --all --check`, and `pnpm --dir apps/desktop build`.
- `bdl-core::account` now validates imported cookies, redacts sensitive cookie/signed URL fields, maps QR login statuses, generates QR SVGs, and polls `bpi-rs` QR login.
- `AppState` now restores a persisted account cookie from `.bdl/account.cookie`, exposes account import/logout/verify, and builds video resolvers with the current cookie so parsing can return logged-in streams.
- The account UI now uses `apps/desktop/src/stores/account.ts`, supports Cookie import, QR generation/polling, account event updates, and top-right logout. The Settings tab remains account-free.
- Current account persistence is file-backed through `secure_store.rs`; replacing it with an OS credential-store backend remains a hardening follow-up.
- Task 14 page-size policy landed in commit `4a88d3e`.
- Task 14 favorite and uploader video parsing landed in commits `f2310f4`, `df916c0`, `751fb73`, `57e981a`, and `8ad66c2`.
- Uploader source parsing now loads the first page through `bpi-rs user.uploaded_videos`, `parse_load_more` appends one page, and `parse_load_all` batches pages up to the current explicit/default limit of 100.
- Favorite source parsing now loads video resources from `bpi-rs fav.list_detail` for links with `fid` or `media_id`, appends pages through the same `parse_load_more` path, and uses API `has_more` to avoid count issues when non-video resources are filtered out.
- `下载已选择` now hydrates selected uploader placeholder parts through the video resolver before planning tasks, so multi-P videos selected from an uploader list expand to their real parts. List sources no longer default to all selected in the parse UI.
- Transfer page product grill produced `flightdeck/work/bdl-downloader-app/transfer-product-refactor-plan.md`. Key outcomes: batch task management comes first, the list should become a quasi-table, filters should be `活动` / `失败` / `已完成` / `全部`, completed records stay in Transfer instead of a separate History page, failure diagnosis and recovery actions must be explicit, queue ordering controls are intentionally out of scope, and settings must only expose backend-honored behavior.
- Transfer Phase 1 implementation is in progress: the task list now uses workflow filters (`活动` / `失败` / `已完成` / `全部`), frontend-only transfer view models, a quasi-table task list, icon-based row actions, row selection, short issue labels, short location display, and correct empty states. Verified with `pnpm run build`; manual visual QA and commit are pending.
- Transfer Phase 2 is code-complete pending manual QA: the right detail pane now defaults failed/cancelled tasks to `诊断`, separates `概览` / `轨道` / `事件` / `原始日志`, keeps raw logs out of the primary view, redacts obvious cookie/signed URL material in displayed logs, can copy a redacted diagnostic summary, and `bdl-core::diagnostics` classifies 404, 403, FFmpeg missing, merge failure, and timeout into recommended actions.
- Transfer Phase 3/4 main code paths are implemented: `queue_retry` is ordinary retry, `queue_refresh_urls_and_retry` explicitly refreshes URLs before retry, frontend `refresh_retry` calls the explicit command, and Transfer now has a `BulkActionBar` for pause/resume/retry/refresh-retry/remove plus `清理已完成`. Backend bulk commands return per-task `updated`, `removed`, and `failed` results.
- Transfer Phase 5 main code paths are implemented: settings now expose only backend-honored transfer controls (`concurrent_tasks`, `retry_count`, `auto_refresh_expired_urls`), the worker honors configured concurrency and retry count, expired URL errors can auto-refresh once per task, and Transfer has a compact status strip with real counts plus `速度 --`.
- Transfer refactor checkpoint was committed as `794f97c feat: refine transfer manager workflow`.
- Task 14 collection and series parsing is implemented through `crates/bdl-core/src/resolver/collection.rs`, with `parse_load_more` / `parse_load_all` appending collection and series pages through the same normalized tree path.
- Task 14 bangumi and course parsing is implemented through dedicated `bangumi` and `cheese` resolvers. Bangumi `ss`/`ep` links normalize to season episode items and hydrate streams through `bangumi.playurl`; course `ss`/`ep` links normalize to paged course episode items and hydrate streams through `cheese.playurl`.
- Tauri selection hydration and task URL refresh now dispatch by source type instead of assuming every empty stream can be refreshed as a normal BV video.
- Task 14 resolver verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test bangumi_resolver --test cheese_resolver --test paged_resolvers`, `cargo test -p bdl-tauri state`, and `cargo check -p bdl-desktop`.
- Task 15 naming template slice is implemented: `bdl-core::naming` renders sanitized template paths, planner reserves duplicate output paths with `(1)` suffixes, settings persist `naming_template`, and the settings UI shows a sample preview.
- Task 15 naming verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test naming --test planner`, `cargo test -p bdl-tauri state`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.

Decisions:
- Main navigation: `解析`, `传输`, `设置`; account lives in the top-right account button. Completed records live under Transfer's `已完成` filter, not a separate History page.
- Core pipeline: `Input -> Resolver -> NormalizedDownloadPlan -> TaskQueue -> Downloader -> PostProcess -> History`.
- Frontend consumes normalized DTOs only; Rust backend owns business state and transfer state.
- Download engine is custom around `reqwest`/`tokio`, with a future `Fetcher` trait for optional alternate backends.
- Resume is resource-level, with URL refresh through `bpi-rs` when CDN URLs expire.
- Account state belongs in the top-right account menu/dialog, not Settings.
- Queue ordering controls are not part of the product scope.
