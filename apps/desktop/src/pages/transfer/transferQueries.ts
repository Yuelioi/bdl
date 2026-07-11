import type { DownloadTask, TaskStatus } from '../../api/dto';

export type TransferSortMode = 'queue' | 'name_asc' | 'progress_desc' | 'speed_desc' | 'issue_first';

export interface TransferSortMetrics {
  progress(task: DownloadTask): number;
  speed(taskId: string): number;
}

export const sortTransferTasks = (
  tasks: DownloadTask[],
  mode: TransferSortMode,
  metrics: TransferSortMetrics,
): DownloadTask[] => {
  if (mode === 'queue') return tasks;

  const indexed = tasks.map((task, index) => ({ task, index }));
  indexed.sort((left, right) => {
    if (mode === 'name_asc') {
      const byTitle = left.task.title.localeCompare(right.task.title, 'zh-Hans-CN', {
        numeric: true,
        sensitivity: 'base',
      });
      return byTitle || left.index - right.index;
    }
    if (mode === 'progress_desc') {
      return metrics.progress(right.task) - metrics.progress(left.task) || left.index - right.index;
    }
    if (mode === 'speed_desc') {
      return metrics.speed(right.task.id) - metrics.speed(left.task.id) || left.index - right.index;
    }
    return issueRank(left.task.status) - issueRank(right.task.status) || left.index - right.index;
  });
  return indexed.map((entry) => entry.task);
};

export const matchesTransferSearch = (task: DownloadTask, query: string, sourceLabel: string): boolean => {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  return [task.title, task.source_id, sourceLabel, task.output_path].some((value) =>
    value.toLocaleLowerCase().includes(normalizedQuery),
  );
};

export const isPausable = (status: TaskStatus): boolean =>
  status === 'waiting' || status === 'parsing' || status === 'downloading';

export const isCancellable = (status: TaskStatus): boolean => isPausable(status) || status === 'paused';

export const isRetryable = (status: TaskStatus): boolean =>
  status === 'failed' || status === 'cancelled' || status === 'completed';

const issueRank = (status: TaskStatus): number => {
  if (status === 'failed' || status === 'cancelled') return 0;
  if (status === 'paused') return 1;
  if (status === 'waiting' || status === 'parsing' || status === 'downloading' || status === 'muxing') return 2;
  return 3;
};
