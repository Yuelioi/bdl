import { ref } from 'vue';
import { computed, type ComputedRef } from 'vue';
import { sourceReference, useQueueStore, type QueueFilter } from '../../stores/queue';
import { createTransferTaskView } from '../../stores/transferView';
import { useUiStore } from '../../stores/ui';
import { matchesTransferSearch, sortTransferTasks, type TransferSortMode } from './transferQueries';
import { useTransferBulkActions } from './useTransferBulkActions';
import { useTransferDialogs } from './useTransferDialogs';
import { contextIcon, useTransferContextMenu } from './useTransferContextMenu';
import { useTransferPageLifecycle } from './useTransferPageLifecycle';
import { useTransferTaskDetail } from './useTransferTaskDetail';
import { useTransferTaskActions } from './useTransferTaskActions';
import { isAndroidPlatform, isMobilePlatform } from '../../platform/environment';
import { queueDeletePreview } from '../../api/tauri';
import { useFileRemoval } from './useFileRemoval';

export function useTransferPage() {
  const queue = useQueueStore();
  const ui = useUiStore();
  const removal = useFileRemoval('tasks', queueDeletePreview, (token) => queue.deleteFiles(token));
  const transferCapabilities = { canOpenOutput: !isMobilePlatform() || isAndroidPlatform() };
  const completedSearch = ref('');
  const transferSortByFilter = ref<Partial<Record<QueueFilter, TransferSortMode>>>({});
  const transferSort = computed({
    get: () => transferSortByFilter.value[queue.activeFilter] ?? (queue.activeFilter === 'completed' ? 'completed_desc' : 'queue'),
    set: (value: TransferSortMode) => { transferSortByFilter.value[queue.activeFilter] = value; },
  });
  const {
    taskDetailOpen,
    selectedLogs,
    selectedLogsLoading,
    selectedProgress,
    selectedDetailTitle,
    openTaskDetail,
    refreshSelectedLogs,
  } = useTransferTaskDetail(queue);
  const {
    scheduleDialogOpen,
    scheduleLocal,
    scheduleMin,
    scheduleError,
    speedLimitDialogOpen,
    speedLimitMb,
    speedLimitError,
    openScheduleDialog,
    submitSchedule,
    openSpeedLimitDialog,
    submitSpeedLimit,
  } = useTransferDialogs(queue);
  const { runTaskAction: handleTaskAction } = useTransferTaskActions(queue, {
    openScheduleDialog,
    openSpeedLimitDialog,
    openDeletionDialog: (taskId) => removal.show([taskId]),
  });
  const queueFilter = computed({
    get: () => queue.activeFilter,
    set: (value: QueueFilter) => queue.setFilter(value),
  });
  const tabs = computed<Array<{ label: string; value: QueueFilter; count: number }>>(() => [
    { label: '活动', value: 'active', count: queue.countByFilter('active') },
    { label: '失败', value: 'failed', count: queue.countByFilter('failed') },
    { label: '已完成', value: 'completed', count: queue.countByFilter('completed') },
    { label: '全部', value: 'all', count: queue.tasks.length },
  ]);
  const transferSortOptions = computed(() => [
    ...(queue.activeFilter === 'completed' ? [{ label: '最新完成在前', value: 'completed_desc' }] : []),
    { label: '最新添加在前', value: 'queue' },
    { label: '名称（升序）', value: 'name_asc' },
    { label: '进度（高到低）', value: 'progress_desc' },
    { label: '速度（快到慢）', value: 'speed_desc' },
    { label: '失败 / 取消在前', value: 'issue_first' },
  ]);
  const completedSearchQuery = computed(() => completedSearch.value.trim().toLowerCase());
  const visibleTasks = computed(() => {
    let tasks = queue.filteredTasks;
    if (queue.activeFilter === 'completed' && completedSearchQuery.value) {
      tasks = tasks.filter((task) =>
        matchesTransferSearch(task, completedSearchQuery.value, sourceReference(task.source_id)),
      );
    }

    return sortTransferTasks(tasks, transferSort.value, {
      progress: (task) => queue.taskProgress(task),
      speed: (taskId) => queue.taskTransferProgress(taskId)?.speedBytesPerSecond ?? 0,
    });
  });
  const rowViews = new Map<string, { task: (typeof queue.tasks)[number]; view: ComputedRef<ReturnType<typeof createTransferTaskView>> }>();
  const taskViews = computed(() => {
    const ids = new Set(queue.tasks.map(task => task.id));
    for (const id of rowViews.keys()) if (!ids.has(id)) rowViews.delete(id);
    return visibleTasks.value.map((task) => {
      let row = rowViews.get(task.id);
      if (!row || row.task !== task) {
        row = { task, view: computed(() => createTransferTaskView(
        task,
        queue.taskProgress(task),
        queue.logsByTask[task.id] ?? [],
        queue.taskTransferProgress(task.id),
        transferCapabilities,
        queue.taskStageProgress(task),
        queue.outputSizesByTask[task.id]?.path === task.output_path ? queue.outputSizesByTask[task.id]?.bytes ?? null : null,
        )) };
        rowViews.set(task.id, row);
      }
      return row.view.value;
    });
  });
  const {
    pausableTaskIds,
    cancellableTaskIds,
    resumableTaskIds,
    retryableTaskIds,
    removableTaskIds,
    runBulkPause,
    runBulkCancel,
    runBulkResume,
    runBulkRetry,
    runBulkRefreshRetry,
    runBulkRemove,
    runClearCompleted,
  } = useTransferBulkActions(queue, visibleTasks);
  const completedTaskCount = computed(() => queue.tasks.filter((task) => task.status === 'completed').length);
  const emptyTitle = computed(() => {
    if (queue.tasks.length === 0) {
      return '还没有传输任务';
    }
    if (queue.activeFilter === 'active') {
      return '没有活动任务';
    }
    if (queue.activeFilter === 'failed') {
      return '没有失败任务';
    }
    if (queue.activeFilter === 'completed') {
      if (completedSearchQuery.value) {
        return '没有匹配的完成记录';
      }
      return '还没有完成任务';
    }

    return '没有匹配任务';
  });
  const emptyDescription = computed(() => (queue.tasks.length === 0 ? '在解析页选择视频后，任务会出现在这里。' : ''));
  const {
    contextMenu,
    contextTaskView,
    contextActions,
    openContextMenu,
    closeContextMenu,
    closeContextMenuOnEscape,
    runContextAction,
  } = useTransferContextMenu(taskViews, handleTaskAction);
  useTransferPageLifecycle(queue, { closeContextMenu, closeContextMenuOnEscape });
  return {
    queue,
    ui,
    completedSearch,
    transferSort,
    taskDetailOpen,
    selectedLogs,
    selectedLogsLoading,
    selectedProgress,
    selectedDetailTitle,
    openTaskDetail,
    refreshSelectedLogs,
    scheduleDialogOpen,
    scheduleLocal,
    scheduleMin,
    scheduleError,
    speedLimitDialogOpen,
    speedLimitMb,
    speedLimitError,
    submitSchedule,
    submitSpeedLimit,
    handleTaskAction,
    queueFilter,
    tabs,
    transferSortOptions,
    taskViews,
    pausableTaskIds,
    cancellableTaskIds,
    resumableTaskIds,
    retryableTaskIds,
    removableTaskIds,
    runBulkPause,
    runBulkCancel,
    runBulkResume,
    runBulkRetry,
    runBulkRefreshRetry,
    runBulkRemove,
    runClearCompleted,
    completedTaskCount,
    emptyTitle,
    emptyDescription,
    contextMenu,
    contextTaskView,
    contextActions,
    openContextMenu,
    closeContextMenu,
    runContextAction,
    contextIcon,
    runBulkDeleteFiles: () => removal.show(removableTaskIds.value),
    removal,
  };
}
