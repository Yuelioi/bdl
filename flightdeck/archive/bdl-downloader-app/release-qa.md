# BDL Release QA Checklist

SUMMARY: Manual and scripted release checks for BDL desktop builds.
READ WHEN: preparing a release build, validating installer artifacts, or checking downloader regressions before handing the app to users.

## Scope

Run this checklist against a packaged or Tauri-dev build after `scripts/check.ps1` passes.

Required scripted checks:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\check.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\package.ps1 -SkipCheck
```

Record the tested commit, app version, Windows version, and artifact path before manual QA.

## Environment

- Window size: start at `1100x720`, then optionally re-check at a larger desktop size.
- Save directory: use a throwaway directory outside the repo, or a gitignored temp directory.
- Account: test both logged-out and logged-in states when a valid Bilibili cookie is available.
- Network: use a small public BV for smoke testing, plus one multi-part/list source if available.
- FFmpeg: test the normal configured/system path first; then set an invalid custom FFmpeg path to check the missing-FFmpeg failure message.

## Parse

- [ ] Paste a plain BV ID.
  Expected: source appears in `来源`, results show selectable downloadable parts, and no duplicate title stack is shown for a single video.
- [ ] Paste a full Bilibili video URL.
  Expected: it resolves to the same source shape as the BV ID.
- [ ] Paste multiple lines or import a text file with multiple inputs.
  Expected: each input creates a retained parse source; retained sources do not exceed the current cap without a prompt.
- [ ] For a list source, click `解析更多` and `解析全部`.
  Expected: loaded count increases predictably, `解析全部` shows an explicit limit/confirmation, and selection state remains understandable.
- [ ] Click `下载已选择`.
  Expected: the app stays on `解析`, creates tasks, shows a success/error toast, and increments the `传输` badge without forcing navigation.

## Download And Transfer

- [ ] Start one selected public video download.
  Expected: Transfer `活动` shows the task, progress uses real bytes/speed when available, and detail opens without horizontal scrolling.
- [ ] Pause, resume, cancel, and retry a task.
  Expected: status changes are durable, resources are not duplicated, and failed/cancelled tasks expose `重试`.
- [ ] Use `刷新链接后重试` on an expired-link style failure when available.
  Expected: task logs show URL refresh before retry, and only the target task is affected.
- [ ] Complete a small task.
  Expected: final file/folder actions are visible, completed records appear under `已完成`, and re-download resets resources to pending.
- [ ] Restart with an unfinished task.
  Expected: unfinished tasks are recovered; by default they are paused and surfaced through the startup recovery prompt unless auto-recovery is enabled.

## Account Persistence

- [ ] Import Cookie from the top-right account dialog.
  Expected: account button changes from `未登录` to UID/name, and Settings does not show account controls.
- [ ] Restart the app.
  Expected: account summary is restored, startup verification emits `account://updated`, and the user is not asked to import Cookie again.
- [ ] Logout from the top-right menu and restart.
  Expected: account remains logged out, OS credential entry is cleared, and SQLite stores no raw Cookie/session data.

## Logs And Redaction

- [ ] Open a failed task detail and inspect `诊断`, `事件`, and `原始日志`.
  Expected: common failure causes are classified before raw logs; raw logs are not the default tab for failed tasks.
- [ ] Export diagnostics from Transfer inspector and Settings.
  Expected: exported JSON contains app/task/settings context but redacts Cookie, Authorization, signed URL query strings, and oversized bodies.
- [ ] Search completed records with a warning/error summary.
  Expected: warning/error summaries are useful but do not expose secrets.

## FFmpeg Missing

- [ ] Set FFmpeg path to an invalid executable path.
  Expected: affected tasks fail with `未找到 FFmpeg`, detail recommends checking settings, and the command error code maps to `missing_ffmpeg`.
- [ ] Restore FFmpeg path to empty/system or a valid configured path.
  Expected: muxing succeeds again without needing to recreate parse sources.

## Minimum Window

- [ ] At `1100x720`, inspect `解析`, `传输`, and `设置`.
  Expected: no page-level horizontal scrollbar, no incoherent overlap, and primary actions remain visible.
- [ ] In Transfer, select failed and completed tasks when available.
  Expected: inspector content uses tabs/sections without forcing page overflow.

## Evidence

For each release candidate, append a short note to the release issue or handoff with:

- Commit hash.
- Artifact paths printed by `scripts/package.ps1`.
- QA date and tester.
- Public test inputs used, without private Cookie values.
- Any failures, screenshots, or waived risks.
