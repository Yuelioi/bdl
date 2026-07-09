# BDL Downloader UI Follow-ups

Status: Backlog open
Started: 2026-07-09

## Completed Current Pass

- Reduced routine toast usage; global toast is reserved for errors and severe failures.
- Kept parse, normalized selection, and transfer as separate workflows.
- Moved common actions next to their work surface, and folded advanced or rarely used content.
- Replaced the parse page side summary with a download settings dialog opened from `下载已选择`.
- Removed the dedicated source column; multiple parsed inputs now switch from the parse results panel.

## 



``



## Follow-up Backlog

1. Parse result search and filtering for large collections.
2. Sort controls for parse results and transfer lists.
3. Range selection for parse results, for example `1-5,7,9-12`.
4. Per-batch quality/codec overrides if the backend exposes those task-level options.
5. Duplicate-task policy UI: skip, create anyway, or ask.
6. Transfer context menu/right-click parity for task actions.
7. Download directory health check with inline repair action.
8. Account asset entry points after downloader core stabilizes: favorites, watch later, history, follows.
