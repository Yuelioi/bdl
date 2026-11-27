import type { DownloadTask, TaskStatus } from '../../api/dto';

export type TransferSortMode = 'queue' | 'completed_desc' | 'name_asc' | 'progress_desc' | 'speed_desc' | 'issue_first';

export interface TransferSortMetrics {
  progress(task: DownloadTask): number;
  speed(taskId: string): number;
}

const titleOrder = new Intl.Collator('zh-Hans-CN', { numeric: true, sensitivity: 'base' });

export const sortTransferTasks = (
  tasks: DownloadTask[],
  mode: TransferSortMode,
  metrics: TransferSortMetrics,
): DownloadTask[] => {
  // The store keeps the backend's insertion order; display newest entries first.
  if (mode === 'queue') return [...tasks].reverse();
  if (mode === 'name_asc') return [...tasks].sort((left, right) => titleOrder.compare(left.title, right.title));

  const score = mode === 'completed_desc' ? completionTime
    : mode === 'progress_desc' ? (task: DownloadTask) => metrics.progress(task)
      : mode === 'speed_desc' ? (task: DownloadTask) => metrics.speed(task.id)
        : (task: DownloadTask) => -issueRank(task.status);
  const ranked = tasks.map((task, index) => ({ task, score: score(task), index }));
  const tieOrder = mode === 'completed_desc' ? -1 : 1;
  ranked.sort((left, right) => right.score - left.score || tieOrder * (left.index - right.index));
  return ranked.map(({ task }) => task);
};

const completionTime = (task: DownloadTask): number => {
  const time = task.status === 'completed' && task.completed_at ? Date.parse(task.completed_at) : NaN;
  return Number.isFinite(time) ? time : 0;
};

export const matchesTransferSearch = (task: DownloadTask, query: string, sourceLabel: string): boolean => {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  return [task.title, task.source_id, sourceLabel, task.output_path].some((value) =>
    value.toLocaleLowerCase().includes(normalizedQuery),
  );
};

export const isPausable = (status: TaskStatus): boolean =>
  status === 'waiting' || status === 'parsing' || status === 'downloading' || status === 'mux_queued';

export const isCancellable = (status: TaskStatus): boolean => isPausable(status) || status === 'paused';

export const isRetryable = (status: TaskStatus): boolean =>
  status === 'failed' || status === 'cancelled' || status === 'completed';

const issueRank = (status: TaskStatus): number => {
  if (status === 'failed' || status === 'cancelled') return 0;
  if (status === 'paused') return 1;
  if (isPausable(status) || status === 'muxing') return 2;
  return 3;
};
