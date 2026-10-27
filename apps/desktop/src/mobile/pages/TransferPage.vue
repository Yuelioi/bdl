<script setup lang="ts">
import { computed, ref } from 'vue';
import MobileSheet from '../components/MobileSheet.vue';
import UiButton from '../../shared/ui/Button.vue';
import UiDialog from '../../shared/ui/Dialog.vue';
import UiEmptyState from '../../shared/ui/EmptyState.vue';
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import UiSelect from '../../shared/ui/Select.vue';
import UiTabs from '../components/MobileTabs.vue';
import UiTextField from '../../shared/ui/TextField.vue';
import BulkActionBar from '../../shared/ui/BulkActionBar.vue';
import TaskInspector from '../../shared/ui/TaskInspector.vue';
import MobileTransferList from '../components/MobileTransferList.vue';
import MobileListActions from '../components/MobileListActions.vue';
import MobileListHeader from '../components/MobileListHeader.vue';
import MobileListSearch from '../components/MobileListSearch.vue';
import { useMobileListSelection } from '../components/useMobileListSelection';
import { useTransferPage } from '../../shared/features/transfer/useTransferPage';
import FileRemovalDialog from '../../shared/features/transfer/FileRemovalDialog.vue';
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
  runBulkDeleteFiles,
  removal,
  runClearCompleted,
  completedTaskCount,
  emptyTitle,
  emptyDescription,
} = useTransferPage();
const { managing, toggleManaging, select } = useMobileListSelection(() =>
  queue.setVisibleTaskSelection(queue.selectedTaskIds, false),
);
const toolsOpen = ref(false);
const searchOpen = ref(false);
const visibleSelectedCount = computed(() => {
  const visibleIds = new Set(taskViews.value.map((view) => view.id));
  return queue.selectedTaskIds.filter((id) => visibleIds.has(id)).length;
});
const beginManaging = (taskId: string) => {
  select(() => {
    if (!queue.selectedTaskIds.includes(taskId)) queue.toggleTaskSelection(taskId);
  });
};
</script>
<template>
  <section class="mobile-page transfer-page">
    <section class="transfer-main">
      <UiInlineNotice v-if="queue.notice" :tone="queue.notice.tone">
        {{ queue.notice.message }}
      </UiInlineNotice>

      <MobileListHeader class="transfer-toolbar">
        <h2 class="mobile-transfer-title">传输</h2>
        <template #actions>
          <MobileListActions
            v-model:search-open="searchOpen"
            search-label="搜索已完成任务"
            :show-search="queue.activeFilter === 'completed'"
            options-label="传输选项"
            manage-label="管理任务"
            :managing="managing"
            @options="toolsOpen = true"
            @manage="toggleManaging"
          />
        </template>
      </MobileListHeader>
      <UiTabs v-model="queueFilter" :tabs="tabs" :stack-counts="false" />
      <MobileListSearch
        v-if="queue.activeFilter === 'completed'"
        v-model="completedSearch"
        v-model:open="searchOpen"
        label="搜索已完成"
        placeholder="搜索标题或保存位置"
        :disabled="queue.loading"
      />
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
          @delete-files="runBulkDeleteFiles"
          @clear-completed="runClearCompleted"
          @refresh="queue.list"
        />
      </footer>
    </section>

    <FileRemovalDialog :controller="removal" />
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
        v-model="speedLimitMb"
        label="最大下载速度（MB/s）"
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
