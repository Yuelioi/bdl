import { computed } from 'vue';

import type { DownloadTask } from '../../api/dto';
import { isCancellable, isPausable, isRetryable } from './transferQueries';

export interface TransferBulkQueue {
  tasks: DownloadTask[];
  selectedTaskIds: string[];
  bulkPause(taskIds: string[]): unknown;
  bulkCancel(taskIds: string[]): unknown;
  bulkResume(taskIds: string[]): unknown;
  bulkRetry(taskIds: string[]): unknown;
  bulkRefreshUrlsAndRetry(taskIds: string[]): unknown;
  bulkRemove(taskIds: string[]): unknown;
}

export const useTransferBulkActions = (queue: TransferBulkQueue) => {
  const selectedTaskSet = computed(() => new Set(queue.selectedTaskIds));
  const selectedTasks = computed(() => queue.tasks.filter((task) => selectedTaskSet.value.has(task.id)));
  const bulkScopeTasks = computed(() => (selectedTasks.value.length > 0 ? selectedTasks.value : queue.tasks));
  const pausableTaskIds = computed(() =>
    bulkScopeTasks.value.filter((task) => isPausable(task.status)).map((task) => task.id),
  );
  const cancellableTaskIds = computed(() =>
    bulkScopeTasks.value.filter((task) => isCancellable(task.status)).map((task) => task.id),
  );
  const resumableTaskIds = computed(() =>
    bulkScopeTasks.value.filter((task) => task.status === 'paused').map((task) => task.id),
  );
  const retryableTaskIds = computed(() =>
    selectedTasks.value.filter((task) => task.status !== 'completed' && isRetryable(task.status)).map((task) => task.id),
  );
  const redownloadTaskIds = computed(() => selectedTasks.value.filter(task => task.status === 'completed').map(task => task.id));
  const removableTaskIds = computed(() => selectedTasks.value.map((task) => task.id));
  const completedTaskIds = computed(() => queue.tasks.filter((task) => task.status === 'completed').map((task) => task.id));
  const cancelledTaskIds = computed(() => queue.tasks.filter((task) => task.status === 'cancelled').map((task) => task.id));

  return {
    selectedTasks,
    pausableTaskIds,
    cancellableTaskIds,
    resumableTaskIds,
    retryableTaskIds,
    redownloadTaskIds,
    removableTaskIds,
    runBulkPause: () => void queue.bulkPause(pausableTaskIds.value),
    runBulkCancel: () => void queue.bulkCancel(cancellableTaskIds.value),
    runBulkResume: () => void queue.bulkResume(resumableTaskIds.value),
    runBulkRetry: () => void queue.bulkRetry(retryableTaskIds.value),
    runBulkRedownload: () => void queue.bulkRetry(redownloadTaskIds.value),
    runBulkRefreshRetry: () => void queue.bulkRefreshUrlsAndRetry(retryableTaskIds.value),
    runBulkRemove: () => void queue.bulkRemove(removableTaskIds.value),
    completedTaskIds,
    cancelledTaskIds,
    runClearCompleted: () => void queue.bulkRemove(completedTaskIds.value),
    runClearCancelled: () => void queue.bulkRemove(cancelledTaskIds.value),
  };
};
