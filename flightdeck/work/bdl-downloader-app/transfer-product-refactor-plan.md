# Transfer Product Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the Transfer page from a prototype-style card/detail view into a mature downloader task manager optimized for batch task control, failure recovery, and diagnostics.

**Architecture:** Keep backend task state authoritative. The frontend should render task-manager view models derived from backend DTOs, while backend commands expose durable recovery actions, diagnostics, and eventually real concurrency settings. The Transfer page remains separate from Parse: it may refresh URLs for existing tasks, but it must not contain a new parse input flow.

**Tech Stack:** Rust 2024, Tauri 2 commands/events, Vue 3, TypeScript, Pinia, custom UI components, SQLite task/log persistence.

---

## Product Decisions From Grill

- The Transfer page is for batch download management first; single-task debugging is secondary.
- The main task list should become a quasi-table, not a card feed.
- The default filter set is `活动`, `失败`, `已完成`, `全部`.
- `活动` includes running, queued, paused, failed, and cancelled tasks. It is the default when there is anything to handle.
- Completed downloads stay in Transfer under `已完成`; do not reintroduce a separate History navigation tab.
- Do not add queue ordering controls, drag handles, priority fields, or queue-position UI.
- Keep Parse and Transfer separate. Transfer can run task recovery actions such as refreshing download URLs, but it must not accept arbitrary new input.
- Failure UX must show a short actionable reason in the list and a richer diagnosis in the inspector.
- Distinguish recovery actions: `重试`, `刷新链接并重试`, `登录后重试`, and merge/component recovery.
- Details should be an inspector: status and recommended action first, tabs for `诊断`, `概览`, `轨道`, `事件`, and `原始日志`.
- Ordinary UI should say `轨道` or concrete media terms (`视频`, `音频`, `字幕`, `封面`) instead of exposing the abstract `资源` concept.
- Raw logs remain available for open-source diagnostics, but default UI shows a user-facing event timeline.
- Batch selection and bulk actions are required for this to feel like a real downloader.
- Settings must only expose options that work end to end. Concurrency/retry settings should appear after backend support lands.

## Non-Goals

- No queue order editing.
- No separate History page.
- No parse input inside Transfer.
- No fake limit-speed UI until the download engine implements it.
- No component library; continue using the existing custom UI kit.

## Phase Gates

- Phase 1 is done when the Transfer list is a quasi-table with aligned columns, simplified filters, state-specific row actions, better title/path display, and correct empty states.
- Phase 2 is done when failed tasks show a short list reason, the inspector has a default `诊断` tab for failures, and raw logs move behind an advanced tab.
- Phase 3 is done when task recovery actions distinguish ordinary retry from URL refresh retry and diagnostics recommend the right action.
- Phase 4 is done when multi-select and bulk task actions work for common download workflows.
- Phase 5 is done when real concurrency/retry/auto-refresh settings are wired through backend state and reflected in the Transfer status strip.

## File Map

Frontend:

- Modify: `apps/desktop/src/pages/TransferPage.vue` - page layout, toolbar, filters, inspector host.
- Modify or replace: `apps/desktop/src/ui/TaskRow.vue` - retire card-row behavior or make it table-row compatible.
- Create: `apps/desktop/src/ui/TransferTaskTable.vue` - quasi-table task list with stable columns.
- Create: `apps/desktop/src/ui/TaskInspector.vue` - right-side inspector with status summary and tabs.
- Create: `apps/desktop/src/ui/TaskActionMenu.vue` - state-specific secondary actions.
- Create: `apps/desktop/src/ui/BulkActionBar.vue` - selection count and bulk actions.
- Create: `apps/desktop/src/stores/transferView.ts` - frontend-only view-model helpers for filters, short titles, short paths, issue text, and action labels.
- Modify: `apps/desktop/src/stores/queue.ts` - selected task IDs, new filters, bulk command wrappers, event loading.
- Modify: `apps/desktop/src/api/dto.ts` - add task issue/diagnostic DTOs after backend support exists.
- Modify: `apps/desktop/src/api/tauri.ts` - add typed wrappers for new commands.

Backend:

- Modify: `crates/bdl-core/src/queue.rs` - task diagnostics fields or helpers, action recommendation model.
- Create: `crates/bdl-core/src/diagnostics.rs` - classify common failures into short user-facing reasons and recommended recovery actions.
- Modify: `crates/bdl-core/src/storage.rs` - persist issue/event data if it becomes part of task snapshots.
- Modify: `crates/bdl-tauri/src/commands.rs` - commands for diagnostics, refresh-link retry, bulk actions, and settings.
- Modify: `crates/bdl-tauri/src/state.rs` - backend implementations for diagnostics and bulk queue operations.
- Modify: `crates/bdl-core/src/settings.rs` - concurrent task count, retry count, and auto-refresh setting only when backend honors them.

Tests and verification:

- Create: `crates/bdl-core/tests/diagnostics.rs`.
- Modify: `crates/bdl-tauri/src/state.rs` tests for recovery and bulk operations.
- Run: `cargo fmt --all --check`.
- Run: `cargo test -p bdl-core --test diagnostics`.
- Run: `cargo test -p bdl-tauri state`.
- Run: `cargo check -p bdl-desktop`.
- Run: `pnpm --dir apps/desktop build`.

---

### Phase 1: Quasi-Table Transfer List

**Files:**

- Modify: `apps/desktop/src/pages/TransferPage.vue`
- Create: `apps/desktop/src/ui/TransferTaskTable.vue`
- Modify: `apps/desktop/src/ui/TaskRow.vue`
- Create: `apps/desktop/src/stores/transferView.ts`
- Modify: `apps/desktop/src/stores/queue.ts`

- [x] **Step 1: Replace status-enum tabs with workflow filters**

Use these filters:

```text
活动 | 失败 | 已完成 | 全部
```

Filter mapping:

```ts
type QueueFilter = 'active' | 'failed' | 'completed' | 'all'

active = waiting | parsing | downloading | muxing | paused | failed | cancelled
failed = failed | cancelled
completed = completed
all = every task
```

Default rule:

```text
If any task is active or failed, default to 活动.
Otherwise default to 已完成.
Do not force-switch after the user manually picks another filter.
```

- [x] **Step 2: Add a frontend task view model helper**

Create `apps/desktop/src/stores/transferView.ts` with pure helpers:

```ts
export interface TransferTaskView {
  id: string
  displayTitle: string
  subtitle: string
  statusLabel: string
  progressLabel: string
  speedLabel: string
  etaLabel: string
  sizeLabel: string
  issueLabel: string
  shortLocation: string
  primaryAction: 'retry' | 'refresh_retry' | 'resume' | 'pause' | 'open_file' | 'none'
}
```

Rules:

- `displayTitle` should prefer part/episode title when it can be extracted; otherwise use current title.
- `subtitle` can show source/course/group title, truncated by CSS.
- `shortLocation` is the final directory name or configured root directory name, not the full output path.
- `issueLabel` is short; full errors stay in inspector.

- [x] **Step 3: Implement the quasi-table list**

Columns:

```text
[select] 名称 | 状态 | 进度 | 速度 | 剩余 | 问题 | 位置 | 操作
```

Column behavior:

- `名称` gets the most width and uses two-line title/subtitle.
- `状态`, `进度`, `速度`, `剩余`, and `问题` are aligned for scanning.
- `位置` displays a short location and full path in tooltip.
- `操作` shows only state-relevant primary action plus a secondary menu.

- [x] **Step 4: Retire noisy row metadata**

Remove row text shaped like:

```text
最佳 · 音频 · 自动 · -- · ETA -- · 已完成
```

Keep comparable fields in columns:

```text
进度 | 速度 | 剩余 | 大小
```

Show `--` for values the backend does not yet provide.

- [x] **Step 5: Add correct empty states**

Empty copy:

```text
No tasks at all:
还没有传输任务
在解析页选择视频后，任务会出现在这里。
[去解析]

Active filter empty:
没有活动任务

Failed filter empty:
没有失败任务

Completed filter empty:
还没有完成任务
```

- [ ] **Step 6: Verify Phase 1**

Run:

```powershell
pnpm --dir apps/desktop build
```

Manual QA:

- At 1365x768, the list has no horizontal scrollbar.
- With 15 tasks, failed tasks are visible and scannable.
- Completed tasks do not dominate the default view when failures exist.
- Long titles and long paths do not expand row height unpredictably.

Commit:

```powershell
git add apps/desktop/src
git commit -m "feat: refactor transfer list for batch management"
```

Progress note:

- `pnpm run build` in `apps/desktop` passes.
- `git diff --check` for the touched Transfer frontend files passes; Git only reports CRLF normalization warnings.
- Manual Tauri-window visual QA and the phase commit are still pending.

---

### Phase 2: Inspector and Diagnostics IA

**Files:**

- Create: `apps/desktop/src/ui/TaskInspector.vue`
- Modify: `apps/desktop/src/pages/TransferPage.vue`
- Modify: `apps/desktop/src/stores/transferView.ts`
- Create: `crates/bdl-core/src/diagnostics.rs`
- Modify: `crates/bdl-core/src/lib.rs`
- Test: `crates/bdl-core/tests/diagnostics.rs`

- [x] **Step 1: Add backend diagnostic classification tests**

Cover these mappings:

```text
HTTP 404 while fetching resource length -> 链接可能已过期 -> refresh_retry
HTTP 403 -> 权限或登录状态异常 -> login_or_refresh
ffmpeg not found -> 未找到 FFmpeg -> configure_ffmpeg
merge command failure -> 合并失败 -> inspect_raw_log
network timeout -> 网络超时 -> retry
```

- [x] **Step 2: Implement `diagnostics.rs`**

Expose:

```rust
pub enum RecommendedAction {
    Retry,
    RefreshUrlsAndRetry,
    LoginThenRetry,
    ConfigureFfmpeg,
    InspectRawLog,
}

pub struct TaskDiagnostic {
    pub summary: String,
    pub detail: String,
    pub recommended_action: RecommendedAction,
    pub failed_intent: Option<DownloadResourceIntent>,
}
```

- [x] **Step 3: Replace detail panel with inspector**

Inspector tabs:

```text
诊断 | 概览 | 轨道 | 事件 | 原始日志
```

Default tab:

- Failed or cancelled task: `诊断`.
- Running/completed task: `概览`.
- If the user manually changes tabs, do not keep auto-switching while they are inspecting.

- [x] **Step 4: Make `诊断` actionable**

For a failed task, show:

```text
音频轨道下载失败：链接可能已过期。
建议操作：刷新链接并重试。
影响范围：音频 1 个轨道失败，视频轨道已完成。
```

Do not show raw Rust error as the first thing users see.

Progress note:

- The frontend inspector is implemented in `apps/desktop/src/ui/TaskInspector.vue`.
- Failed/cancelled tasks default to `诊断`; normal/completed tasks default to `概览`.
- Backend diagnostic classification is implemented in `crates/bdl-core/src/diagnostics.rs` with coverage for 404, 403, FFmpeg missing, merge failure, and timeout.

- [x] **Step 5: Separate event timeline from raw logs**

`事件` tab:

```text
14:32 开始下载
14:32 视频轨道下载完成
14:33 音频轨道下载失败：链接可能已过期
```

`原始日志` tab:

- Full task logs.
- `复制诊断信息`.
- Redact cookies, signed URLs, and headers.

Progress note:

- `事件` and `原始日志` tabs exist.
- Raw log messages are displayed behind the advanced tab and pass through basic cookie/signed-URL redaction.
- `复制诊断信息` copies the task summary, diagnosis, events, and redacted raw logs.

- [ ] **Step 6: Verify Phase 2**

Run:

```powershell
cargo fmt --all --check
cargo test -p bdl-core --test diagnostics
pnpm --dir apps/desktop build
```

Manual QA:

- Failed task opens `诊断`.
- Completed task opens `概览`.
- Raw error text does not crowd the default view.
- Copy diagnostics does not include cookies or full signed URLs.

Commit:

```powershell
git add crates/bdl-core apps/desktop/src
git commit -m "feat: add transfer diagnostics inspector"
```

---

### Phase 3: Recovery Actions

**Files:**

- Modify: `crates/bdl-tauri/src/commands.rs`
- Modify: `crates/bdl-tauri/src/state.rs`
- Modify: `apps/desktop/src/api/tauri.ts`
- Modify: `apps/desktop/src/stores/queue.ts`
- Modify: `apps/desktop/src/ui/TaskActionMenu.vue`
- Modify: `apps/desktop/src/ui/TaskInspector.vue`

- [x] **Step 1: Add explicit recovery commands**

Commands:

```text
queue_retry
queue_refresh_urls_and_retry
queue_bulk_retry
queue_bulk_refresh_urls_and_retry
```

Semantics:

- `queue_retry`: retry without forcing URL refresh.
- `queue_refresh_urls_and_retry`: refresh Bilibili media URLs using stored task identity, then retry.
- If a task lacks enough identity for refresh, return an actionable error that tells the user to parse again.

Progress note:

- Single-task `queue_retry` and `queue_refresh_urls_and_retry` are split.
- `queue_retry` no longer refreshes URLs implicitly.
- Frontend `refresh_retry` now calls `queue_refresh_urls_and_retry`.
- Bulk retry and bulk refresh-retry commands are implemented.

- [x] **Step 2: Map recommended actions to UI**

Primary action labels:

```text
Retryable network error -> 重试
Expired URL / 404 / length fetch failed -> 刷新链接并重试
Login/permission issue -> 登录后重试
FFmpeg missing -> 配置 FFmpeg
Completed task -> 打开文件
```

Progress note:

- Expired URL / 404 failures can surface `刷新链接并重试` from the frontend classifier and call the explicit refresh command.
- Generic retry and completed open-file actions are mapped.
- Login/FFmpeg configuration actions still need dedicated UI/backend surfaces, but failed URL refresh, generic retry, and completed open-file actions are mapped.

- [x] **Step 3: Keep Parse separate**

Do not add an input box or new parse source creation to Transfer.

Allowed Transfer recovery:

```text
Use stored task source/part identity -> refresh media URLs -> retry failed resources.
```

Progress note:

- Transfer recovery uses stored task identity only. No parse input was added to Transfer.

- [ ] **Step 4: Verify Phase 3**

Run:

```powershell
cargo test -p bdl-tauri state
cargo check -p bdl-desktop
pnpm --dir apps/desktop build
```

Manual QA:

- A 404 failed task offers `刷新链接并重试`, not only generic `重试`.
- A normal cancelled task can still use ordinary retry.
- The parse page still does not auto-navigate after task creation.

Commit:

```powershell
git add crates/bdl-tauri apps/desktop/src
git commit -m "feat: add explicit transfer recovery actions"
```

Progress note:

- `cargo check -p bdl-desktop` passes.
- `pnpm run build` passes.
- Manual QA and commit remain pending.

---

### Phase 4: Batch Selection and Bulk Actions

**Files:**

- Modify: `apps/desktop/src/ui/TransferTaskTable.vue`
- Create: `apps/desktop/src/ui/BulkActionBar.vue`
- Modify: `apps/desktop/src/stores/queue.ts`
- Modify: `crates/bdl-tauri/src/commands.rs`
- Modify: `crates/bdl-tauri/src/state.rs`

- [x] **Step 1: Add multi-select state**

Store:

```ts
selectedTaskIds: string[]
```

Behavior:

- Checkbox per row.
- Header checkbox selects visible tasks in current filter.
- Changing filters preserves selection only for tasks still visible; otherwise keep hidden selections only if the bulk bar clearly says total selected count.

- [x] **Step 2: Add bulk bar**

When no selection:

```text
暂停全部 | 继续全部 | 清理已完成 | 刷新
```

When selected:

```text
已选择 N 个 | 暂停 | 继续 | 重试 | 刷新链接并重试 | 移除
```

- [x] **Step 3: Add backend bulk operations**

Bulk commands should return per-task results:

```ts
interface BulkQueueResult {
  updated: DownloadTask[]
  failed: Array<{ task_id: string; message: string }>
}
```

Do not fail the whole operation when one task cannot be changed.

- [x] **Step 4: Add clear-completed**

`清理已完成` removes completed tasks from the Transfer list. It must not delete final output files.

Progress note:

- `BulkActionBar` is wired into Transfer.
- Backend commands now include bulk pause, resume, retry, refresh-retry, remove, and clear-completed.
- `BulkQueueResult` returns updated, removed, and failed per-task outcomes.

- [ ] **Step 5: Verify Phase 4**

Run:

```powershell
cargo test -p bdl-tauri state
cargo check -p bdl-desktop
pnpm --dir apps/desktop build
```

Manual QA:

- Select all failed tasks and run refresh retry.
- Select completed tasks and remove them without deleting files.
- Bulk errors show which tasks failed.

Commit:

```powershell
git add crates/bdl-tauri apps/desktop/src
git commit -m "feat: add transfer bulk actions"
```

---

### Phase 5: Real Transfer Settings and Status Strip

**Files:**

- Modify: `crates/bdl-core/src/settings.rs`
- Modify: `crates/bdl-core/src/queue.rs`
- Modify: `crates/bdl-tauri/src/state.rs`
- Modify: `apps/desktop/src/stores/settings.ts`
- Modify: `apps/desktop/src/App.vue`
- Modify: `apps/desktop/src/pages/TransferPage.vue`

- [x] **Step 1: Add only backend-honored settings**

Settings:

```text
同时下载任务数: 1 | 2 | 3 | 5
失败自动重试次数: 0 | 1 | 3 | 5
链接过期时自动刷新: on/off
```

Do not add speed limit controls until the downloader enforces speed limits.

- [x] **Step 2: Honor concurrent task count in the queue worker**

The queue worker should start up to the configured number of active tasks and persist setting changes.

- [x] **Step 3: Add compact status strip**

Example:

```text
下载 2 · 队列 8 · 失败 1 · 速度 -- · 限速 未启用
```

If speed is not implemented, keep `速度 --` but do not invent fake values.

Progress note:

- Settings now include `concurrent_tasks`, `retry_count`, and `auto_refresh_expired_urls`.
- The queue worker starts up to the configured concurrent task count.
- Fetch retry count is passed into `ReqwestFetcher`.
- Expired URL errors can trigger one automatic URL refresh per task before final failure.
- Transfer status strip shows download, queue, failure, speed placeholder, and configured concurrency.

- [ ] **Step 4: Verify Phase 5**

Run:

```powershell
cargo test -p bdl-core --test queue_state
cargo test -p bdl-tauri state
cargo check -p bdl-desktop
pnpm --dir apps/desktop build
```

Manual QA:

- Changing concurrent task count affects how many tasks run.
- Retry count affects automatic retry behavior.
- Auto-refresh setting controls expired URL recovery.

Commit:

```powershell
git add crates/bdl-core crates/bdl-tauri apps/desktop/src
git commit -m "feat: add real transfer settings"
```

Progress note:

- `cargo test -p bdl-core --test diagnostics --test queue_state` passes.
- `cargo test -p bdl-tauri state` passes.
- `cargo check -p bdl-desktop` passes.
- `pnpm run build` in `apps/desktop` passes.
- Manual QA and commit remain pending.

---

## Verification Matrix

Run before claiming the refactor done:

```powershell
cargo fmt --all --check
cargo test -p bdl-core --test diagnostics --test queue_state
cargo test -p bdl-tauri state
cargo check -p bdl-desktop
pnpm --dir apps/desktop build
git diff --check
```

Manual UI checks:

- 1365x768 window: no horizontal page scroll.
- 1100x720 window: list remains usable and inspector can collapse or move below.
- Failed task: short issue visible in list; diagnosis visible in inspector without reading raw logs.
- Completed task: primary action is open file or open folder.
- Bulk retry and bulk remove do not require opening each task.
- Transfer page still contains no parse input and does not reintroduce a History nav item.

## Plan Self-Review

Spec coverage:

- Batch management is covered by Phases 1 and 4.
- Failure diagnostics and recovery are covered by Phases 2 and 3.
- Settings and global status are covered by Phase 5.
- Decisions not to implement queue ordering, separate history, fake speed limit, or parse input in Transfer are captured as non-goals.

Placeholder scan:

- No unresolved placeholder or open-ended implementation steps remain.
- Features without backend support are explicitly blocked from UI exposure until backend support exists.

Type consistency:

- `DownloadTask` remains the backend-owned task DTO.
- `TransferTaskView` is frontend-only presentation data.
- Recovery commands distinguish ordinary retry from URL-refresh retry.
