# BDL Downloader UI Follow-ups

Status: Backlog open
Started: 2026-07-09

## Completed Current Pass

- Reduced routine toast usage; global toast is reserved for errors and severe failures.
- Kept parse, normalized selection, and transfer as separate workflows.
- Moved common actions next to their work surface, and folded advanced or rarely used content.
- Replaced the parse page side summary with a download settings dialog opened from `下载已选择`.
- Removed the dedicated source column; multiple parsed inputs now switch from the parse results panel.
- Added parse result search, visible-result sorting, visible-range selection, and `全选可见` behavior for large collections.
- Added Transfer list sorting by default order, name, progress, speed, and issue priority.
- Confirmed per-batch download overrides live in the `下载已选择` settings dialog: media mode, video quality, audio quality, codec, container, and archive mode.

## 



``



## Follow-up Backlog

1. Duplicate-task policy UI: skip, create anyway, or ask.
2. Transfer context menu/right-click parity for task actions.
3. Download directory health check with inline repair action.
4. Account asset entry points after downloader core stabilizes: favorites, watch later, history, follows.
