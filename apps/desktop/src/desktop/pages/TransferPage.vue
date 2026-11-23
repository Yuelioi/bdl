<script setup lang="ts">
import UiButton from '../../shared/ui/Button.vue';
import UiDialog from '../../shared/ui/Dialog.vue';
import UiEmptyState from '../../shared/ui/EmptyState.vue';
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import UiTabs from '../../shared/ui/Tabs.vue';
import BulkActionBar from '../../shared/ui/BulkActionBar.vue';
import TaskInspector from '../../shared/ui/TaskInspector.vue';
import TransferTaskTable from '../components/transfer/TransferTaskTable.vue';
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
  contextMenu,
  contextTaskView,
  contextActions,
  openContextMenu,
  closeContextMenu,
  runContextAction,
  contextIcon,
} = useTransferPage();

const recoverTask = (taskId: string, action: 'retry' | 'refresh_retry') => {
  taskDetailOpen.value = false;
  handleTaskAction(taskId, action);
};
</script>
<template>
  <section class="page-grid transfer-page">
    <section class="panel transfer-main">
      <h2 class="transfer-page-title">传输</h2>
      <UiInlineNotice v-if="queue.notice" :tone="queue.notice.tone">
        {{ queue.notice.message }}
      </UiInlineNotice>

      <div class="transfer-toolbar">
        <UiTabs v-model="queueFilter" :tabs="tabs" />
        <div class="transfer-list-tools">
          <UInput
            v-if="queue.activeFilter === 'completed'"
            v-model="completedSearch"
            class="transfer-search"
            type="search"
            aria-label="搜索已完成"
            icon="i-tabler-search"
            placeholder="搜索标题、来源或保存路径"
            :disabled="queue.loading"
            color="neutral"
            variant="outline"
            size="sm"
          />
          <USelect
            v-model="transferSort"
            :items="transferSortOptions"
            value-key="value"
            label-key="label"
            aria-label="列表排序"
            class="transfer-sort"
            size="sm"
            :disabled="queue.loading"
          />
        </div>
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
          @visible-tasks="queue.loadOutputSizes"
        />
      </div>
      <UiEmptyState
        v-else
        :title="emptyTitle"
        :description="emptyDescription || undefined"
        icon="i-tabler-transfer"
        layout="stacked"
        compact
        embedded
      >
        <template v-if="queue.tasks.length === 0" #action>
          <UiButton variant="secondary" @click="ui.setTab('parse')">去解析</UiButton>
        </template>
      </UiEmptyState>

      <footer class="transfer-footer">
        <BulkActionBar
          :selected-count="queue.selectedTaskIds.length"
          :items="batchItems"
          :loading="queue.loading || queue.bulkBusy"
          @refresh="queue.list"
        />
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
          @recover="recoverTask"
        />
    </UiDialog>


    <Teleport to="body">
      <div v-if="contextMenu" class="context-menu-scrim" @pointerdown="closeContextMenu">
        <div
          class="task-context-menu"
          role="menu"
          :aria-label="`${contextTaskView?.displayTitle ?? '任务'}操作`"
          :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px`, maxHeight: `calc(100dvh - ${contextMenu.y}px - 8px)` }"
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
