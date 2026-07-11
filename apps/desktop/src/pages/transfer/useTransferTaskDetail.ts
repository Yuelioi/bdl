import { computed, ref } from 'vue';

import type { DownloadTask, QueueLogEntry } from '../../api/dto';

export interface TransferTaskDetailQueue {
  selectedTaskId: string | null;
  selectedTask: DownloadTask | null;
  logsByTask: Record<string, QueueLogEntry[]>;
  logsLoadingByTask: Record<string, boolean>;
  selectTask(taskId: string): void;
  loadLogs(taskId: string): unknown;
  taskProgress(task: DownloadTask): number;
}

export const useTransferTaskDetail = (queue: TransferTaskDetailQueue) => {
  const taskDetailOpen = ref(false);
  const selectedLogs = computed(() => (queue.selectedTaskId ? (queue.logsByTask[queue.selectedTaskId] ?? []) : []));
  const selectedLogsLoading = computed(() =>
    queue.selectedTaskId ? Boolean(queue.logsLoadingByTask[queue.selectedTaskId]) : false,
  );
  const selectedProgress = computed(() => (queue.selectedTask ? queue.taskProgress(queue.selectedTask) : 0));
  const selectedDetailTitle = computed(() => queue.selectedTask?.title ?? '任务详情');

  const openTaskDetail = (taskId: string) => {
    queue.selectTask(taskId);
    taskDetailOpen.value = true;
  };
  const refreshSelectedLogs = () => {
    if (queue.selectedTaskId) void queue.loadLogs(queue.selectedTaskId);
  };

  return {
    taskDetailOpen,
    selectedLogs,
    selectedLogsLoading,
    selectedProgress,
    selectedDetailTitle,
    openTaskDetail,
    refreshSelectedLogs,
  };
};
