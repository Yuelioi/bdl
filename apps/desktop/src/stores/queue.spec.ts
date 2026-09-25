import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { DownloadTask } from '../api/dto'
import { useQueueStore } from './queue'

const api = vi.hoisted(() => ({ queueList: vi.fn() }))
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
  })

  it('repairs a stale muxing view from durable completed state without manual refresh', async () => {
    api.queueList.mockResolvedValue([task('completed')])
    const queue = useQueueStore()
    queue.tasks = [task('muxing')]

    await queue.reconcile()

    expect(queue.tasks[0]?.status).toBe('completed')
    expect(queue.reconciling).toBe(false)
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
