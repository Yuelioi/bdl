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
  batchItems,
  removal,
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
            :disabled="queue.loading || queue.bulkBusy || !batchItems.length"
          >批量操作 <UIcon name="i-tabler-chevron-up" class="size-4" aria-hidden="true" /></UiButton>
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
