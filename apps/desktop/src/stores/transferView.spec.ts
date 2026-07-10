import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import type { DownloadTask } from '../api/dto'
import { createTaskDiagnosticView, createTransferTaskView } from './transferView'

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
    expect(diagnostic.impact).toContain('限速 2 MiB/s')
  })
})

describe('completed transfer task warnings', () => {
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
})
