import { ref } from 'vue';
import { computed, type ComputedRef } from 'vue';
import { sourceReference, useQueueStore, type QueueFilter } from '../../stores/queue';
import { createTransferTaskView } from '../../stores/transferView';
import { useUiStore } from '../../stores/ui';
import { matchesTransferSearch, sortTransferTasks, type TransferSortMode } from './transferQueries';
import { useTransferBulkActions } from './useTransferBulkActions';
import { contextIcon, useTransferContextMenu } from './useTransferContextMenu';
import { useTransferPageLifecycle } from './useTransferPageLifecycle';
import { useTransferTaskDetail } from './useTransferTaskDetail';
import { useTransferTaskActions } from './useTransferTaskActions';
import { isAndroidPlatform, isMobilePlatform } from '../../platform/environment';
import { queueDeletePreview } from '../../api/tauri';
import { useFileRemoval } from './useFileRemoval';

export interface TransferTaskRow {
  id: string;
  view: ComputedRef<ReturnType<typeof createTransferTaskView>>;
}

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
  const { runTaskAction: handleTaskAction } = useTransferTaskActions(queue, {
    openDeletionDialog: (taskId) => removal.show([taskId]),
  });
  const queueFilter = computed({
    get: () => queue.activeFilter,
    set: (value: QueueFilter) => {
      if (value !== queue.activeFilter) queue.setVisibleTaskSelection(queue.selectedTaskIds, false);
      queue.setFilter(value);
    },
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
  const rowViews = new WeakMap<(typeof queue.tasks)[number], TransferTaskRow>();
  const taskRows = computed(() => visibleTasks.value.map((task) => {
    let row = rowViews.get(task);
    if (!row) {
      row = {
        id: task.id,
        view: computed(() => createTransferTaskView(
          task,
          queue.taskProgress(task),
          queue.logsByTask[task.id] ?? [],
          queue.taskTransferProgress(task.id),
          transferCapabilities,
          queue.taskStageProgress(task),
          queue.outputSizesByTask[task.id]?.path === task.output_path ? queue.outputSizesByTask[task.id]?.bytes ?? null : null,
        )),
      };
      rowViews.set(task, row);
    }
    return row;
  }));
  const {
    pausableTaskIds,
    cancellableTaskIds,
    resumableTaskIds,
    retryableTaskIds,
    redownloadTaskIds,
    removableTaskIds,
    runBulkPause,
    runBulkCancel,
    runBulkResume,
    runBulkRetry,
    runBulkRedownload,
    runBulkRefreshRetry,
    runBulkRemove,
    runClearCompleted,
    runClearCancelled,
    completedTaskIds,
    cancelledTaskIds,
  } = useTransferBulkActions(queue);
  const runBulkDeleteFiles = () => removal.show(removableTaskIds.value);
  const batchItems = computed(() => {
    const selected = removableTaskIds.value.length > 0;
    const controls = [
      {
        label: selected ? '暂停所选' : '暂停全部任务',
        icon: 'i-tabler-player-pause',
        available: pausableTaskIds.value.length > 0,
        onSelect: runBulkPause,
      },
      {
        label: selected ? '取消所选任务' : '取消全部任务',
        icon: 'i-tabler-x',
        available: cancellableTaskIds.value.length > 0,
        onSelect: runBulkCancel,
      },
      {
        label: selected ? '继续所选' : '继续全部任务',
        icon: 'i-tabler-player-play',
        available: resumableTaskIds.value.length > 0,
        onSelect: runBulkResume,
      },
      {
        label: '重试所选',
        icon: 'i-tabler-refresh',
        available: retryableTaskIds.value.length > 0,
        onSelect: runBulkRetry,
      },
      {
        label: '刷新链接并重试',
        icon: 'i-tabler-link',
        available: retryableTaskIds.value.length > 0,
        onSelect: runBulkRefreshRetry,
      },
      {
        label: '重新下载所选',
        icon: 'i-tabler-download',
        available: redownloadTaskIds.value.length > 0,
        onSelect: runBulkRedownload,
      },
    ].filter(item => item.available);
    const cleanup = selected
      ? [
          { label: '仅删除所选记录', icon: 'i-tabler-file-minus', onSelect: runBulkRemove },
          { label: '删除所选任务及文件', icon: 'i-tabler-trash', color: 'error' as const, onSelect: runBulkDeleteFiles },
        ]
      : [
          {
            label: '仅清除已完成记录',
            icon: 'i-tabler-file-minus',
            available: completedTaskIds.value.length > 0,
            onSelect: runClearCompleted,
          },
          {
            label: '仅清除已取消记录',
            icon: 'i-tabler-file-minus',
            available: cancelledTaskIds.value.length > 0,
            onSelect: runClearCancelled,
          },
        ].filter(item => item.available);
    return [controls, cleanup].filter(group => group.length > 0);
  });
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
  } = useTransferContextMenu(
    (id) => taskRows.value.find((row) => row.id === id)?.view.value,
    handleTaskAction,
  );
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
    handleTaskAction,
    queueFilter,
    tabs,
    transferSortOptions,
    taskRows,
    batchItems,
    emptyTitle,
    emptyDescription,
    contextMenu,
    contextTaskView,
    contextActions,
    openContextMenu,
    closeContextMenu,
    runContextAction,
    contextIcon,
    removal,
  };
}
