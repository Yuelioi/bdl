# BDL Product Flow

SUMMARY: BDL is a desktop Bilibili downloader focused on parsing links, selecting normalized media parts, controlling transfers, recovering failures, and producing predictable files.
READ WHEN: changing page structure, parse UX, transfer UX, settings, account UX, task actions, or user-facing copy.

---

## Product Position

BDL is a downloader, not a media library, browser, player, or recommendation client.

`内容库` is a narrow account-source picker, not a discovery feed or local-media catalog. It exposes only authenticated sources that can enter the existing parse flow.

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

- `解析`: source input, single-source or link-level batch results, selection, and download settings dialog
- `内容库`: authenticated created/collected favorite folders, filtering, folder detail, pagination, and direct download planning
- `传输`: active/failed/completed task management
- `设置`: download, media, naming, processing, additional content, updates, network, and maintenance
- `关于`: installed version plus project, author, and website links

Account lives in the top-right account button. It must not be placed in settings.

## Account Library Flow

1. A verified account opens `内容库`.
2. Backend reads created favorite folders or collected favorite folders through the authenticated Bilibili API.
3. Folder lists use top tabs, local filtering, and a pagination component; folder cards are navigation targets, not multi-select controls.
4. Opening one folder enters an in-page detail view with paged video cards, per-video download, page selection, selected download, and download-all.
5. The detail view reuses the parse tree and download planner. Account metadata is never treated as queue truth, and playable URLs are still hydrated only when tasks are created.

Logged-out, loading, empty, API-error, pagination, cover-fallback, and narrow-window states are explicit. Cached account-library pages are cleared whenever the active account changes.

## Parse Flow

1. User enters BV/AV, URL, short link, or multi-line text.
2. Backend classifies input and resolves only the metadata needed for the normalized tree. Initial parsing must not fetch playable stream URLs for every visible part.
3. Multiple input lines enter link-level batch mode: each input is represented as one video entry and shares one download configuration. A single container link retains its internal selectable content tree.
4. User searches, sorts, range-selects, or manually toggles loaded results.
5. User clicks `下载所选`.
6. A download settings dialog opens with task-level overrides.
7. Creating tasks stays on `解析`; show inline notice with `查看传输` action.

Playable stream URLs, codecs, and quality profiles are hydrated when tasks are created, and only for the selected CID when the source already exposes one. Hydrating one selected part must preserve streams already hydrated for sibling parts.

Duplicate confirmation and repeated download actions reuse the already hydrated backend tree even when the frontend selection still contains the original list placeholder ID. The backend maps favorite/uploader/collection/series placeholders to the hydrated parts with the same BVID; it must not ask the user to reparse the source.

Current parse controls:

- refresh current source
- close current result and return to source input
- search result titles/owner/BV
- sort by original order, title, or duration
- range-select the current filtered/sorted results, e.g. `1-5,7,9-12`
- select all currently loaded/search-matching results without implying that unloaded pages are included
- paged list sources load lightweight metadata progressively; users can request 200, 500, or 1000 more items per operation

## Selection Rules

The frontend owns selection until tasks are created.

Default selection:

- single normal video: selected
- multi-part normal video: all parts selected
- list sources: not selected by default
- bangumi, course, collection, and series: parsed items are visible and user-selected

The primary selection action is `下载所选`; account-folder detail also offers explicit per-item and `下载全部` actions.

Paged sources expose loaded and known-total lightweight metadata while more pages remain:

```text
已加载 200 项 · 共 13030 项
```

Initial parse and refresh fetch the first useful metadata page rather than exhausting an arbitrarily large source. `解析更多` uses the selected 200/500/1000 batch size and stops when the source reports `has_more = false`. Download-all from an account-folder detail may explicitly exhaust remaining lightweight pages before task creation. Paging still fetches metadata only; stream URLs and codec profiles remain download-time hydration.

The result area presents this metadata as a content selector. Its stable counts are `已加载 N 项`, optional `共 N 项`, and `已选 N 项`; an active search adds `搜索找到 N 项`. It must not expose the legacy `可见` or `未拉流` vocabulary. Range selection operates on the current sorted/search result, and the UI explains once that stream details are fetched only for selected content at task creation.

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

Before task creation, the same environment health contract used by Settings checks the effective batch download directory and the persisted FFmpeg configuration. Task creation is blocked while either component is unhealthy. A missing directory can be created inline; FFmpeg repair routes to Settings so the chosen executable is saved before the queue worker uses it.

The download dialog optionally accepts a one-time local start time. The backend stores it in UTC, keeps future tasks out of the worker, and restores the wakeup after restart. Transfer rows label scheduled work explicitly and offer `立即开始` plus `修改时间`; paused tasks can also be scheduled. Invalid or past times are shown inline beside the native datetime input.

The same dialog optionally accepts a per-task speed cap in MiB/s. Leaving it empty means the task only follows the global cap. Settings exposes a live global cap shared across concurrent tasks. Transfer actions let waiting, scheduled, paused, and failed tasks change or clear their own cap; an active download must be paused first. All range segments consume one task budget rather than multiplying the configured speed.

Task creation uses an explicit duplicate policy:

- `ask` is the default and is atomic: if any selected logical task already exists, create nothing and return the matches for confirmation.
- `skip` creates only non-duplicate selections.
- `create` creates safe copies with new task/resource IDs and output paths that do not collide with queued work.

The duplicate dialog shows a short status-aware preview and offers cancel, skip duplicates, or create anyway. It must not silently choose on the user's behalf.

The parsed-source tree uses one roving Tab stop. Arrow Up/Down and Home/End move through visible rows; Arrow Right enters an expanded group, Arrow Left returns to its parent, and Enter/Space toggles selection.

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
- A completed task only shows `有警告` for completion or archive warnings. Historical control-flow notices such as pause, cancel, and automatic URL refresh remain in the timeline but do not contaminate the final status.
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
- `媒体`: video/audio quality and container
- `文件命名`: naming template, duplicate path behavior, preview, variables
- `编码与处理`: codec, segments, raw-stream retention, and FFmpeg
- `附加内容`: cover, subtitle, danmaku, NFO, and embedding controls
- `应用更新`: installed version, default-off automatic detection, manual check, and signed install
- `网络与维护`: proxy, log level, data directory, cleanup, environment health, and diagnostics export

Naming template defaults:

```text
{title}/P{part_index} - {part_title}.{ext}
{series_title}/S{season_index}E{episode_index} - {episode_title}.{ext}
{collection_title}/{index} - {title}.{ext}
```

## Feedback And Errors

Prefer inline feedback near the work surface.

Use global toast only for severe or cross-surface failures. Non-error notices and toasts share the five-second auto-dismiss setting. Error feedback persists until dismissed or replaced.

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
- Compact task-row actions use one border treatment: secondary icon actions remain borderless inside the row, and focus indicators render inside clipped scroll regions so no edge is cut off.
- Theme remains quiet, compact, and task-focused. Both modes use achromatic cool-neutral structure: clean white/gray chrome in light mode and a charcoal surface ladder in dark mode. Bilibili pink is the sole brand accent; other hues appear only for semantic warning, danger, and success states.
- The desktop uses a frameless Tauri window with one custom title bar containing brand/version, conditional update notice, appearance, account, minimize, maximize/restore, and close controls.
- The global appearance menu offers system, light, and dark modes. System mode reacts to operating-system changes; explicit choices persist locally. Theme changes affect presentation only and never alter downloader state.
- Do not add a separate History page unless the product becomes a media library.
