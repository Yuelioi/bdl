import { describe, expect, it, vi } from 'vitest';

import type { DownloadTask } from '../../api/dto';
import { useTransferTaskDetail, type TransferTaskDetailQueue } from './useTransferTaskDetail';

const task = { id: 'task:one', title: '任务一' } as DownloadTask;
const createQueue = (): TransferTaskDetailQueue => ({
  selectedTaskId: null,
  selectedTask: null,
  logsByTask: {},
  logsLoadingByTask: {},
  selectTask: vi.fn(),
  loadLogs: vi.fn(),
  taskProgress: vi.fn(() => 42),
});

describe('transfer task detail', () => {
  it('selects the task before opening the inspector', () => {
    const queue = createQueue();
    const detail = useTransferTaskDetail(queue);
    detail.openTaskDetail('task:one');
    expect(queue.selectTask).toHaveBeenCalledWith('task:one');
    expect(detail.taskDetailOpen.value).toBe(true);
  });

  it('projects selected task title, progress and logs', () => {
    const queue = createQueue();
    queue.selectedTaskId = task.id;
    queue.selectedTask = task;
    queue.logsByTask[task.id] = [{ task_id: task.id, level: 'info', message: '完成', created_at: '' }];
    const detail = useTransferTaskDetail(queue);
    expect([detail.selectedDetailTitle.value, detail.selectedProgress.value, detail.selectedLogs.value.length]).toEqual(
      ['任务一', 42, 1],
    );
  });

  it('does not refresh logs without a selected task', () => {
    const queue = createQueue();
    useTransferTaskDetail(queue).refreshSelectedLogs();
    expect(queue.loadLogs).not.toHaveBeenCalled();
  });
});
