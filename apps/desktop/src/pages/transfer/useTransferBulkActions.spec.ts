import { ref } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import type { DownloadTask } from '../../api/dto';
import { useTransferBulkActions, type TransferBulkQueue } from './useTransferBulkActions';

const task = (id: string, status: DownloadTask['status']): DownloadTask => ({ id, status }) as DownloadTask;
const createQueue = (tasks: DownloadTask[], selectedTaskIds: string[] = []) =>
  ({
    tasks,
    selectedTaskIds,
    bulkPause: vi.fn(),
    bulkCancel: vi.fn(),
    bulkResume: vi.fn(),
    bulkRetry: vi.fn(),
    bulkRefreshUrlsAndRetry: vi.fn(),
    bulkRemove: vi.fn(),
    clearCompleted: vi.fn(),
  }) satisfies TransferBulkQueue;

describe('transfer bulk actions', () => {
  const tasks = [task('active', 'downloading'), task('paused', 'paused'), task('failed', 'failed')];

  it('uses visible tasks when nothing is selected', () => {
    const queue = createQueue(tasks);
    const bulk = useTransferBulkActions(queue, ref(tasks.slice(0, 2)));
    bulk.runBulkCancel();
    expect(queue.bulkCancel).toHaveBeenCalledWith(['active', 'paused']);
  });

  it('limits actions to selected tasks when a selection exists', () => {
    const queue = createQueue(tasks, ['failed']);
    const bulk = useTransferBulkActions(queue, ref(tasks));
    bulk.runBulkRetry();
    expect(queue.bulkRetry).toHaveBeenCalledWith(['failed']);
  });

  it('only removes explicitly selected tasks', () => {
    const queue = createQueue(tasks);
    const bulk = useTransferBulkActions(queue, ref(tasks));
    bulk.runBulkRemove();
    expect(queue.bulkRemove).toHaveBeenCalledWith([]);
  });
});
