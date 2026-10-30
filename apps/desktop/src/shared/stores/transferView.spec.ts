import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import type { DownloadTask } from '../api/dto'
import { createTaskDiagnosticView, createTransferTaskView, createTaskTimeline, redactDiagnosticMessage, redactLogMessage } from './transferView'
import failureCases from '../../../../../crates/bdl-core/tests/fixtures/failure-diagnostics.json'

const scheduledTask = (): DownloadTask => ({
  id: 'task:scheduled',
  title: '定时任务',
  source_id: 'video:scheduled',
  status: 'waiting',
  resources: [],
  output_path: 'downloads/scheduled.mp4',
  refresh_intent: null,
  media_selection: {
    video_quality: '80',
    audio_quality: '30280',
    video_codec: 'avc',
    container: 'mp4',
  },
  scheduled_at: '2026-07-10T13:30:00Z',
  speed_limit_bytes_per_second: null,
})

describe('failure diagnostics', () => {
  const task = { ...scheduledTask(), status: 'failed' as const, scheduled_at: null }
  const log = (message: string, created_at = '2026-07-10T12:00:00Z') => ({ task_id: task.id, level: 'error' as const, message, created_at })

  it('keeps all source diagnostics for copying while limiting displayed logs', () => {
    const message = `${'response cause '.repeat(400)}\nsource=3 cause=final-cause token=private-token\nCookie: SESSDATA=private-cookie`;
    const full = redactDiagnosticMessage(message);
    expect(full).toContain('source=3 cause=final-cause');
    expect(full).not.toContain('private-token');
    expect(full).not.toContain('private-cookie');
    expect(full).not.toContain('<truncated>');
    expect(redactLogMessage(message)).toContain('<truncated>');
  })

  it.each(failureCases)('classifies $message', ({ message, summary, action }) => {
    const view = createTransferTaskView(task, 0, [log(message)])
    const diagnostic = createTaskDiagnosticView(task, [log(message)])
    expect(view.issueLabel).toBe(summary)
    expect(diagnostic.recommendedAction).toBe(action === 'retry' ? 'retry' : action === 'refresh_urls_and_retry' ? 'refresh_retry' : action === 'login_then_retry' ? 'retry' : null)
  })

  it('uses the latest failure rather than combining unrelated attempts', () => {
    const logs = [log('Cookie 已失效'), log('下载请求失败: HTTP 403 Forbidden', '2026-07-10T12:02:00Z')]
    expect(createTransferTaskView(task, 0, logs).issueLabel).toBe('源站拒绝访问')
  })

  it('does not diagnose an earlier attempt after a new attempt has started', () => {
    const logs = [log('Cookie 已失效'), { ...log('开始下载任务', '2026-07-10T12:02:00Z'), level: 'info' as const }]
    expect(createTransferTaskView(task, 0, logs).issueLabel).toBe('任务失败')
    const sameTime = [{ ...logs[1] }, { ...logs[0], created_at: logs[1].created_at }]
    expect(createTransferTaskView(task, 0, sameTime).issueLabel).toBe('任务失败')
  })

  it('keeps deletion errors separate from a completed download', () => {
    const completed = { ...task, status: 'completed' as const }
    const logs = [{ ...log('[stage=delete_files code=file_in_use] 文件清理失败：os_error=32'), level: 'warning' as const }]
    expect(createTransferTaskView(completed, 100, logs).statusLabel).toBe('已完成')
    expect(createTaskDiagnosticView(completed, logs).tone).toBe('success')
    expect(createTaskTimeline(completed, logs)[0].title).toBe('文件清理失败：文件被占用')
  })
})

describe('scheduled transfer task view', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-07-10T12:00:00Z'))
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows a scheduled task without presenting it as an active download', () => {
    const view = createTransferTaskView(scheduledTask(), 0)

    expect(view.statusLabel).toBe('已定时')
    expect(view.primaryAction).toBe('unschedule')
    expect(view.primaryActionLabel).toBe('立即开始')
  })

  it('explains the schedule in task diagnostics', () => {
    const diagnostic = createTaskDiagnosticView(scheduledTask())

    expect(diagnostic.summary).toBe('任务已定时')
    expect(diagnostic.recommendedAction).toBe('unschedule')
  })

  it('exposes and explains a task-specific speed limit', () => {
    const task = scheduledTask()
    task.speed_limit_bytes_per_second = 2 * 1024 * 1024

    const view = createTransferTaskView(task, 0)
    const diagnostic = createTaskDiagnosticView(task)

    expect(view.secondaryActions.map((action) => action.kind)).toContain('speed_limit')
    expect(diagnostic.impact).toContain('限速 2.1 MB/s')
  })
})

describe('active transfer stages', () => {
  const activeTask = (intent: 'video' | 'audio', status: DownloadTask['status'] = 'downloading'): DownloadTask => ({
    ...scheduledTask(),
    status,
    scheduled_at: null,
    resources: [
      {
        id: `resource:${intent}`,
        kind: intent === 'video' ? 'video' : 'audio',
        intent,
        current_urls: ['https://example.com/media'],
        headers: [],
        status: 'downloading',
        target_path: `downloads/${intent}`,
        temp_path: `downloads/${intent}.part`,
      },
    ],
  })

  it('labels video, audio and muxing as separate stages', () => {
    expect(createTransferTaskView(activeTask('video'), 99, [], null, undefined, 99).statusLabel).toBe('下载视频中')
    expect(createTransferTaskView(activeTask('audio'), 85, [], null, undefined, 85).statusLabel).toBe('下载音频中')
    expect(createTransferTaskView(activeTask('video', 'muxing'), 99).statusLabel).toBe('合并中')
  })

  it('shows the current track percentage instead of the changing aggregate', () => {
    const view = createTransferTaskView(activeTask('audio'), 92, [], null, undefined, 35)

    expect(view.progressValue).toBe(92)
    expect(view.stageProgressLabel).toBe('35%')
  })
})

describe('completed transfer task warnings', () => {
  it('reports successful media recovery as completed while preserving genuine attachment failures', () => {
    const task = { ...scheduledTask(), status: 'completed' as const, scheduled_at: null }
    const logs = [
      { task_id: task.id, level: 'error' as const, message: 'FFmpeg 无法打开视频输入', created_at: '2026-07-10T12:00:00Z' },
      { task_id: task.id, level: 'info' as const, message: '重试任务', created_at: '2026-07-10T12:01:00Z' },
      { task_id: task.id, level: 'warning' as const, message: '已完成的媒体输入无效，重新下载该轨道', created_at: '2026-07-10T12:01:01Z' },
      { task_id: task.id, level: 'info' as const, message: '下载完成', created_at: '2026-07-10T12:02:00Z' },
    ]
    expect(createTransferTaskView(task, 100, logs).statusLabel).toBe('已完成')
    expect(createTaskDiagnosticView(task, logs).tone).toBe('success')
    const withAttachmentFailure = [...logs, { task_id: task.id, level: 'warning' as const, message: '字幕嵌入失败', created_at: '2026-07-10T12:01:30Z' }]
    expect(createTransferTaskView(task, 100, withAttachmentFailure).statusLabel).toBe('部分失败')
    expect(createTaskDiagnosticView(task, withAttachmentFailure).detail).toBe('字幕嵌入失败')
  })

  it('uses a compact status label when optional resources fail', () => {
    const task = scheduledTask()
    task.status = 'completed'
    task.scheduled_at = null
    task.resources = [
      {
        id: 'resource:cover',
        kind: 'asset',
        intent: 'cover',
        current_urls: [],
        headers: [],
        status: 'failed',
        target_path: 'downloads/cover.jpg',
        temp_path: 'downloads/cover.jpg.part',
      },
    ]

    const view = createTransferTaskView(task, 100, [
      {
        task_id: task.id,
        level: 'warning',
        message: '附加内容失败',
        created_at: '2026-07-10T12:01:00Z',
      },
    ])

    expect(view.statusLabel).toBe('部分失败')
    expect(view.issueLabel).toBe('-')
  })

  it('offers the source page in the completed task menu when identity is available', () => {
    const task = scheduledTask()
    task.status = 'completed'
    task.scheduled_at = null
    task.refresh_intent = { input: { kind: 'video_bvid', bvid: 'BV1xx411c7mD' }, cid: 1 }

    const view = createTransferTaskView(task, 100)

    expect(view.secondaryActions.map((action) => action.kind)).toContain('open_source')
  })

  it('does not present historical pause logs as an archive warning', () => {
    const task = scheduledTask()
    task.status = 'completed'
    task.scheduled_at = null
    const logs = [
      {
        task_id: task.id,
        level: 'warning' as const,
        message: '任务已暂停或取消',
        created_at: '2026-07-10T12:00:00Z',
      },
      {
        task_id: task.id,
        level: 'info' as const,
        message: '下载完成',
        created_at: '2026-07-10T12:01:00Z',
      },
    ]

    const view = createTransferTaskView(task, 100, logs)
    const diagnostic = createTaskDiagnosticView(task, logs)

    expect(view.statusLabel).toBe('已完成')
    expect(view.statusBadge).toBe('done')
    expect(view.issueLabel).toBe('-')
    expect(diagnostic.summary).toBe('任务已完成')
    expect(diagnostic.tone).toBe('success')
  })

  it('does not treat known missing optional assets as a partial failure', () => {
    const task = scheduledTask()
    task.status = 'completed'
    task.scheduled_at = null
    const logs = [
      {
        task_id: task.id,
        level: 'warning' as const,
        message: '跳过字幕：暂无可用地址',
        created_at: '2026-07-10T12:00:00Z',
      },
      {
        task_id: task.id,
        level: 'warning' as const,
        message: '跳过弹幕：暂无可用地址',
        created_at: '2026-07-10T12:00:01Z',
      },
    ]

    const view = createTransferTaskView(task, 100, logs)
    const diagnostic = createTaskDiagnosticView(task, logs)

    expect(view.statusLabel).toBe('已完成')
    expect(view.statusBadge).toBe('done')
    expect(diagnostic.summary).toBe('任务已完成')
    expect(diagnostic.tone).toBe('success')
  })

  it('does not show stale transfer speed after completion', () => {
    const task = scheduledTask()
    task.status = 'completed'
    task.scheduled_at = null

    const view = createTransferTaskView(task, 100, [], {
      downloadedBytes: 128 * 1024 * 1024,
      totalBytes: 128 * 1024 * 1024,
      speedBytesPerSecond: 13 * 1024 * 1024,
      updatedAt: Date.now(),
    })

    expect(view.speedLabel).toBe('--')
    expect(view.etaLabel).toBe('--')
    expect(view.sizeLabel).toBe('128 MB')
  })

  it('hides desktop output actions when the platform cannot open local output paths', () => {
    const task = scheduledTask()
    task.status = 'completed'
    task.scheduled_at = null

    const capabilities = { canOpenOutput: false }
    const view = createTransferTaskView(task, 100, [], null, capabilities)
    const diagnostic = createTaskDiagnosticView(task, [], capabilities)

    expect(view.primaryAction).toBe('none')
    expect(view.secondaryActions.map((action) => action.kind)).not.toContain('open_dir')
    expect(diagnostic.recommendedAction).toBeNull()
  })
})
