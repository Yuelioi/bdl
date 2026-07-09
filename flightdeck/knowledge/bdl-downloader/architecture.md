# BDL Architecture

SUMMARY: BDL backend owns Bilibili resolution, normalization, task queue, resource-level downloading, ffmpeg post-processing, storage, and Tauri event boundaries.
READ WHEN: implementing or changing BDL crates, normalized models, task engine, download resume, Tauri commands/events, storage, or account persistence.

---

BDL is built around a strict pipeline:

```text
Input -> Resolver -> NormalizedDownloadPlan -> TaskQueue -> Downloader -> PostProcess -> History
```

The backend owns business state and downloading. The frontend owns presentation, selection state, and user interaction.

## Workspace Shape

Recommended structure:

```text
crates/
  bdl-core/        parsing normalization, planning, task engine, downloader, post-processing
  bdl-tauri/       Tauri commands, events, secure storage, OS integration
  bdl-cli/         thin developer CLI for testing core behavior
apps/
  desktop/         Tauri frontend
```

If the repo starts with a simpler Tauri layout, keep the same boundary: downloader logic belongs in `bdl-core`, not inside command handlers.

## Core Modules

`bdl-core` modules:

```text
input           input classification and source identity extraction
resolver        bpi-rs integration and content-specific resolvers
normalize       conversion into source tree and download plan DTOs
planner         selected nodes + options -> task/resource plan
queue           task lifecycle, concurrency, cancellation, event emission
fetcher         HTTP download, range segments, retries, CDN fallback
muxer           ffmpeg checks, merge, cover/subtitle embedding
storage         SQLite, settings snapshot, task/resource persistence
history         long-term download records
logging         task logs and redaction
account         cookie/session abstraction consumed by bpi-rs
```

`bdl-tauri` should be glue only:

- Command registration.
- Event forwarding.
- App data paths.
- Secure storage access.
- File opening and directory opening.
- Window and tray integration if needed.

## bpi-rs Boundary

`bpi-rs` is the API binding and session-aware Bilibili client. BDL should not duplicate API signing or response models unless it needs a stable app DTO.

Use `bpi-rs` for:

- `video.view`, `video.play_url`, and video detail APIs.
- Bangumi and course play URL APIs.
- Favorite, collection, series, and uploader list APIs.
- Login QR generation and polling.
- Cookie import and account verification.

BDL owns:

- Input classification.
- Result normalization.
- Quality selection policy.
- Task/resource state.
- Downloading and resume.
- URL refresh policy.
- ffmpeg post-processing.
- UI-facing DTOs.

## Normalized Models

The frontend consumes normalized DTOs, not raw `bpi-rs` responses.

Core tree:

```text
NormalizedSourceTree
  source: SourceSummary
  groups: Vec<NormalizedGroup>

NormalizedGroup
  id
  kind
  title
  items
  paging

NormalizedItem
  id
  title
  owner
  cover
  duration
  parts

NormalizedPart
  id
  title
  aid
  bvid
  cid
  streams
  assets
```

Stream model:

```text
MediaStream
  id
  kind: video | audio
  quality
  codec
  container
  bandwidth
  urls
  headers
  acquired_at
```

Asset model:

```text
DerivedAsset
  kind: cover | subtitle | danmaku | nfo
  format
  availability
  fetch_policy
```

The DTO must preserve enough resource identity to refresh expired URLs later.

## Resource Intent and URL Refresh

Do not treat play URLs as durable facts. Every downloadable resource stores intent separately from current URLs.

```text
DownloadResourceIntent
  source_id
  item_id
  part_id
  aid
  bvid
  cid
  media_kind
  quality_preference
  codec_preference
```

```text
ResolvedResourceUrl
  primary_url
  backup_urls
  headers
  acquired_at
  expires_hint
```

Refresh policy:

1. Try current URL.
2. Try backup CDN URLs from the same playurl payload.
3. If all URLs fail or a clear signature/permission failure occurs, refresh via `bpi-rs` using the resource intent.
4. If refresh fails, mark the resource failed and expose retry/reparse actions.

When switching CDN during range downloads, verify resource consistency using content length, ETag, last-modified, or equivalent response metadata when available. If consistency cannot be trusted, restart the resource download instead of mixing segments.

## Download Engine

The engine is custom because BDL downloads structured media plans, not plain URLs.

Use mature primitives:

- `reqwest` and `tokio` for async HTTP.
- `tokio::fs` for file writes.
- `futures` streams for concurrent segment scheduling.
- `serde` for state snapshots.
- `ffmpeg` command execution for muxing and embedding.

Keep the fetcher behind a trait:

```rust
trait Fetcher {
    async fn fetch(
        &self,
        resource: DownloadResource,
        target: std::path::PathBuf,
    ) -> Result<FetchReport, FetchError>;
}
```

First implementation:

```text
ReqwestFetcher
```

Possible future implementation:

```text
Aria2Fetcher
```

Do not bind the first version to an external download manager.

## Concurrency Defaults

Default transfer settings:

```text
global concurrent tasks: 2
segments per resource: 4
failure retries: 3
```

These are settings, not constants.

The queue controls task concurrency. The fetcher controls segment concurrency inside one resource.

## Resume Granularity

Resume at resource level, not only task level.

A video task may contain:

```text
video.m4s
audio.m4s
cover.jpg
subtitle.srt
danmaku.xml
movie.nfo
final.mp4
```

Each resource has its own state:

```text
pending
downloading
completed
failed
paused
cancelled
```

Resource state stores:

- URL intent.
- Current URL and backup URLs.
- Headers.
- Target path.
- Temporary path.
- Content length.
- ETag or last-modified when present.
- Segment ranges and downloaded bytes.

Completed resources are skipped on retry. Incomplete resources resume from `.bdlpart` state when possible. Expired URLs are refreshed before continuing.

## Integrity Checks

First version checks size, not hash.

Rules:

- Read content length before download when available.
- Track each segment's downloaded byte count.
- After segment merge, verify total size equals content length.
- If content length is unavailable, allow the download but mark it as `unverified_size`.
- Before muxing, verify video/audio files exist and are non-empty.
- After muxing, verify final file exists and is non-empty.

Hash verification is out of scope unless the source provides a stable hash.

## Post-Processing

`MediaMuxer` owns ffmpeg integration.

Responsibilities:

- Detect configured or system `ffmpeg`.
- Merge video and audio.
- Output mp4 or mkv.
- Optionally embed cover and subtitles.
- Optionally retain raw video/audio streams.
- Capture exit code and stderr summary.

The app should support a custom ffmpeg path. It should not require bundled ffmpeg at the code level, although platform releases may choose to bundle it.

## Tauri Commands

Command names:

```text
parse_create_source(input)
parse_load_more(source_id)
parse_load_all(source_id, limit?)
parse_close_source(source_id)
parse_refresh_source(source_id)

selection_create_tasks(source_id, selected_ids, options)

queue_list()
queue_pause(task_id)
queue_resume(task_id)
queue_cancel(task_id)
queue_retry(task_id)
queue_remove(task_id)
queue_open_file(task_id)
queue_open_dir(task_id)

settings_get()
settings_update(patch)

account_get()
account_login_qr_start()
account_login_qr_poll(session_id)
account_import_cookie(cookie)
account_logout()
account_verify()
```

Commands should return DTOs or accepted operation IDs. Long-running work emits events.

## Tauri Events

Event names:

```text
parse://source-updated
parse://items-appended
queue://task-updated
queue://resource-updated
queue://log-appended
settings://updated
account://updated
```

Task state is backend-authoritative. The frontend must not infer completion, failure, or paused state independently.

## Frontend Stores

Pinia store split:

```text
useParseStore      parse sessions, active source, result tree, selection state
useQueueStore      task queue, task events, filters
useSettingsStore   defaults, paths, ffmpeg, naming templates
useAccountStore    login status, account summary
useUiStore         drawers, dialogs, toasts, theme
```

Selection state is frontend-owned until `selection_create_tasks` is called. Transfer state is backend-owned.

## UI Kit

Because no component library is used, build a small internal UI kit before business pages spread.

Initial components:

```text
AppShell
Sidebar
Toolbar
Button
IconButton
TextField
Textarea
Select
SegmentedControl
Checkbox
Switch
Tabs
Dialog
Drawer
Toast
ProgressBar
StatusBadge
Tree
TaskList
TaskRow
EmptyState
```

Initial tokens:

```text
colors: background, surface, panel, border, text, muted, accent, danger, warning, success
spacing: 4, 8, 12, 16, 24, 32
radius: 4, 6, 8
font sizes: 12, 13, 14, 16, 18, 22
fixed heights: button 32, input 34, toolbar 40, task row 64
```

Business pages should use UI kit components instead of one-off controls.

## Responsive Layout

Desktop minimum:

```text
min-width: 1100
min-height: 720
```

Breakpoints:

- `>= 1280px`: main nav, source list, result tree, and details panel can be visible.
- `960-1279px`: main nav, source list, and result tree; details opens as a drawer.
- `< 960px`: narrow nav; source list and result tree switch views; details use a full-screen drawer.

## Persistence

Use SQLite for structured task data and history. Use JSON for settings.

Files:

```text
settings.json
tasks.sqlite
*.bdlpart
logs/
```

SQLite tables:

```text
tasks
resources
segments
history
task_logs
```

Settings JSON stores user preferences only. Cookie/session data must not be stored in JSON or SQLite in plaintext.

## Account Storage

Cookie persistence:

```text
system secure storage
```

SQLite account summary:

```text
display_name
mid
avatar_url
vip_status
last_checked_at
login_method
cookie_valid
```

Startup flow:

1. Read cookie from secure storage.
2. Inject it into the `bpi-rs` client.
3. Verify account state through account APIs.
4. Update SQLite account summary.
5. Emit `account://updated`.

Redact these values everywhere:

- `SESSDATA`
- `bili_jct`
- `DedeUserID`
- `Cookie`
- Authorization-like headers.

## Logs

Default retention:

```text
30 days or latest 1000 task summaries
```

Save:

- Stage changes.
- Parse summary.
- Resource selection summary.
- Download start and completion.
- Retry records.
- ffmpeg command summary and exit code.
- Error summaries.

Do not save:

- Raw cookies.
- Full sensitive headers.
- Long signed URL history.
- Large response bodies.

Detailed logs are opt-in through task details or debug mode.

## Startup Recovery

On startup, load persisted tasks and classify them:

- Incomplete.
- Completed.
- Failed.

Do not automatically resume downloads unless the user enabled startup auto-recovery. Incomplete tasks should be visible and resumable from the transfer page.

## Developer CLI

`bdl-cli` is a developer tool, not the primary product.

Commands:

```text
parse <input> --json
download <input> --output <dir> --quality best
verify-cookie
ffmpeg-check
```

The CLI should call `bdl-core` directly, making parser, planner, downloader, and account validation testable without clicking through the Tauri UI.

## Implementation Order

Recommended sequence:

1. Workspace skeleton: Tauri 2, Vue 3, TypeScript, `bdl-core`, `bdl-cli`.
2. Single-video vertical slice: input, `bpi-rs` view/playurl, normalization, task creation, video/audio download, ffmpeg merge.
3. UI kit, parse page, transfer page basics.
4. Task engine: concurrency, pause/resume, retry, resource-level resume.
5. Multiple parse sources and paged parsing for favorites, uploader videos, collections, and series.
6. Bangumi and course support.
7. Derived resources: cover, subtitles, danmaku, NFO, naming templates.
8. Account: cookie import first, QR login second, secure storage, verification.
9. History, settings, logs, cleanup.
10. Packaging and polish.
