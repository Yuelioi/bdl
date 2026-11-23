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
  }) satisfies TransferBulkQueue;

describe('transfer bulk actions', () => {
  const tasks = [task('active', 'downloading'), task('paused', 'paused'), task('failed', 'failed')];

  it('uses the whole queue when nothing is selected', () => {
    const queue = createQueue([...tasks, task('waiting', 'waiting')]);
    const bulk = useTransferBulkActions(queue);
    bulk.runBulkCancel();
    expect(queue.bulkCancel).toHaveBeenCalledWith(['active', 'paused', 'waiting']);
  });

  it('limits actions to selected tasks when a selection exists', () => {
    const queue = createQueue(tasks, ['failed']);
    const bulk = useTransferBulkActions(queue);
    bulk.runBulkRetry();
    expect(queue.bulkRetry).toHaveBeenCalledWith(['failed']);
  });

  it('only removes explicitly selected tasks', () => {
    const queue = createQueue([...tasks, task('completed', 'completed'), task('cancelled', 'cancelled')]);
    const bulk = useTransferBulkActions(queue);
    bulk.runBulkRemove();
    expect(queue.bulkRemove).toHaveBeenCalledWith([]);
    bulk.runClearCompleted();
    bulk.runClearCancelled();
    expect(queue.bulkRemove.mock.calls.slice(1)).toEqual([[['completed']], [['cancelled']]]);
  });
});
