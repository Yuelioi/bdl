<script setup lang="ts">
import { computed, ref } from 'vue';
import MobileSheet from './MobileSheet.vue';
import UiButton from '../../ui/Button.vue';
import UiDialog from '../../ui/Dialog.vue';
import UiEmptyState from '../../ui/EmptyState.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import UiSelect from '../../ui/Select.vue';
import UiTabs from './MobileTabs.vue';
import UiTextField from '../../ui/TextField.vue';
import BulkActionBar from '../../ui/BulkActionBar.vue';
import TaskInspector from '../../ui/TaskInspector.vue';
import MobileTransferList from './MobileTransferList.vue';
import { useTransferPage } from '../useTransferPage';
const {
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
  speedLimitMib,
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
} = useTransferPage();
const managing = ref(false);
const toolsOpen = ref(false);
const searchOpen = ref(false);
const visibleSelectedCount = computed(() => {
  const visibleIds = new Set(taskViews.value.map((view) => view.id));
  return queue.selectedTaskIds.filter((id) => visibleIds.has(id)).length;
});
const toggleManaging = () => {
  if (managing.value) queue.setVisibleTaskSelection(queue.selectedTaskIds, false);
  managing.value = !managing.value;
};
const beginManaging = (taskId: string) => {
  managing.value = true;
  if (!queue.selectedTaskIds.includes(taskId)) queue.toggleTaskSelection(taskId);
};
</script>
<template>
  <section class="mobile-page transfer-page">
    <section class="transfer-main">
      <UiInlineNotice v-if="queue.notice" :tone="queue.notice.tone">
        {{ queue.notice.message }}
      </UiInlineNotice>

      <header class="transfer-toolbar">
        <h1 class="sr-only">传输</h1>
        <div class="transfer-header-actions">
          <button
            v-if="queue.activeFilter === 'completed'"
            type="button"
            class="mobile-icon-button"
            aria-label="搜索已完成任务"
            :aria-pressed="searchOpen"
            @click="searchOpen = !searchOpen"
          >
            <UIcon name="i-tabler-search" />
          </button>
          <button type="button" class="mobile-icon-button" aria-label="传输选项" @click="toolsOpen = true">
            <UIcon name="i-tabler-adjustments-horizontal" />
          </button>
          <button
            type="button"
            class="mobile-icon-button"
            :aria-label="managing ? '完成管理' : '管理任务'"
            @click="toggleManaging"
          >
            <UIcon :name="managing ? 'i-tabler-check' : 'i-tabler-list-check'" />
          </button>
        </div>
      </header>
      <UiTabs v-model="queueFilter" :tabs="tabs" />
      <label v-if="queue.activeFilter === 'completed' && searchOpen" class="transfer-search">
        <span class="sr-only">搜索已完成</span><UIcon name="i-tabler-search" />
        <input v-model="completedSearch" type="search" placeholder="搜索标题或保存位置" :disabled="queue.loading" />
        <button
          type="button"
          aria-label="关闭搜索"
          @click="
            completedSearch = '';
            searchOpen = false;
          "
        >
          <UIcon name="i-tabler-x" />
        </button>
      </label>
      <MobileTransferList
        v-if="taskViews.length"
        :views="taskViews"
        :selected-task-ids="queue.selectedTaskIds"
        :managing="managing"
        :loading="queue.loading"
        @inspect-task="openTaskDetail"
        @toggle-task-selection="queue.toggleTaskSelection"
        @long-press-task="beginManaging"
        @task-action="handleTaskAction"
      />
      <UiEmptyState
        v-else
        :title="emptyTitle"
        :description="emptyDescription || undefined"
        layout="stacked"
        compact
        embedded
      >
        <template v-if="queue.tasks.length === 0" #action>
          <UiButton variant="secondary" @click="ui.setTab('parse')">去解析</UiButton>
        </template>
      </UiEmptyState>

      <footer v-if="managing" class="transfer-footer">
        <button
          type="button"
          class="transfer-select-all"
          @click="
            queue.setVisibleTaskSelection(
              taskViews.map((view) => view.id),
              visibleSelectedCount < taskViews.length,
            )
          "
        >
          {{ visibleSelectedCount === taskViews.length ? '取消全选' : `全选 · ${visibleSelectedCount}` }}
        </button>
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

    <MobileSheet v-model="toolsOpen" title="传输选项">
      <UiSelect v-model="transferSort" label="列表排序" :options="transferSortOptions" />
      <UiButton
        variant="secondary"
        :disabled="queue.loading"
        @click="
          queue.list();
          toolsOpen = false;
        "
        >刷新任务</UiButton
      >
      <UiButton
        variant="ghost"
        :disabled="queue.loading || !completedTaskCount"
        @click="
          runClearCompleted();
          toolsOpen = false;
        "
        >清理已完成记录</UiButton
      >
    </MobileSheet>
  </section>
</template>

<style scoped src="./TransferPage.css"></style>
