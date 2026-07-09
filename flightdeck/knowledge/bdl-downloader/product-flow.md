# BDL Product Flow

SUMMARY: BDL is a Tauri desktop Bilibili downloader centered on parsing, normalized selection, transfer control, completed-task review, and settings.
READ WHEN: designing BDL product scope, page structure, parse UX, transfer UX, account UX, or download/archive options.

---

BDL is a desktop Bilibili downloader. The product is optimized for fast link parsing, clear selection, reliable transfer, and predictable media output. It is not a media library, player, recommendation client, or account resource browser.

## Product Scope

Supported content:

- Normal videos by BV, AV, short link, or full URL.
- Multi-part videos.
- Collections, series, favorites, and uploader video lists.
- Bangumi, movie, and course pages.
- Batch input by multi-line paste or text file import.

Out of scope:

- Bilibili browsing history, weekly picks, rankings, homepage recommendations, and followed bangumi list entry points.
- Full local media library, poster wall, player, or disk scanner.
- Browser extension and clipboard monitoring.
- Any paid-wall or permission bypass.

## Main Navigation

The app has three primary pages:

- `解析`: input, parsing, normalized result tree, selection, and download option editing.
- `传输`: task management for active, failed, completed, and all transfer records.
- `设置`: download, media, archive, and advanced settings.

Account access is not a settings page. The top-right account button owns login, account status, cookie import, verification, and logout.

## Parse Page

The parse page is the main entry point.

Layout:

- Top: input area for a single link, mixed text, BV/AV IDs, or multi-line batch input.
- Left: source list. Each parsed input becomes a retained source with type, status, selected count, close, retry, and refresh actions.
- Center: normalized result tree for the active source.
- Right: detail and download settings panel. On narrower windows this becomes a drawer.

The parse page keeps multiple sources instead of replacing previous results. The default cap is the latest 20 sources, after which the app asks the user to clear older parse results.

## Input Flow

Input handling is split into two stages:

1. `InputClassifier`: classifies raw text or URL into a `SourceKind` and extracted IDs.
2. `Resolver`: calls `bpi-rs` and returns a normalized source tree.

Accepted inputs:

- BV ID.
- AV ID.
- Normal video URL.
- b23 short link.
- Bangumi or movie URL.
- Course URL.
- Favorite URL.
- Collection or series URL.
- Uploader space URL.
- Multi-line mixed input.

Unrecognized input should produce a short actionable message:

```text
无法识别这个输入。请粘贴 BV/AV、视频、番剧、课程、收藏夹、合集或 UP 主空间链接。
```

## Normalized Result Tree

Every content type is normalized into the same tree model:

```text
Source
  Group
    Item
      Part
        Streams
        Assets
```

Meaning:

- `Source`: one raw user input.
- `Group`: video, collection, favorite, bangumi season, course, or uploader list group.
- `Item`: one video, episode, or course episode.
- `Part`: video part, episode part, or downloadable segment unit.
- `Streams`: available video/audio streams, quality, codec, and CDN URLs.
- `Assets`: cover, subtitles, danmaku, NFO, and other derived files.

Frontend pages must consume this model only. They should not special-case Bilibili business structures except for display labels.

## Selection Rules

The result tree uses tri-state selection:

- Source checkbox.
- Group checkbox.
- Item checkbox.
- Part checkbox.

Parent states:

- Fully selected.
- Partially selected.
- Not selected.

Default selection:

- Single video: selected.
- Multi-part normal video: all parts selected.
- Favorite and uploader lists: not selected by default.
- Bangumi, course, collection, and series: show parsed items and let the user choose.

The primary action is `下载已选择`. Do not use ambiguous labels like `下载全部`.

When a paged source has more remote items:

```text
已加载 30 / 1240
[加载更多] [解析全部]
```

`解析全部` requires confirmation and may accept a maximum item limit.

## Paging Rules

Paged parsing is progressive. The UI should allow the user to work with already loaded results while the backend continues loading more.

Page size rule:

```text
page_size = min(api_known_max_or_default, 100)
```

Examples from `bpi-rs` defaults:

- Favorite resources: `ps=20`.
- Uploader videos: `ps=30`.
- Collection and series endpoints: commonly `10` or `20`.

The command `下载已选择` only creates tasks for currently loaded and selected items.

## Download Options

The default mode is fast download:

- Download selected video and audio streams.
- Merge into one playable file.
- Do not fetch subtitles, danmaku, cover, or NFO unless requested.

Presets:

- `快速下载`: final merged video only.
- `完整归档`: video, cover, subtitles, danmaku, NFO, and optional embedding.
- `自定义`: user-controlled resources and post-processing.

Global defaults live in settings and are applied automatically after parsing:

- Quality: highest available, or a user-selected target.
- Audio quality: highest available, or a user-selected target.
- Codec: auto, HEVC, AVC, or AV1.
- Container: mp4 or mkv.
- Missing target quality: choose lower, skip, or ask.

Per-source, group, item, or part overrides are allowed.

## Account UX

Login is on demand, not required at startup.

Account button states:

- Logged out: show `登录`.
- Logged in: show avatar, display name, and VIP/status summary.

Single click opens the account dialog. The dropdown supports:

- View account state.
- Re-check login state.
- Import cookie.
- Logout.

Supported login methods:

- QR login.
- Cookie import.

Cookie must be persisted in system secure storage. SQLite stores only account summaries.

When logged out, parsing should show only streams currently available. Missing higher quality is not an error. The UI may show a light hint:

```text
未登录，登录后可能获得更高清晰度
```

Only private resources or expired login state should require explicit login action.

## Transfer Page

The transfer page is a downloader task manager, not a debug log viewer. Batch management comes first; selected-task details are secondary.

Filters:

- `活动`: waiting, parsing, downloading, muxing, paused, failed, and cancelled tasks.
- `失败`: failed and cancelled tasks that need user attention.
- `已完成`: completed transfer records.
- `全部`: all current transfer records.

The default filter should show work that still needs attention. If any active or failed task exists, default to `活动`; otherwise default to `已完成`, unless the user has manually selected another filter.

The task list should be a quasi-table with stable scanning columns:

```text
名称 | 状态 | 进度 | 速度 | 剩余 | 问题 | 位置 | 操作
```

Rules:

- Long source titles should be split into a scannable main title and secondary context. The full original title belongs in tooltip or inspector.
- The list should show a short save location, not the full output path.
- A failed task should show a short actionable issue in the `问题` column.
- Row actions are state-specific. Do not show every possible action on every row.
- Completed records stay under `已完成`; do not add a separate History page.

Clicking a task opens an inspector. The inspector is not a long detail page. It should prioritize:

1. Current status and recommended action.
2. Short diagnosis when a task failed.
3. Tabs for `诊断`, `概览`, `轨道`, `事件`, and `原始日志`.

Ordinary UI should prefer concrete media terms:

- `视频轨道`
- `音频轨道`
- `字幕`
- `封面`

Avoid exposing the abstract backend term `resource` in user-facing copy unless the context is explicitly technical.

Creating tasks from the parse page does not auto-navigate to the transfer page. Show a toast with a `查看传输` action and update the navigation badge.

## Task States

Task states:

- `waiting`
- `parsing`
- `downloading`
- `muxing`
- `completed`
- `failed`
- `paused`
- `cancelled`

Actions:

- Pause and resume.
- Cancel.
- Retry.
- Refresh links and retry.
- Remove.
- Open file.
- Open directory.
- Copy log.

Pause, cancel, and delete are distinct:

- Pause keeps partial files and queue state.
- Cancel stops the task, keeps the record, and asks whether to remove temporary files.
- Remove deletes the transfer record from the list and must not delete final output files unless the UI explicitly asks for file deletion.

## Completed Records

Completed downloads are reviewed from the Transfer page's `已完成` filter. They are not a separate primary navigation page unless the product later becomes a media library.

Saved fields:

- Original input.
- Title, cover, and source type.
- Download time.
- Final file path.
- Final task state.
- Selected quality, audio, codec, and container.
- Error summary.

Supported actions:

- Search by title, source URL, or save path.
- Open file.
- Open directory.
- Re-download.
- Copy source link.
- Remove completed record.
- Clear completed records.

## Settings Page

Settings categories:

- `下载`: default save directory, global concurrent tasks, per-resource segment count, retries, speed limit, startup recovery.
- `媒体`: quality, audio, codec, container, ffmpeg path, raw stream retention.
- `归档`: preset defaults, cover, subtitles, danmaku, NFO, naming templates, duplicate naming strategy.
- `高级`: proxy, log level, data directory, cache cleanup, temporary file cleanup, diagnostics export.

Account is intentionally not placed in settings.

## Naming Templates

Naming templates are supported in the first version, but no script engine is included.

Defaults:

```text
{title}/P{part_index} - {part_title}.{ext}
{series_title}/S{season_index}E{episode_index} - {episode_title}.{ext}
{collection_title}/{index} - {title}.{ext}
```

Supported variables:

- `title`
- `part_title`
- `part_index`
- `bvid`
- `aid`
- `cid`
- `owner_name`
- `owner_mid`
- `series_title`
- `season_index`
- `episode_index`
- `collection_title`
- `index`
- `quality`
- `codec`
- `date`
- `ext`

The app must sanitize file names, preview templates, and append suffixes such as `(1)` for duplicate paths.

## Error UX

Main UI errors should be short and actionable:

- Parse failed: input cannot be recognized.
- Login expired: sign in again.
- Private resource: login required.
- Download failed: CDN timeout after retries.
- Mux failed: ffmpeg not found.
- File failed: save directory is not writable.

Detailed logs are available from task details. Logs must redact cookies, sensitive headers, and long signed URLs.
