# Index - bdl-downloader-app

## State

Task 14 is code-complete on branch `bdl-downloader-app`. The workspace has a Rust/Tauri/Vue scaffold, frontend-safe normalized DTOs, a pure Bilibili input classifier, `bpi-rs` backed video/favorite/uploader/collection/series/bangumi/cheese resolvers, a planner that turns selected normalized parts into backend-owned download tasks/resources, a resumable `reqwest` fetcher, an ffmpeg muxer, a CLI parse/download path, registered Tauri commands/events, a custom Vue UI kit, parse/transfer pages, SQLite-backed queue persistence, and top-right account login through Cookie import or QR scan.

## Next

Continue from `flightdeck/work/bdl-downloader-app/open-task-list.md`. Immediate focus is Parse selection IA, Settings clarity, unfinished archive assets, completed records inside Transfer, and Transfer manual QA. Do not use the older History-page wording in `plan.md` as current product direction.

## Read now

- flightdeck/knowledge/bdl-downloader/product-flow.md
- flightdeck/knowledge/bdl-downloader/architecture.md
- flightdeck/work/bdl-downloader-app/open-task-list.md
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
- Task 15 archive asset execution is partially implemented: complete archive cover resources download when URLs exist, NFO files are generated locally after muxing, and currently unavailable cover/subtitle/danmaku resources are completed with warning logs instead of remaining pending. Subtitle/danmaku API fetching and custom asset selection remain pending.
- Task 15 archive execution verification passed: `cargo fmt --all --check`, `cargo test -p bdl-tauri commands`, and `cargo check -p bdl-desktop`.
- P0 parse/settings continuation landed in commit `31e641e feat: complete parse and settings defaults`.
- The parse page now supports multi-line input, text-file import, source refresh, retained-source cap prompts, and explicit `解析全部` confirmation. Settings now exposes backend-honored naming, duplicate-path, media, proxy, log, data directory, cleanup, and diagnostics controls. The planner honors video/audio quality, codec preference, missing-quality policy, duplicate naming strategy, custom ffmpeg path, proxy, and raw stream retention. Transfer now shows a side-nav badge after parse creates tasks.
- P0 continuation verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test settings --test fetcher_resume --test planner --test naming --test muxer`, `cargo test -p bdl-tauri state`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.
- Manual Transfer visual QA at `1365x768` and `1100x720` is still pending because the in-app browser backend was unavailable in this session (`agent.browsers.list()` returned no browsers).
- P1 archive subtitle/danmaku fetching landed in commit `b7c266c`.
- Normal video resolving now reads subtitle URLs from `video.player_info_v2` through `bpi-rs` when stream hydration is requested, creates danmaku XML assets through `bpi-rs` `DanmakuXmlListParams`, carries archive asset URLs/headers into planner resources, and downloads subtitle/danmaku asset resources when URLs exist.
- P1 archive verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test video_resolver --test planner --test normalization`, `cargo test -p bdl-tauri should_fetch`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.
- P1 custom archive selection landed in commit `9bf256b`.
- Settings now exposes `自定义归档` with cover/subtitle/danmaku/NFO toggles and raw stream retention in the same archive section. `bdl-core::planner` supports `ArchiveMode::Custom` and filters archive resources through `ArchiveAssetSelection`; old settings files default to full asset selection.
- Custom archive verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test planner --test settings`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.
- P1 optional archive embedding landed in commit `5d1bfc1`.
- Settings now exposes optional cover/subtitle embedding. The muxer embeds supported cover images for MP4-like outputs and supported subtitle sidecars for MP4/MKV outputs; unsupported formats such as current Bilibili JSON subtitles stay as sidecar files and produce warning logs.
- Embedding verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test muxer --test settings`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, `cargo check -p bdl-cli`, and `pnpm --dir apps/desktop build`.
- P1 archive warning display landed in commit `8a28baa`.
- Completed tasks with warning logs now surface as `已完成 · 有警告` in Transfer and show warning details in the diagnosis panel instead of looking like a fully clean archive.
- Archive warning display verification passed: `pnpm --dir apps/desktop build`.
- P1 completed-record search/actions landed in commit `8aa13a9`.
- Transfer `已完成` now has search by title, source id/link, and output path. Completed task actions include open file, open folder, re-download, copy source, remove record, and existing clear-completed bulk cleanup. Re-download of completed tasks resets all resources to pending so cleaned raw streams are fetched again.
- Completed-record search/actions verification passed: `pnpm --dir apps/desktop build`, `cargo test -p bdl-tauri state`, `cargo check -p bdl-desktop`, and `cargo fmt --all --check`.
- P1 completed-record metadata persistence landed in commit `38ead80`.
- `DownloadTask` now carries a `media_selection` snapshot from the planner, including selected video quality, audio quality, codec, and output container. `TaskStorage` migrates existing SQLite databases, persists that task metadata, and writes durable `history` completed records with final output path, source id, completion time, and redacted warning/error summary when a task completes.
- Completed-record metadata verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test storage --test planner --test diagnostics`, `cargo test -p bdl-tauri state`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.
- P1 durable refresh intent landed in commit `bd46547`.
- New tasks now persist `refresh_intent` separately from media URLs and task IDs. The planner writes stable refresh identity for normal video BV/AV + CID and for bangumi/course EP + CID; URL refresh uses this field first and only falls back to legacy task ID parsing for already-persisted tasks.
- Durable refresh intent verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test planner --test storage --test diagnostics`, `cargo test -p bdl-tauri state`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.
- P1 backup CDN fallback landed in commit `4ddaff2`.
- `ReqwestFetcher` now attempts every non-empty URL in `DownloadResource.current_urls` before returning failure to the queue worker. Attempts are interleaved by retry round, so a failed primary CDN is followed by backup URLs before the automatic full URL refresh path runs.
- Backup CDN fallback verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test fetcher_resume`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.
- P1 ranged-download consistency checks landed in commit `90f3376`.
- Fetch state now stores ETag and Last-Modified from HEAD responses in addition to length and downloaded bytes. Ranged resume continues only when the saved byte count and length still match and any shared ETag/Last-Modified validators are unchanged; otherwise the `.bdlpart` and state file are removed and the resource restarts cleanly.
- Ranged consistency verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test fetcher_resume`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.
- P1 real transfer progress landed in commit `fd49836`.
- The queue worker now forwards fetcher progress through `queue://progress-updated` events. The desktop queue store aggregates resource bytes per task, estimates per-task and global speed, derives ETA from known totals, uses byte progress when available, and shows downloaded/total size in the Transfer progress column instead of leaving active downloads at placeholder-only metrics.
- Real progress verification passed: `cargo fmt --all --check`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, and `pnpm --dir apps/desktop build`.
- P1 segmented downloads landed in commit `70af626`.
- `ReqwestFetcher` now honors `FetchConfig.segment_count` for fresh downloads with known content length, splits resources into bounded Range requests, merges segment files into the normal `.bdlpart` path, and keeps existing ranged resume behavior on already-started `.bdlpart` files. Settings now exposes `单任务分段数` with supported values 1/2/4/8.
- Segmented download verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test fetcher_resume --test settings`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, `pnpm --dir apps/desktop build`, and `git diff --check`.
- P1 startup recovery prompt landed in commit `506ed7f`.
- On startup, persisted `waiting` / `parsing` / `downloading` / `muxing` tasks are normalized into recoverable queue state. By default they are paused and surfaced through a global recovery prompt; when `启动时自动继续未完成任务` is enabled, the app resumes those tasks after loading the queue. `queue_list` no longer starts the worker implicitly.
- Startup recovery verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test settings`, `cargo test -p bdl-tauri state`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, `pnpm --dir apps/desktop build`, and `git diff --check`.
- P1 task log retention landed in commit `ff49c40`.
- `TaskStorage::append_task_log` now prunes task logs after each insert, deleting entries older than 30 days and keeping only the latest 1000 logs for the affected task. Existing task logs are still preserved when queue snapshots are replaced.
- Task log retention verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test storage`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, and `git diff --check`.
- P1 Transfer inspector diagnostics export landed in commit `ac15dad`.
- The task inspector now exposes `导出诊断` in both the diagnosis tab and raw-log toolbar, reusing the existing backend diagnostics export command and surfacing the export path through a toast. Settings still keeps its existing diagnostics export action.
- Transfer inspector diagnostics export verification passed: `pnpm --dir apps/desktop build` and `git diff --check`.
- P1 diagnostic redaction hardening landed in commit `78d9bd0`.
- Rust redaction now handles Cookie, Authorization, Proxy-Authorization, common Bilibili cookie/query keys, signed URL query strings, and oversized response bodies. Diagnostics export writes redacted task URLs, resource headers, and task logs instead of raw queue snapshots. Transfer inspector log redaction uses the same broader key set and truncates large messages.
- Diagnostic redaction verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test account --test storage`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, `pnpm --dir apps/desktop build`, and `git diff --check`.
- P1 action-specific errors landed in commit `14b498e`.
- Tauri commands now return specific error codes/messages for unrecognized parse input, unsupported sources, unwritable output paths, missing FFmpeg, login-required failures, and private/inaccessible resources. Transfer diagnostics distinguish unwritable save directories, private resources, login failures, missing FFmpeg, expired URLs, and generic mux failures.
- Action-specific error verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test diagnostics`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, `pnpm --dir apps/desktop build`, and `git diff --check`.
- P2 OS credential storage and SQLite account summary landed in commit `d4cd069`.
- Account cookies now persist through the OS credential store via `keyring 3.6.3`; the old `.bdl/account.cookie` file is migrated into the credential store and deleted. SQLite now stores only `AccountSummary` in `account_summary`, and import/logout/verification keep the keyring, in-memory cookie, and SQLite summary in sync.
- Account storage verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test storage --test account`, `cargo test -p bdl-tauri secure_store`, `cargo test -p bdl-tauri state`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, `pnpm --dir apps/desktop build`, and `git diff --check`.
- P2 startup account verification landed in commit `f818154`.
- Startup now schedules a background account verification that calls `login.nav` through `bpi-rs`, refreshes the persisted `AccountSummary` from verified profile data, clears local account state only when Bilibili reports the session is logged out, and emits `account://updated` when verification completes. The frontend registers the account listener before the initial account load.
- Startup account verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core --test account`, `cargo test -p bdl-tauri state`, `cargo test -p bdl-tauri commands`, `cargo check -p bdl-desktop`, `pnpm --dir apps/desktop build`, and `git diff --check`.
- P2 workspace check script landed in commit `f3c9475`.
- `scripts/check.ps1` now runs Rust formatting, clippy with warnings as errors, the full Cargo workspace tests, the desktop frontend build, and `git diff --check`. The script explicitly checks native command exit codes so PowerShell cannot report a failed cargo command as success.
- Workspace check verification passed by running `powershell -ExecutionPolicy Bypass -File .\scripts\check.ps1`; clippy cleanup was included for the warnings surfaced by that script.
- P2 packaging script landed in commit `57d619e`.
- `scripts/package.ps1` runs the workspace checks by default, executes `pnpm --dir apps/desktop tauri build`, then prints the release executable and bundle artifacts discovered under `target/release`. `-SkipCheck` is available for packaging-only reruns after checks have already passed.
- Packaging verification passed with `powershell -ExecutionPolicy Bypass -File .\scripts\package.ps1 -SkipCheck`, producing `target/release/bdl-desktop.exe`, `target/release/bundle/msi/BDL_0.1.0_x64_en-US.msi`, and `target/release/bundle/nsis/BDL_0.1.0_x64-setup.exe`.
- P2 minimum-window QA at 1100x720 completed.
- Chrome/Playwright checked `解析`, `传输`, and `设置` at `1100x720`; each page reported `document.body.scrollWidth == document.body.clientWidth == 1100` and no visible right overflow. Clean screenshots were inspected after dismissing browser-only Tauri `invoke` fallback toasts. No page-level horizontal scrollbar, major overlap, or blocked primary workflow panel was observed.
- P2 release QA checklist landed in `flightdeck/work/bdl-downloader-app/release-qa.md`.
- The release checklist covers parse, download/transfer recovery, account persistence, diagnostics/log redaction, missing FFmpeg messaging, and minimum-window inspection, with explicit expected outcomes and evidence to record per release candidate.
- Transfer visual QA follow-up is complete. Chrome/Playwright rendered real Transfer components with mocked Tauri startup, settings, account, queue, logs, and command responses at `1365x768` and `1100x720`. Both viewports reported `documentElement.scrollWidth == clientWidth`, no visible horizontal scrollers, and inspector raw logs/track lists stayed inside their panels. The narrow `1100x720` layout now gives the task list usable height and lays the inspector summary/tabs side-by-side so the summary actions are not clipped.
- Persisted Transfer workflow verification is covered by `state::tests::persisted_queue_management_workflows_survive_reload`. The test uses a temporary real SQLite `TaskStorage`, exercises pause/resume, retry, refresh-link URL replacement plus retry, remove, and clear-completed behavior through `AppState`, then reopens storage and verifies the results persisted.
- Transfer QA verification passed: `cargo fmt --all --check`, `cargo test -p bdl-tauri state`, `pnpm --dir apps/desktop build`, and `git diff --check`.
- Plan maintenance is complete. `plan.md` now treats Task 15 as completed under current product decisions, explicitly keeps completed records inside Transfer instead of a separate History page, marks the older completed Task 1-3 checkboxes, and removes the stale instruction that agent workers should use subagents by default.

Decisions:
- Main navigation: `解析`, `传输`, `设置`; account lives in the top-right account button. Completed records live under Transfer's `已完成` filter, not a separate History page.
- Core pipeline: `Input -> Resolver -> NormalizedDownloadPlan -> TaskQueue -> Downloader -> PostProcess -> History`.
- Frontend consumes normalized DTOs only; Rust backend owns business state and transfer state.
- Download engine is custom around `reqwest`/`tokio`, with a future `Fetcher` trait for optional alternate backends.
- Resume is resource-level, with URL refresh through `bpi-rs` when CDN URLs expire.
- Account state belongs in the top-right account menu/dialog, not Settings.
- Queue ordering controls are not part of the product scope.
