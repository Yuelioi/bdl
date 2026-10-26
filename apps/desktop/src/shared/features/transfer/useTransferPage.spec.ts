import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useTransferPage } from './useTransferPage';
import type { DownloadTask } from '../../api/dto';

vi.mock('./useTransferPageLifecycle', () => ({ useTransferPageLifecycle: vi.fn() }));
const api = vi.hoisted(() => ({ queueList: vi.fn(), queueDeletePreview: vi.fn() }));
vi.mock('../../api/tauri', () => api);

const task = (id: string, status: DownloadTask['status'] = 'waiting'): DownloadTask => ({
  id, title: id, status, source_id: `video:${id}`, output_path: `downloads/${id}.mkv`,
  resources: [], refresh_intent: null,
  media_selection: { video_quality: 'best', audio_quality: 'best', video_codec: 'auto', container: 'mkv' },
  scheduled_at: null, speed_limit_bytes_per_second: null,
});

describe('transfer page sorting defaults', () => {
  beforeEach(() => setActivePinia(createPinia()));

  it('uses newest completion first and remembers user choices separately from the active queue', () => {
    const page = useTransferPage();
    expect(page.transferSort.value).toBe('queue');
    expect(page.transferSortOptions.value[0]?.label).toBe('最新添加在前');
    page.queue.setFilter('completed');
    expect(page.transferSort.value).toBe('completed_desc');
    expect(page.transferSortOptions.value[0]?.label).toBe('最新完成在前');
    page.transferSort.value = 'name_asc';
    page.queue.setFilter('active');
    expect(page.transferSort.value).toBe('queue');
    page.transferSort.value = 'speed_desc';
    page.queue.setFilter('completed');
    expect(page.transferSort.value).toBe('name_asc');
    page.queue.setFilter('active');
    expect(page.transferSort.value).toBe('speed_desc');
  });

  it('keeps newest additions first through batch creation, status events, refresh and a fresh session', async () => {
    const page = useTransferPage();
    page.queue.setFilter('all');
    const first = task('first');
    const second = task('second', 'failed');
    const third = task('third', 'cancelled');
    page.queue.tasks = [first];
    page.queue.applyCreatedTasks([second, third]);
    expect(page.taskViews.value.map((view) => view.id)).toEqual(['third', 'second', 'first']);
    page.queue.upsertTask({ ...first, status: 'downloading' });
    expect(page.taskViews.value.map((view) => view.id)).toEqual(['third', 'second', 'first']);
    page.queue.applyBulkResult({ updated: [{ ...second, status: 'waiting' }], removed: [], failed: [] });
    expect(page.taskViews.value.map((view) => view.id)).toEqual(['third', 'second', 'first']);
    api.queueList.mockResolvedValue([first, second, third]);
    await page.queue.reconcile();
    expect(page.taskViews.value.map((view) => view.id)).toEqual(['third', 'second', 'first']);
    page.queue.setFilter('active');
    expect(page.taskViews.value.map((view) => view.id)).toEqual(['third', 'second', 'first']);
    page.queue.setFilter('failed');
    expect(page.taskViews.value.map((view) => view.id)).toEqual(['third', 'second']);

    setActivePinia(createPinia());
    const restored = useTransferPage();
    restored.queue.setFilter('all');
    await restored.queue.reconcile();
    expect(restored.taskViews.value.map((view) => view.id)).toEqual(['third', 'second', 'first']);
  });

  it('preserves unaffected row views through one progress event and an identical durable snapshot', async () => {
    const page = useTransferPage();
    page.queue.setFilter('all');
    page.queue.tasks = Array.from({ length: 500 }, (_, index) => task(`task-${index}`, 'downloading'));
    const before = new Map(page.taskViews.value.map(view => [view.id, view]));
    page.queue.applyProgress({ task_id: 'task-250', resource_id: 'video', downloaded_bytes: 50, total_bytes: 100, created_at: new Date().toISOString() });
    const after = page.taskViews.value;
    expect(after.filter(view => view !== before.get(view.id)).map(view => view.id)).toEqual(['task-250']);
    api.queueList.mockResolvedValue(JSON.parse(JSON.stringify(page.queue.tasks)));
    await page.queue.reconcile();
    const reconciled = new Map(page.taskViews.value.map(view => [view.id, view]));
    expect(after.every(view => reconciled.get(view.id) === view)).toBe(true);
  });
});
