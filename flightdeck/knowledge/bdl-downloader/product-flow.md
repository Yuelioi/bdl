# BDL Product Flow

SUMMARY: BDL is a desktop Bilibili downloader focused on parsing links, selecting normalized media parts, controlling transfers, recovering failures, and producing predictable files.
READ WHEN: changing page structure, parse UX, transfer UX, settings, account UX, task actions, or user-facing copy.

---

## Product Position

BDL is a downloader, not a media library, browser, player, or recommendation client.

Primary user jobs:

- paste one or many Bilibili inputs
- inspect normalized results
- select many parts efficiently
- choose download settings
- monitor and control transfers
- recover failed or expired downloads
- open final files or folders

Out of scope:

- content discovery feeds
- full local media library
- player/poster wall
- browser extension
- permission bypass

## Navigation

Primary pages:

- `解析`: input, parse source switching, result selection, download settings dialog
- `传输`: active/failed/completed task management
- `设置`: download, media, archive, naming, advanced, diagnostics

Account lives in the top-right account button. It must not be placed in settings.

## Parse Flow

1. User enters BV/AV, URL, short link, or multi-line text.
2. Backend classifies input and resolves it into a normalized tree.
3. UI keeps parsed sources in a source switcher instead of replacing all previous results.
4. User searches, sorts, range-selects, or manually toggles visible results.
5. User clicks `下载已选择`.
6. A download settings dialog opens with task-level overrides.
7. Creating tasks stays on `解析`; show inline notice with `查看传输` action.

Current parse controls:

- source switcher with loaded/selected counts
- refresh current source
- close current source when multiple sources exist
- search result titles/owner/BV
- sort by original order, title, or duration
- range select visible results, e.g. `1-5,7,9-12`
- `全选可见`, not ambiguous download-all behavior
- `解析更多` and guarded `解析全部`

## Selection Rules

The frontend owns selection until tasks are created.

Default selection:

- single normal video: selected
- multi-part normal video: all parts selected
- list sources: not selected by default
- bangumi, course, collection, and series: parsed items are visible and user-selected

The primary action is always `下载已选择`.

Paged sources are progressive:

```text
已加载 30 / 1240
[解析更多] [解析全部]
```

`解析全部` must confirm the maximum load limit before starting.

## Download Settings Dialog

Settings page defines defaults. The parse dialog provides temporary overrides for the selected batch.

Visible common options:

- save directory
- download content: audio+video, video only, audio only
- video quality
- audio quality

Advanced options:

- codec preference
- container: mp4 or mkv
- archive mode: final media, complete archive, or settings-defined custom archive

Do not show all archive/diagnostic internals in the first view.

## Transfer Flow

Transfer is a task manager, not a log viewer.

Filters:

- `活动`: waiting, parsing, downloading, muxing, paused, failed, cancelled
- `失败`: failed and cancelled
- `已完成`: completed records
- `全部`: every current record

Default filter:

- if work needs attention, show `活动`
- otherwise show `已完成`
- do not override the filter after the user manually changes it

List columns should remain scan-stable:

```text
名称 | 状态 | 进度 | 速度 | 剩余 | 操作
```

Short issue labels belong in the title cell or inspector; raw logs do not belong in the table.

Transfer controls:

- row actions are state-specific
- details open in a modal inspector
- bulk actions support pause, cancel, resume, retry, refresh links and retry, remove, clear completed
- list sorting supports default, name, progress, speed, and issue priority
- completed records support search by title, source, or output path

## Task Semantics

Pause, cancel, retry, remove, and clear completed are distinct:

- Pause stops active downloading and keeps partial files plus queue state.
- Cancel stops active downloading and keeps the transfer record.
- Retry puts failed/cancelled/completed work back into the queue and keeps completed resources when possible.
- Refresh links and retry refreshes expiring media URLs before retry.
- Remove deletes the transfer record, not final output files.
- Clear completed removes completed transfer records from the list.

Task states:

```text
waiting
parsing
downloading
muxing
completed
failed
paused
cancelled
```

Use user-facing media terms:

- `视频轨道`
- `音频轨道`
- `字幕`
- `封面`
- `弹幕`
- `NFO`

Avoid exposing `resource` except in technical diagnostics.

## Task Inspector

The inspector should prioritize:

1. status and recommended action
2. short diagnosis for failed/cancelled tasks
3. progress and output location
4. tracks
5. events
6. raw logs

Raw logs must be behind an explicit tab and redacted.

## Account UX

Login is on demand.

Top-right account button states:

- logged out: show login affordance
- logged in: show account identity and status

Supported flows:

- QR login
- cookie import
- verify account
- logout

Cookie storage must persist through OS credential storage. SQLite stores only account summaries.

Logged-out parsing should degrade by available streams. Missing high quality is not an error unless the resource itself requires login.

## Settings

Settings expose backend-honored behavior only.

Sections:

- `下载`: save directory, concurrency, retries, segments, expired URL refresh, startup recovery
- `默认媒体`: video quality, audio quality, codec, container
- `命名`: naming template, duplicate path behavior, preview, variables
- `归档和素材`: final media/archive/custom assets, raw streams, cover/subtitle embedding
- `高级`: proxy, log level, data directory, cleanup, diagnostics export

Naming template defaults:

```text
{title}/P{part_index} - {part_title}.{ext}
{series_title}/S{season_index}E{episode_index} - {episode_title}.{ext}
{collection_title}/{index} - {title}.{ext}
```

## Feedback And Errors

Prefer inline feedback near the work surface.

Use global toast only for severe or cross-surface failures. Non-error toasts disappear after 3 seconds. Error toasts can persist until dismissed.

Errors should be short and actionable:

- parse failed: explain accepted input types
- login expired: sign in again
- private resource: login required
- expired media URL: refresh links and retry
- network timeout: retry
- ffmpeg missing: configure ffmpeg path
- save directory issue: choose a writable directory

Logs and diagnostics must redact cookies, sensitive headers, and long signed URLs.

## UI Quality Bar

- Common actions visible; uncommon details folded into dialogs, tabs, or disclosures.
- No page-level horizontal scrolling at 1100px wide.
- Long titles and paths truncate with tooltip or move to inspector.
- Controls use Nuxt UI/local wrappers and Tabler icons consistently.
- Theme remains quiet, compact, and task-focused with restrained dark-green accent.
- Do not add a separate History page unless the product becomes a media library.
