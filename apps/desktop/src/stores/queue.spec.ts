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
