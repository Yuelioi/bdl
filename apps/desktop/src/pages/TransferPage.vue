<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import type { TaskStatus } from '../api/dto'
import { sourceReference, useQueueStore, type QueueFilter } from '../stores/queue'
import { useSettingsStore } from '../stores/settings'
import { createTransferTaskView, formatSpeedLabel, type TaskActionKind } from '../stores/transferView'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiTabs from '../ui/Tabs.vue'
import UiTextField from '../ui/TextField.vue'
import BulkActionBar from '../ui/BulkActionBar.vue'
import TaskInspector from '../ui/TaskInspector.vue'
import TransferTaskTable from '../ui/TransferTaskTable.vue'

const queue = useQueueStore()
const settings = useSettingsStore()
const ui = useUiStore()
const completedSearch = ref('')
const queueFilter = computed({
  get: () => queue.activeFilter,
  set: (value: QueueFilter) => queue.setFilter(value),
})

const tabs = computed<Array<{ label: string; value: QueueFilter; count: number }>>(() => [
  { label: '活动', value: 'active', count: queue.countByFilter('active') },
  { label: '失败', value: 'failed', count: queue.countByFilter('failed') },
  { label: '已完成', value: 'completed', count: queue.countByFilter('completed') },
  { label: '全部', value: 'all', count: queue.tasks.length },
])
const completedSearchQuery = computed(() => completedSearch.value.trim().toLowerCase())
const visibleTasks = computed(() => {
  if (queue.activeFilter !== 'completed' || !completedSearchQuery.value) {
    return queue.filteredTasks
  }

  return queue.filteredTasks.filter((task) =>
    [task.title, task.source_id, sourceReference(task.source_id), task.output_path].some((value) =>
      value.toLowerCase().includes(completedSearchQuery.value),
    ),
  )
})
const taskViews = computed(() =>
  visibleTasks.value.map((task) =>
    createTransferTaskView(
      task,
      queue.taskProgress(task),
      queue.logsByTask[task.id] ?? [],
      queue.taskTransferProgress(task.id),
    ),
  ),
)

const selectedLogs = computed(() => (queue.selectedTaskId ? (queue.logsByTask[queue.selectedTaskId] ?? []) : []))
const selectedLogsLoading = computed(() =>
  queue.selectedTaskId ? Boolean(queue.logsLoadingByTask[queue.selectedTaskId]) : false,
)
const selectedProgress = computed(() => (queue.selectedTask ? queue.taskProgress(queue.selectedTask) : 0))
const selectedTransferProgress = computed(() =>
  queue.selectedTaskId ? queue.taskTransferProgress(queue.selectedTaskId) : null,
)
const selectedTaskSet = computed(() => new Set(queue.selectedTaskIds))
const selectedTasks = computed(() => queue.tasks.filter((task) => selectedTaskSet.value.has(task.id)))
const bulkScopeTasks = computed(() => (selectedTasks.value.length > 0 ? selectedTasks.value : visibleTasks.value))
const pausableTaskIds = computed(() => bulkScopeTasks.value.filter((task) => isPausable(task.status)).map((task) => task.id))
const resumableTaskIds = computed(() => bulkScopeTasks.value.filter((task) => task.status === 'paused').map((task) => task.id))
const retryableTaskIds = computed(() => bulkScopeTasks.value.filter((task) => isRetryable(task.status)).map((task) => task.id))
const removableTaskIds = computed(() => selectedTasks.value.map((task) => task.id))
const completedTaskCount = computed(() => queue.tasks.filter((task) => task.status === 'completed').length)
const downloadingTaskCount = computed(
  () => queue.tasks.filter((task) => task.status === 'downloading' || task.status === 'muxing').length,
)
const queuedTaskCount = computed(
  () => queue.tasks.filter((task) => task.status === 'waiting' || task.status === 'parsing').length,
)
const failedTaskCount = computed(
  () => queue.tasks.filter((task) => task.status === 'failed' || task.status === 'cancelled').length,
)
const totalSpeedLabel = computed(() => formatSpeedLabel(queue.totalSpeedBytesPerSecond()))
const emptyTitle = computed(() => {
  if (queue.tasks.length === 0) {
    return '还没有传输任务'
  }
  if (queue.activeFilter === 'active') {
    return '没有活动任务'
  }
  if (queue.activeFilter === 'failed') {
    return '没有失败任务'
  }
  if (queue.activeFilter === 'completed') {
    if (completedSearchQuery.value) {
      return '没有匹配的完成记录'
    }
    return '还没有完成任务'
  }

  return '没有匹配任务'
})
const emptyDescription = computed(() =>
  queue.tasks.length === 0 ? '在解析页选择视频后，任务会出现在这里。' : '',
)

onMounted(() => {
  void queue.startEventListeners()
  void queue.list()
})

const handleTaskAction = (taskId: string, action: Exclude<TaskActionKind, 'none'>) => {
  if (action === 'pause') {
    void queue.pause(taskId)
    return
  }
  if (action === 'resume') {
    void queue.resume(taskId)
    return
  }
  if (action === 'retry') {
    void queue.retry(taskId)
    return
  }
  if (action === 'refresh_retry') {
    void queue.refreshUrlsAndRetry(taskId)
    return
  }
  if (action === 'cancel') {
    void queue.cancel(taskId)
    return
  }
  if (action === 'remove') {
    void queue.remove(taskId)
    return
  }
  if (action === 'open_file') {
    void queue.openFile(taskId)
    return
  }
  if (action === 'open_dir') {
    void queue.openDir(taskId)
    return
  }
  if (action === 'copy_source') {
    void queue.copySource(taskId)
  }
}

const handleSelectedTaskAction = (action: Exclude<TaskActionKind, 'none'>) => {
  if (queue.selectedTaskId) {
    handleTaskAction(queue.selectedTaskId, action)
  }
}

const refreshSelectedLogs = () => {
  if (queue.selectedTaskId) {
    void queue.loadLogs(queue.selectedTaskId)
  }
}

const runBulkPause = () => {
  void queue.bulkPause(pausableTaskIds.value)
}

const runBulkResume = () => {
  void queue.bulkResume(resumableTaskIds.value)
}

const runBulkRetry = () => {
  void queue.bulkRetry(retryableTaskIds.value)
}

const runBulkRefreshRetry = () => {
  void queue.bulkRefreshUrlsAndRetry(retryableTaskIds.value)
}

const runBulkRemove = () => {
  void queue.bulkRemove(removableTaskIds.value)
}

const runClearCompleted = () => {
  void queue.clearCompleted()
}

const isPausable = (status: TaskStatus): boolean =>
  status === 'waiting' || status === 'parsing' || status === 'downloading' || status === 'muxing'

const isRetryable = (status: TaskStatus): boolean => status === 'failed' || status === 'cancelled' || status === 'completed'
</script>

<template>
  <section class="page-grid transfer-page">
    <section class="panel transfer-main">
      <div class="panel-heading">
        <div>
          <h2>任务</h2>
          <span class="muted-text">{{ taskViews.length }} 个</span>
        </div>
        <UiButton variant="secondary" :disabled="queue.loading" @click="queue.list">刷新</UiButton>
      </div>

      <div class="transfer-status-strip" aria-label="传输状态">
        <span>下载 {{ downloadingTaskCount }}</span>
        <span>队列 {{ queuedTaskCount }}</span>
        <span :class="{ danger: failedTaskCount > 0 }">失败 {{ failedTaskCount }}</span>
        <span>速度 {{ totalSpeedLabel }}</span>
        <span>并发 {{ settings.saved.concurrent_tasks }}</span>
      </div>

      <UiTabs v-model="queueFilter" :tabs="tabs" />

      <div v-if="queue.activeFilter === 'completed'" class="completed-search">
        <UiTextField
          v-model="completedSearch"
          label="搜索已完成"
          placeholder="标题、来源或保存路径"
          :disabled="queue.loading"
        />
      </div>

      <BulkActionBar
        :selected-count="queue.selectedTaskIds.length"
        :visible-count="taskViews.length"
        :completed-count="completedTaskCount"
        :can-pause="pausableTaskIds.length > 0"
        :can-resume="resumableTaskIds.length > 0"
        :can-retry="retryableTaskIds.length > 0"
        :can-refresh-retry="retryableTaskIds.length > 0"
        :can-remove="removableTaskIds.length > 0"
        :loading="queue.loading"
        @pause="runBulkPause"
        @resume="runBulkResume"
        @retry="runBulkRetry"
        @refresh-retry="runBulkRefreshRetry"
        @remove="runBulkRemove"
        @clear-completed="runClearCompleted"
        @refresh="queue.list"
      />

      <div v-if="taskViews.length" class="task-list">
        <TransferTaskTable
          :views="taskViews"
          :selected-task-id="queue.selectedTaskId"
          :selected-task-ids="queue.selectedTaskIds"
          :loading="queue.loading"
          @select-task="queue.selectTask"
          @toggle-task-selection="queue.toggleTaskSelection"
          @toggle-visible-selection="queue.setVisibleTaskSelection"
          @task-action="handleTaskAction"
        />
      </div>
      <div v-else class="empty-state task-empty-state">
        <div>
          <strong>{{ emptyTitle }}</strong>
          <p v-if="emptyDescription">{{ emptyDescription }}</p>
        </div>
        <UiButton v-if="queue.tasks.length === 0" variant="secondary" @click="ui.setTab('parse')">去解析</UiButton>
      </div>
    </section>

    <aside class="panel transfer-detail">
      <TaskInspector
        :task="queue.selectedTask"
        :progress="selectedProgress"
        :transfer-progress="selectedTransferProgress"
        :logs="selectedLogs"
        :logs-loading="selectedLogsLoading"
        @task-action="handleSelectedTaskAction"
        @refresh-logs="refreshSelectedLogs"
      />
    </aside>
  </section>
</template>

<style scoped>
.transfer-page {
  grid-template-columns: minmax(0, 1fr) minmax(360px, 420px);
}

.transfer-main,
.transfer-detail {
  min-width: 0;
  min-height: 0;
}

.transfer-main {
  overflow: hidden;
}

.transfer-detail {
  overflow: hidden;
}

.completed-search {
  max-width: 420px;
}

.task-list {
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.transfer-status-strip {
  min-width: 0;
  min-height: 32px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-8);
  padding: 0 var(--space-8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.transfer-status-strip span {
  white-space: nowrap;
}

.transfer-status-strip .danger {
  color: var(--color-danger);
}

.empty-state {
  min-height: 120px;
  display: grid;
  place-items: center;
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
  color: var(--color-muted);
  font-size: var(--font-12);
}

.task-empty-state {
  align-content: center;
  gap: var(--space-12);
  text-align: center;
}

.task-empty-state strong {
  display: block;
  color: var(--color-text);
  font-size: var(--font-14);
}

.task-empty-state p {
  margin: var(--space-4) 0 0;
  color: var(--color-muted);
  font-size: var(--font-13);
}

@media (max-width: 1120px) {
  .transfer-page {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(354px, 1fr) minmax(248px, 38%);
  }
}
</style>
