import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useTransferPage } from './useTransferPage';
import type { DownloadTask } from '../../api/dto';

vi.mock('./useTransferPageLifecycle', () => ({ useTransferPageLifecycle: vi.fn() }));
const api = vi.hoisted(() => ({ queueList: vi.fn(), queueRevision: vi.fn().mockResolvedValue(1), queueDeletePreview: vi.fn() }));
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
    page.queue.toggleTaskSelection(first.id);
    page.queueFilter.value = 'completed';
    expect(page.queue.selectedTaskIds).toEqual([]);
    page.queueFilter.value = 'all';
    page.queue.applyCreatedTasks([second, third]);
    expect(page.taskRows.value.map((row) => row.id)).toEqual(['third', 'second', 'first']);
    page.queue.upsertTask({ ...first, status: 'downloading' });
    page.queue.applyCreatedTasks([first]);
    expect(page.queue.tasks.find(task => task.id === first.id)?.status).toBe('downloading');
    expect(page.taskRows.value.map((row) => row.id)).toEqual(['third', 'second', 'first']);
    page.queue.applyBulkResult({ updated: [{ ...second, status: 'waiting' }], removed: [], failed: [] });
    expect(page.taskRows.value.map((row) => row.id)).toEqual(['third', 'second', 'first']);
    api.queueList.mockResolvedValue([first, second, third]);
    await page.queue.reconcile();
    expect(page.taskRows.value.map((row) => row.id)).toEqual(['third', 'second', 'first']);
    page.queue.setFilter('active');
    expect(page.taskRows.value.map((row) => row.id)).toEqual(['second', 'first']);
    page.queue.setFilter('failed');
    expect(page.taskRows.value.map((row) => row.id)).toEqual(['second']);

    setActivePinia(createPinia());
    const restored = useTransferPage();
    restored.queue.setFilter('all');
    await restored.queue.reconcile();
    expect(restored.taskRows.value.map((row) => row.id)).toEqual(['third', 'second', 'first']);
  });

  it('preserves unaffected row views through one progress event and an identical durable snapshot', async () => {
    const page = useTransferPage();
    page.queue.setFilter('all');
    page.queue.tasks = Array.from({ length: 500 }, (_, index) => task(`task-${index}`, 'downloading'));
    const before = new Map(page.taskRows.value.map((row) => row.view.value).map(view => [view.id, view]));
    page.queue.applyProgress({ task_id: 'task-250', resource_id: 'video', downloaded_bytes: 50, total_bytes: 100, created_at: new Date().toISOString() });
    const after = page.taskRows.value.map((row) => row.view.value);
    expect(after.filter(view => view !== before.get(view.id)).map(view => view.id)).toEqual(['task-250']);
    api.queueList.mockResolvedValue(JSON.parse(JSON.stringify(page.queue.tasks)));
    await page.queue.reconcile();
    const reconciled = new Map(page.taskRows.value.map((row) => row.view.value).map(view => [view.id, view]));
    expect(after.every(view => reconciled.get(view.id) === view)).toBe(true);

    const paused = page.queue.tasks.slice(0, 400).map(task => ({ ...task, status: 'paused' as const }));
    page.queue.applyBulkResult({ updated: paused, removed: [], failed: [] });
    expect(page.queue.tasks.filter(task => task.status === 'paused')).toHaveLength(400);
    expect(page.taskRows.value.map((row) => row.view.value).filter(view => view === reconciled.get(view.id))).toHaveLength(100);

    const resumeSnapshot = new Map(page.queue.tasks.map(task => [task.id, task]));
    page.queue.upsertTask({ ...paused[0], status: 'downloading' });
    page.queue.applyBulkResult({ updated: [{ ...paused[0], status: 'waiting' }], removed: [], failed: [] }, resumeSnapshot);
    expect(page.queue.tasks[0]?.status).toBe('downloading');
  });
});
