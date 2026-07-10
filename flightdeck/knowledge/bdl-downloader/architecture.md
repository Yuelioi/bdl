# BDL Architecture

SUMMARY: BDL is a Rust/Tauri/Vue downloader where Rust owns Bilibili resolution, task state, downloading, persistence, and OS integration; Vue owns presentation and selection interaction.
READ WHEN: changing crates, normalized DTOs, queue/download behavior, Tauri commands/events, storage, account persistence, or frontend/backend boundaries.

---

## Core Boundary

BDL follows one pipeline:

```text
Input -> Resolver -> Normalized Tree -> Planner -> Queue -> Fetcher -> PostProcess -> Completed Record
```

Rules:

- `bdl-core` owns domain logic: input classification, resolvers, normalized models, planning, fetching, muxing, naming, settings, storage DTOs, diagnostics, and account/session helpers.
- `bdl-tauri` owns app glue: command registration, event emission, app data paths, queue orchestration, secure cookie storage, OS file opening, and startup recovery.
- `apps/desktop` owns UI state and interaction: parse input, visible selection state, task presentation, dialogs, filters, and settings forms.
- Frontend never infers durable task state. It displays backend tasks/events and sends commands.

## Workspace

```text
crates/bdl-core      domain model, resolver, planner, fetcher, muxer, settings, storage
crates/bdl-tauri     Tauri commands, queue worker, secure store, OS integration
crates/bdl-cli       developer CLI using bdl-core
apps/desktop         Vue 3 + Tauri frontend
```

## Backend Model

The frontend consumes stable normalized DTOs, not raw API responses:

```text
NormalizedSourceTree
  SourceSummary
  NormalizedGroup[]
    NormalizedItem[]
      NormalizedPart[]
        MediaStream[]
        DerivedAsset[]
```

Important invariants:

- All source types normalize into this tree: video, favorites, uploader lists, collections, series, bangumi, and courses.
- `SourceSummary.loaded_count`, `total_count`, and `has_more` drive progressive paging.
- A part must preserve enough identity to refresh media URLs later: source id, item/part id, aid/bvid/cid, quality, codec, and media kind.
- Settings defaults are applied by the planner and can be overridden per task creation request.

## Download Tasks

`DownloadTask` is backend-owned and persisted. It contains:

- stable task id and source id
- title and output path
- task status
- optional UTC `scheduled_at` start time
- optional per-task `speed_limit_bytes_per_second`
- resource list
- media selection snapshot
- refresh intent for expiring media URLs

`DownloadResource` tracks each downloadable unit:

- intent: video, audio, cover, subtitle, danmaku, or NFO
- current URLs and request headers
- target path and temporary path
- status: pending, downloading, completed, failed, paused, cancelled

Retry skips completed resources and resumes incomplete resources from `.bdlpart` state when possible.

## Fetching And Cancellation

`ReqwestFetcher` is the current fetcher. It supports:

- range resume for existing `.bdlpart` files
- resumable segmented downloads when `segment_count` is 2/4/8; the product default is 4
- CDN fallback through current URL lists
- size/metadata checks with ETag or last-modified when available
- wakeable cancellation through `FetchCancelToken`; metadata requests, GET requests, and response streams race their waits against cancellation
- a shared token-bucket limiter: all active tasks share the global byte budget, while every task's segments share its optional task budget

Pause and cancel are not just queue-state changes. They must cancel the running fetch stream, persist resource status, and allow resume/retry to rebuild pending resource state.

Settings carry a schema version. The first transfer-engine migration upgrades the historical single-segment default to four once; after migration, an explicit one-segment choice remains respected.

Chunk progress is coalesced by the Tauri layer to a bounded UI cadence (currently 200 ms) with a final flush. The frontend derives displayed speed from a rolling cumulative-byte window, not one adjacent event pair. This keeps high-throughput downloads from flooding Vue and makes the speed label resistant to burst timing.

Global speed limiting is a live setting. The queue worker owns one `BandwidthLimiter` shared by every active `ReqwestFetcher` and updates it when settings change. A task-specific limit is persisted with the task, captured when the task starts, and shared by all of that task's range segments. Active tasks must be paused before their task-specific limit is edited; changing the global limit does not restart transfers.

## URL Refresh

Media URLs expire. Current URLs are cache, not identity.

Policy:

1. Try current URLs.
2. Try backup URLs from the same resolved payload.
3. On 404, signature expiry, or clear permission failure, refresh URLs using the task refresh intent.
4. If refresh fails, keep a short classified failure on the task and expose retry or refresh-and-retry.

Do not mix byte ranges from different URLs unless resource consistency is verified.

## Post Processing

`MediaMuxer` owns ffmpeg integration:

- detect configured or system ffmpeg
- merge video/audio
- output mp4 or mkv
- optionally embed cover/subtitles
- optionally retain raw streams
- capture exit code and stderr summary

Final output must exist and be non-empty before the task is marked completed.

Successful muxing is a commit point. Muxing/completed tasks ignore late pause or cancel transitions, completion is persisted before optional raw-stream cleanup, and post-processing errors must not downgrade a valid final output. For older interrupted tasks, a non-empty final output plus completed-but-cleaned media resources is sufficient to reconcile the task to completed instead of attempting another mux with missing inputs.

## Persistence

SQLite persists tasks, resources, completed transfer records, task logs, and account summaries. Settings are JSON. Cookies are stored in the OS credential store, not SQLite or settings JSON.

Scheduled tasks remain backend-owned waiting tasks. `AppState` excludes future schedules from worker pickup and startup recovery, persists the UTC start time, and notifies a capacity-aware Tokio worker when schedules, queue state, or settings change. The worker races active-task completion against the next due time so a scheduled task can fill an available concurrency slot without waiting for the current batch to finish. It re-reads settings before filling worker slots, ensuring a long-sleeping schedule uses the latest concurrency, network, FFmpeg, and archive configuration. The frontend only submits local time as RFC 3339 and renders backend task state.

Files:

```text
settings.json
tasks.sqlite
*.bdlpart
logs/
```

Task logs are retained for 30 days or the latest 1000 entries per task. Logs and diagnostics must redact cookies, auth headers, and signed URLs.

## Queue And Recovery

The queue worker:

- honors `concurrent_tasks`
- uses configured retry count
- emits task, log, and progress events
- records completed metadata
- can auto-refresh expired URLs once per task when enabled

Logical task identity is the deterministic planner ID (`task:<source>:<part>`). Explicit duplicate copies append `:copy:<n>` while still matching the same logical identity. `AppState` applies duplicate detection, `ask`/`skip`/`create`, copy IDs, output-path reservation, and persistence under one queue lock; task/resource re-keying and path retargeting remain `bdl-core` domain behavior.

Startup recovery:

- persisted waiting/parsing/downloading/muxing tasks are normalized on app start
- default behavior pauses incomplete tasks and prompts the user
- `startup_auto_recovery` can resume them automatically

## Commands And Events

Keep command names stable because the frontend wraps them in `apps/desktop/src/api/tauri.ts`.

Core commands:

```text
parse_create_source
parse_load_more
parse_load_all
parse_close_source
parse_refresh_source
selection_create_tasks

environment_health
environment_create_download_directory

queue_list
queue_startup_recovery
queue_dismiss_startup_recovery
queue_logs
queue_pause
queue_resume
queue_schedule
queue_unschedule
queue_set_speed_limit
queue_cancel
queue_retry
queue_refresh_urls_and_retry
queue_remove
queue_open_file
queue_open_dir
queue_bulk_pause
queue_bulk_cancel
queue_bulk_resume
queue_bulk_retry
queue_bulk_refresh_urls_and_retry
queue_bulk_remove
queue_clear_completed

settings_get
settings_update
settings_cleanup_cache
settings_cleanup_temp_files
diagnostics_export

account_get
account_login_qr_start
account_login_qr_poll
account_import_cookie
account_logout
account_verify
```

Events:

```text
parse://source-updated
parse://items-appended
queue://task-updated
queue://progress-updated
queue://log-appended
settings://updated
account://updated
```

## Frontend Architecture

Stores:

```text
useParseStore      parse sources, active source, frontend selection, create task requests
useQueueStore      persisted task queue, logs, progress, filters, bulk actions
useSettingsStore   settings defaults and draft/save behavior
useAccountStore    account state and login flows
useUiStore         active tab, dialogs, toasts
```

UI rules:

- Use Nuxt UI components through local wrappers where wrappers exist.
- Use Tabler icons through icon names, not ad hoc SVG.
- Keep `解析`, `传输`, and `设置` as the only primary navigation pages.
- Account belongs in the top-right account button, not settings.
- Completed records live under Transfer's `已完成` filter, not a separate page.

## Developer Checks

Before claiming backend-affecting work complete, run the narrow relevant Rust tests plus:

```text
cargo fmt
cargo clippy -p bdl-core -p bdl-tauri --all-targets -- -D warnings -A clippy::too_many_arguments
pnpm -C apps/desktop run build
```
