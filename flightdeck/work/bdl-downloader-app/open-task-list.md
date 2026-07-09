# BDL Downloader Open Task List

SUMMARY: Current unfinished and product-deficient work after auditing the BDL flightdeck knowledge and work plans.
READ WHEN: continuing BDL implementation, fixing Parse/Transfer/Settings UX, or deciding next development slices.

Audited on 2026-07-09 from:

- `flightdeck/knowledge/bdl-downloader/product-flow.md`
- `flightdeck/knowledge/bdl-downloader/architecture.md`
- `flightdeck/work/bdl-downloader-app/plan.md`
- `flightdeck/work/bdl-downloader-app/transfer-product-refactor-plan.md`
- `flightdeck/work/bdl-downloader-app/index.md`

## Product Corrections

- [x] Keep primary navigation to `解析`, `传输`, `设置`; do not restore a separate `历史` tab.
- [x] Treat completed records as a Transfer filter and workflow, not as an independent page.
- [x] Update old plan language that still mentions `HistoryPage.vue` or `历史` navigation so future work does not reintroduce it.
- [x] Keep account out of Settings; login remains owned by the top-right account button.

## P0 - Parse Page

- [x] Replace the current always-expanded repeated tree with a downloader-oriented selection tree.
- [x] Collapse redundant `group -> item -> part` rows when they have the same title.
- [x] For single videos, show the actual downloadable parts directly instead of repeating the video title three times.
- [x] For list sources, show one row per video when the item has one part; only expand into parts for true multi-part videos.
- [x] Add visible checkbox/tri-state selection affordances for selectable rows.
- [x] Add `全选已加载` and `清空选择` actions for the active source.
- [x] Show selected count per source in the source list.
- [x] Add multi-line input and text-file import for batch parsing.
- [x] Add `解析全部` confirmation and explicit maximum item limit copy before loading many pages.
- [x] Cap retained parse sources at the latest 20 and ask before clearing older results.
- [x] Add source-level retry/refresh action once `parse_refresh_source(source_id)` is implemented.

## P0 - Settings Page

- [x] Split Settings into clear sections for current supported settings.
- [x] Explain every currently exposed user-facing mode with concrete output behavior.
- [x] Rename archive preset copy so `快速下载` means "final merged video only".
- [x] Explain current `完整归档` support honestly: video + cover when available + generated NFO; subtitles/danmaku still pending.
- [x] Do not expose `自定义` until resource-level custom asset selection works end to end.
- [x] Show all supported naming variables beside the naming template field.
- [x] Provide naming template presets for single video, multi-part video, collection/series, and bangumi/course.
- [x] Replace the default `{title}/{title} - P{part_index} - {part_title}.{ext}` template with a non-duplicating default.
- [x] Validate naming templates before save and surface unknown variables inline.
- [x] Add duplicate-path naming strategy settings once backend supports user choice beyond automatic suffixes.
- [x] Add media defaults: quality, audio quality, codec, missing quality policy, ffmpeg path, raw stream retention.
- [x] Add advanced settings: proxy, log level, data directory, cache cleanup, temp cleanup, diagnostics export.

## P0 - Transfer Page

- [ ] Finish manual visual QA from the transfer refactor plan at 1365x768 and 1100x720.
- [ ] Verify there is no horizontal page scrollbar and inspector content does not force layout overflow.
- [x] Verify failed tasks default to `诊断` and raw logs stay behind `原始日志`.
- [x] Verify completed tasks have primary open-file/open-folder actions.
- [ ] Verify bulk retry, refresh-link retry, pause/resume, remove, and clear-completed workflows on real persisted tasks.
- [x] Add a visible Transfer badge/count when parse creates tasks.

## P1 - Archive Assets

- [ ] Fetch subtitles through `bpi-rs` when available.
- [ ] Fetch danmaku through `bpi-rs` when available.
- [ ] Add custom archive selection for cover, subtitles, danmaku, NFO, and raw stream retention.
- [ ] Add optional cover/subtitle embedding in ffmpeg post-processing when the container supports it.
- [ ] Record unavailable archive assets as user-facing warnings without claiming full archive success.

## P1 - Completed Records

- [ ] Build completed-record search inside Transfer `已完成`.
- [ ] Search by title, source URL/input, and save path.
- [ ] Support open file, open directory, re-download, copy source link, remove record, and clear completed records.
- [ ] Persist selected quality, audio, codec, container, final path, download time, and error summary for completed records.

## P1 - Downloader Reliability

- [ ] Store durable resource intent separately from current URLs, not only recover identity from task IDs.
- [ ] Use backup CDN URLs before full URL refresh.
- [ ] Verify CDN consistency with length, ETag, or last-modified before continuing a ranged download.
- [ ] Add real size and speed reporting instead of `--`.
- [ ] Add segment count setting once the fetcher honors it.
- [ ] Add startup auto-recovery setting and recovery prompt.

## P1 - Logs and Diagnostics

- [ ] Add task log retention policy: 30 days or latest 1000 task summaries.
- [ ] Add diagnostics export from Settings or Transfer inspector.
- [ ] Extend redaction coverage for cookies, authorization headers, signed URLs, and large response bodies.
- [ ] Add action-specific errors for unwritable save directory, missing ffmpeg, login expired, private resource, and parse recognition failure.

## P2 - Account and Storage Hardening

- [ ] Replace file-backed `.bdl/account.cookie` secure store with OS credential storage.
- [ ] Persist account summary in SQLite only, never plaintext cookie/session data.
- [ ] Verify account state at startup and emit `account://updated`.

## P2 - Packaging and QA

- [ ] Add `scripts/check.ps1` for fmt, clippy, tests, and frontend build.
- [ ] Add `scripts/package.ps1` for Tauri packaging and artifact path output.
- [ ] Complete minimum-window QA at 1100x720.
- [ ] Add release QA for parse, download, transfer recovery, persisted cookie, log redaction, and ffmpeg missing messages.

## Plan Maintenance

- [ ] Rewrite Task 15 around current product decisions: no separate History page, completed records live in Transfer.
- [ ] Mark older completed Task 1-3 plan checkboxes consistently or replace them with an audit summary to avoid false "unfinished" signals.
- [ ] Update `index.md` after each completed slice with commit IDs and verification commands.
