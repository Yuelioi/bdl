import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { DownloadTask } from '../api/dto'
import { useQueueStore } from './queue'

const api = vi.hoisted(() => ({ queueList: vi.fn(), queueRevision: vi.fn(), queueOutputSizes: vi.fn() }))
vi.mock('../api/tauri', () => api)

const task = (status: DownloadTask['status']): DownloadTask => ({
  id: 'task:mux',
  title: '合并任务',
  source_id: 'video:BV1',
  status,
  resources: [],
  output_path: 'downloads/video.mp4',
  refresh_intent: null,
  media_selection: { video_quality: 'best', audio_quality: 'best', video_codec: 'auto', container: 'mp4' },
  scheduled_at: null,
  speed_limit_bytes_per_second: null,
})

describe('queue reconciliation', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    api.queueList.mockReset()
    api.queueRevision.mockResolvedValue(1)
  })

  it('repairs a stale muxing view from durable completed state without manual refresh', async () => {
    api.queueList.mockResolvedValue([task('completed')])
    const queue = useQueueStore()
    queue.tasks = [task('muxing')]

    await queue.reconcile()

    expect(queue.tasks[0]?.status).toBe('completed')
    expect(queue.reconciling).toBe(false)
    await queue.reconcile()
    expect(api.queueList).toHaveBeenCalledTimes(1)
  })

  it('keeps completion time through events and replaces it when a retried task completes', async () => {
    const queue = useQueueStore()
    queue.tasks = [task('muxing')]
    queue.upsertTask(task('completed'))
    const completedAt = queue.tasks[0]?.completed_at
    expect(Number.isFinite(Date.parse(completedAt!))).toBe(true)
    queue.upsertTask(task('completed'))
    expect(queue.tasks[0]?.completed_at).toBe(completedAt)
    api.queueList.mockResolvedValue([{ ...task('completed'), completed_at: '2026-10-04T08:06:01Z' }])
    await queue.reconcile()
    queue.upsertTask(task('completed'))
    expect(queue.tasks[0]?.completed_at).toBe('2026-10-04T08:06:01Z')
    queue.upsertTask(task('waiting'))
    expect(queue.tasks[0]?.completed_at).toBeNull()
    queue.upsertTask(task('completed'))
    expect(queue.tasks[0]?.completed_at).not.toBe('2026-10-04T08:06:01Z')
  })
})

describe('queue stage progress', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('reports progress for the resource currently downloading', () => {
    const queue = useQueueStore()
    const downloading = task('downloading')
    downloading.resources = [
      {
        id: 'resource:video',
        kind: 'video',
        intent: 'video',
        current_urls: [],
        headers: [],
        status: 'completed',
        target_path: 'downloads/video.m4s',
        temp_path: 'downloads/video.m4s.part',
      },
      {
        id: 'resource:audio',
        kind: 'audio',
        intent: 'audio',
        current_urls: [],
        headers: [],
        status: 'downloading',
        target_path: 'downloads/audio.m4s',
        temp_path: 'downloads/audio.m4s.part',
      },
    ]
    queue.applyProgress({
      task_id: downloading.id,
      resource_id: 'resource:video',
      downloaded_bytes: 99,
      total_bytes: 100,
      created_at: '2026-09-25T00:00:00Z',
    })
    queue.applyProgress({
      task_id: downloading.id,
      resource_id: 'resource:audio',
      downloaded_bytes: 17,
      total_bytes: 20,
      created_at: '2026-09-25T00:00:01Z',
    })

    expect(queue.taskStageProgress(downloading)).toBe(85)
  })
})

describe('completed output sizes', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    api.queueOutputSizes.mockReset()
  })

  it('reads only requested completed outputs and caches missing files too', async () => {
    const queue = useQueueStore()
    queue.tasks = [task('completed'), { ...task('downloading'), id: 'active' }, { ...task('completed'), id: 'hidden' }]
    api.queueOutputSizes.mockResolvedValue({ 'task:mux': null })
    await queue.loadOutputSizes(['task:mux', 'active'])
    await queue.loadOutputSizes(['task:mux'])
    expect(api.queueOutputSizes).toHaveBeenCalledExactlyOnceWith(['task:mux'])
    expect(queue.outputSizesByTask['task:mux']).toEqual({ path: 'downloads/video.mp4', bytes: null })
    queue.upsertTask({ ...task('completed'), output_path: 'downloads/new.mp4' })
    api.queueOutputSizes.mockResolvedValue({ 'task:mux': 1024 })
    await queue.loadOutputSizes(['task:mux'])
    expect(queue.outputSizesByTask['task:mux']?.bytes).toBe(1024)
  })

  it('coalesces concurrent visible-range requests', async () => {
    const queue = useQueueStore()
    queue.tasks = [task('completed')]
    let resolve!: (value: Record<string, number>) => void
    api.queueOutputSizes.mockReturnValue(new Promise<Record<string, number>>((done) => { resolve = done }))
    const first = queue.loadOutputSizes(['task:mux'])
    await queue.loadOutputSizes(['task:mux'])
    expect(api.queueOutputSizes).toHaveBeenCalledTimes(1)
    resolve({ 'task:mux': 2048 })
    await first
    expect(queue.outputSizesLoading).toEqual([])
    expect(queue.outputSizesByTask['task:mux']?.bytes).toBe(2048)
  })
})
