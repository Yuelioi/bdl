<script setup lang="ts">
import { computed, ref } from 'vue';

import { sourceReference, useQueueStore, type QueueFilter } from '../stores/queue';
import { createTransferTaskView } from '../stores/transferView';
import { useUiStore } from '../stores/ui';
import UiButton from '../ui/Button.vue';
import UiDialog from '../ui/Dialog.vue';
import UiEmptyState from '../ui/EmptyState.vue';
import UiInlineNotice from '../ui/InlineNotice.vue';
import UiSelect from '../ui/Select.vue';
import UiTabs from '../ui/Tabs.vue';
import UiTextField from '../ui/TextField.vue';
import BulkActionBar from '../ui/BulkActionBar.vue';
import TaskInspector from '../ui/TaskInspector.vue';
import TransferTaskTable from '../ui/TransferTaskTable.vue';
import { matchesTransferSearch, sortTransferTasks, type TransferSortMode } from './transfer/transferQueries';
import { useTransferBulkActions } from './transfer/useTransferBulkActions';
import { useTransferDialogs } from './transfer/useTransferDialogs';
import { contextIcon, useTransferContextMenu } from './transfer/useTransferContextMenu';
import { useTransferPageLifecycle } from './transfer/useTransferPageLifecycle';
import { useTransferTaskDetail } from './transfer/useTransferTaskDetail';
import { useTransferTaskActions } from './transfer/useTransferTaskActions';
import { isAndroidPlatform, isMobilePlatform } from '../utils/platform';

const queue = useQueueStore();
const ui = useUiStore();
const transferCapabilities = { canOpenOutput: !isMobilePlatform() || isAndroidPlatform() };
const completedSearch = ref('');
const transferSort = ref<TransferSortMode>('queue');
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
  speedLimitMib,
  speedLimitError,
  openScheduleDialog,
  submitSchedule,
  openSpeedLimitDialog,
  submitSpeedLimit,
} = useTransferDialogs(queue);
const { runTaskAction: handleTaskAction } = useTransferTaskActions(queue, {
  openScheduleDialog,
  openSpeedLimitDialog,
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
const transferSortOptions = [
  { label: '默认顺序', value: 'queue' },
  { label: '名称 A-Z', value: 'name_asc' },
  { label: '进度高优先', value: 'progress_desc' },
  { label: '速度高优先', value: 'speed_desc' },
  { label: '问题优先', value: 'issue_first' },
];
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
const taskViews = computed(() =>
  visibleTasks.value.map((task) =>
    createTransferTaskView(
      task,
      queue.taskProgress(task),
      queue.logsByTask[task.id] ?? [],
      queue.taskTransferProgress(task.id),
      transferCapabilities,
    ),
  ),
);
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
</script>

<template>
  <section class="page-grid transfer-page">
    <section class="panel transfer-main">
      <UiInlineNotice v-if="queue.notice" :tone="queue.notice.tone">
        {{ queue.notice.message }}
      </UiInlineNotice>

      <div class="transfer-toolbar">
        <UiTabs v-model="queueFilter" :tabs="tabs" />
      </div>

      <div class="transfer-list-tools">
        <UiTextField
          v-if="queue.activeFilter === 'completed'"
          v-model="completedSearch"
          label="搜索已完成"
          placeholder="标题、来源或保存路径"
          :disabled="queue.loading"
        />
        <UiSelect v-model="transferSort" label="排序" :options="transferSortOptions" :disabled="queue.loading" />
      </div>

      <div v-if="taskViews.length" class="flex min-h-0 flex-1 flex-col gap-3 overflow-hidden">
        <TransferTaskTable
          :views="taskViews"
          :mode="queue.activeFilter === 'completed' ? 'completed' : 'transfer'"
          :selected-task-id="taskDetailOpen ? queue.selectedTaskId : null"
          :selected-task-ids="queue.selectedTaskIds"
          :loading="queue.loading"
          @inspect-task="openTaskDetail"
          @toggle-task-selection="queue.toggleTaskSelection"
          @toggle-visible-selection="queue.setVisibleTaskSelection"
          @task-action="handleTaskAction"
          @open-context-menu="openContextMenu"
        />
      </div>
      <UiEmptyState v-else :title="emptyTitle" :description="emptyDescription || undefined" layout="stacked" compact>
        <template v-if="queue.tasks.length === 0" #action>
          <UiButton variant="secondary" @click="ui.setTab('parse')">去解析</UiButton>
        </template>
      </UiEmptyState>

      <footer class="transfer-footer">
        <BulkActionBar
          :selected-count="queue.selectedTaskIds.length"
          :completed-count="completedTaskCount"
          :can-pause="pausableTaskIds.length > 0"
          :can-cancel="cancellableTaskIds.length > 0"
          :can-resume="resumableTaskIds.length > 0"
          :can-retry="retryableTaskIds.length > 0"
          :can-refresh-retry="retryableTaskIds.length > 0"
          :can-remove="removableTaskIds.length > 0"
          :loading="queue.loading"
          @pause="runBulkPause"
          @cancel="runBulkCancel"
          @resume="runBulkResume"
          @retry="runBulkRetry"
          @refresh-retry="runBulkRefreshRetry"
          @remove="runBulkRemove"
          @clear-completed="runClearCompleted"
          @refresh="queue.list"
        />
      </footer>
    </section>

    <UiDialog v-model="taskDetailOpen" :title="selectedDetailTitle" size="wide">
      <div class="task-detail-dialog">
        <TaskInspector
          :task="queue.selectedTask"
          :progress="selectedProgress"
          :logs="selectedLogs"
          :logs-loading="selectedLogsLoading"
          @refresh-logs="refreshSelectedLogs"
        />
      </div>
    </UiDialog>

    <UiDialog v-model="scheduleDialogOpen" title="设置开始时间">
      <UiTextField
        v-model="scheduleLocal"
        type="datetime-local"
        label="任务开始时间"
        :min="scheduleMin"
        :error="scheduleError"
        helper="到点后应用会自动把任务加入下载队列"
      />
      <template #footer>
        <UiButton variant="secondary" @click="scheduleDialogOpen = false">取消</UiButton>
        <UiButton :disabled="Boolean(scheduleError) || queue.loading" @click="submitSchedule">保存定时</UiButton>
      </template>
    </UiDialog>

    <UiDialog v-model="speedLimitDialogOpen" title="设置单任务限速">
      <UiTextField
        v-model="speedLimitMib"
        label="最大下载速度（MiB/s）"
        placeholder="留空时不单独限速"
        :error="speedLimitError ?? undefined"
        helper="留空时仅受全局限速影响；同一任务的所有分段共享此额度"
      />
      <template #footer>
        <UiButton variant="secondary" @click="speedLimitDialogOpen = false">取消</UiButton>
        <UiButton :disabled="Boolean(speedLimitError) || queue.loading" @click="submitSpeedLimit"> 保存限速 </UiButton>
      </template>
    </UiDialog>

    <Teleport to="body">
      <div v-if="contextMenu" class="context-menu-scrim" @pointerdown="closeContextMenu">
        <div
          class="task-context-menu"
          role="menu"
          :aria-label="`${contextTaskView?.displayTitle ?? '任务'}操作`"
          :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
          @pointerdown.stop
        >
          <div class="context-menu-heading">
            <span>任务操作</span>
            <strong>{{ contextTaskView?.displayTitle }}</strong>
          </div>
          <button
            v-for="action in contextActions"
            :key="action.kind"
            type="button"
            role="menuitem"
            :class="{ danger: action.tone === 'danger' }"
            @click="runContextAction(action.kind)"
          >
            <UIcon :name="contextIcon(action.icon)" aria-hidden="true" />
            {{ action.label }}
          </button>
        </div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped src="./TransferPage.css"></style>
