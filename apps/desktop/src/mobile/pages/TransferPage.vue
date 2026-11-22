<script setup lang="ts">
import { computed, ref } from 'vue';
import MobileSheet from '../components/MobileSheet.vue';
import UiButton from '../../shared/ui/Button.vue';
import UiDialog from '../../shared/ui/Dialog.vue';
import UiEmptyState from '../../shared/ui/EmptyState.vue';
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import UiSelect from '../../shared/ui/Select.vue';
import UiTabs from '../components/MobileTabs.vue';
import TaskInspector from '../../shared/ui/TaskInspector.vue';
import MobileTransferList from '../components/MobileTransferList.vue';
import MobileListActions from '../components/MobileListActions.vue';
import MobileListHeader from '../components/MobileListHeader.vue';
import MobileListSearch from '../components/MobileListSearch.vue';
import { useMobileListSelection } from '../components/useMobileListSelection';
import { useMobileBack } from '../app/useMobileBack';
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
const searching = ref(false);
const searchOpen = computed({
  get: () => searching.value,
  set: (open: boolean) => {
    searching.value = open;
    if (!open) completedSearch.value = '';
  },
});
useMobileBack('transfer', () => {
  if (managing.value) {
    toggleManaging();
    return true;
  }
  if (!searchOpen.value) return false;
  searchOpen.value = false;
  return true;
});
const visibleSelectedCount = computed(() => {
  const visibleIds = new Set(taskViews.value.map((view) => view.id));
  return queue.selectedTaskIds.filter((id) => visibleIds.has(id)).length;
});
const allVisibleSelected = computed(() => taskViews.value.length > 0 && visibleSelectedCount.value === taskViews.value.length);
const batchItems = computed(() => {
  const selected = queue.selectedTaskIds.length > 0;
  const taskActions = [
    {
      label: selected ? '暂停' : '暂停当前列表',
      icon: 'i-tabler-player-pause',
      available: pausableTaskIds.value.length > 0,
      onSelect: runBulkPause,
    },
    {
      label: selected ? '继续' : '继续当前列表',
      icon: 'i-tabler-player-play',
      available: resumableTaskIds.value.length > 0,
      onSelect: runBulkResume,
    },
    {
      label: '重试',
      icon: 'i-tabler-refresh',
      available: selected && retryableTaskIds.value.length > 0,
      onSelect: runBulkRetry,
    },
    {
      label: '刷新链接并重试',
      icon: 'i-tabler-link',
      available: selected && retryableTaskIds.value.length > 0,
      onSelect: runBulkRefreshRetry,
    },
    {
      label: selected ? '取消下载' : '取消当前列表的下载',
      icon: 'i-tabler-x',
      available: cancellableTaskIds.value.length > 0,
      onSelect: runBulkCancel,
    },
  ].filter((item) => item.available);
  const removalActions = removableTaskIds.value.length
    ? [
        { label: '仅删除任务记录', icon: 'i-tabler-file-minus', onSelect: runBulkRemove },
        { label: '删除任务及文件', icon: 'i-tabler-trash', color: 'error' as const, onSelect: runBulkDeleteFiles },
      ]
    : [];
  const completedActions = !selected && queue.activeFilter === 'completed' && completedTaskCount.value
    ? [{ label: '移除全部完成记录，保留文件', icon: 'i-tabler-file-minus', onSelect: runClearCompleted }]
    : [];
  return [taskActions, removalActions, completedActions].filter((group) => group.length > 0);
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
          <div class="transfer-toolbar-actions">
            <MobileListActions
              v-if="!managing"
              v-model:search-open="searchOpen"
              search-label="搜索已完成任务"
              :show-search="queue.activeFilter === 'completed'"
            />
            <UiButton v-if="!managing" variant="ghost" class="transfer-sort-button" @click="toolsOpen = true">
              <UIcon name="i-tabler-arrows-sort" class="size-4" aria-hidden="true" />
              排序
            </UiButton>
            <UiButton
              variant="ghost"
              class="transfer-mode-button"
              :aria-label="managing ? '完成管理' : '管理任务'"
              @click="toggleManaging"
            >{{ managing ? '完成' : '管理' }}</UiButton>
          </div>
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
        <UiButton
          variant="ghost"
          class="transfer-select-all"
          :disabled="queue.loading || !taskViews.length"
          @click="
            queue.setVisibleTaskSelection(
              taskViews.map((view) => view.id),
              !allVisibleSelected,
            )
          "
        >{{ allVisibleSelected ? '取消全选' : '全选' }}</UiButton>
        <span class="transfer-selection-count" aria-live="polite">已选 {{ queue.selectedTaskIds.length }} 项</span>
        <UDropdownMenu
          :items="batchItems"
          :content="{ align: 'end', side: 'top', sideOffset: 6, collisionPadding: 8 }"
          :ui="{ item: 'min-h-11 [@media(max-height:400px)]:min-h-9' }"
        >
          <UiButton
            variant="secondary"
            class="transfer-batch-button"
            aria-label="更多批量操作"
            :disabled="queue.loading || !batchItems.length"
          >批量操作 <UIcon name="i-tabler-chevron-up" class="size-4" /></UiButton>
        </UDropdownMenu>
      </footer>
    </section>

    <FileRemovalDialog :controller="removal" />
    <UiDialog v-model="taskDetailOpen" :title="selectedDetailTitle" size="wide">
      <TaskInspector
        :task="queue.selectedTask"
        :progress="selectedProgress"
        :logs="selectedLogs"
        :logs-loading="selectedLogsLoading"
        @refresh-logs="refreshSelectedLogs"
      />
    </UiDialog>
    <MobileSheet v-model="toolsOpen" title="排序与刷新">
      <UiSelect v-model="transferSort" label="列表排序" :options="transferSortOptions" />
      <UiButton
        variant="secondary"
        :disabled="queue.loading"
        @click="
          queue.list();
          toolsOpen = false;
        "
      >
        <UIcon name="i-tabler-refresh" class="size-4" aria-hidden="true" />
        刷新任务列表
      </UiButton>
    </MobileSheet>
  </section>
</template>

<style scoped src="./TransferPage.css"></style>
