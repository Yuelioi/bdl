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
})
