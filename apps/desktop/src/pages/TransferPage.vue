<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import type { TaskStatus } from '../api/dto'
import { sourceReference, useQueueStore, type QueueFilter } from '../stores/queue'
import {
  createTransferTaskView,
  formatSpeedLabel,
  type TaskActionDescriptor,
  type TaskActionKind,
} from '../stores/transferView'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiDialog from '../ui/Dialog.vue'
import UiInlineNotice from '../ui/InlineNotice.vue'
import UiSelect from '../ui/Select.vue'
import UiTabs from '../ui/Tabs.vue'
import UiTextField from '../ui/TextField.vue'
import BulkActionBar from '../ui/BulkActionBar.vue'
import TaskInspector from '../ui/TaskInspector.vue'
import TransferTaskTable from '../ui/TransferTaskTable.vue'

type TransferSortMode = 'queue' | 'name_asc' | 'progress_desc' | 'speed_desc' | 'issue_first'

const queue = useQueueStore()
const ui = useUiStore()
const completedSearch = ref('')
const transferSort = ref<TransferSortMode>('queue')
const taskDetailOpen = ref(false)
const contextMenu = ref<{ taskId: string; x: number; y: number } | null>(null)
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
const transferSortOptions = [
  { label: '默认顺序', value: 'queue' },
  { label: '名称 A-Z', value: 'name_asc' },
  { label: '进度高优先', value: 'progress_desc' },
  { label: '速度高优先', value: 'speed_desc' },
  { label: '问题优先', value: 'issue_first' },
]
const completedSearchQuery = computed(() => completedSearch.value.trim().toLowerCase())
const visibleTasks = computed(() => {
  let tasks = queue.filteredTasks
  if (queue.activeFilter === 'completed' && completedSearchQuery.value) {
    tasks = tasks.filter((task) =>
      [task.title, task.source_id, sourceReference(task.source_id), task.output_path].some((value) =>
        value.toLowerCase().includes(completedSearchQuery.value),
      ),
    )
  }

  return sortTransferTasks(tasks, transferSort.value)
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
const activeTaskCount = computed(
  () => queue.tasks.filter((task) => ['waiting', 'parsing', 'downloading', 'muxing'].includes(task.status)).length,
)
const attentionTaskCount = computed(
  () => queue.tasks.filter((task) => task.status === 'failed' || task.status === 'cancelled').length,
)
const completedTaskCount = computed(() => queue.tasks.filter((task) => task.status === 'completed').length)
const aggregateSpeed = computed(() =>
  queue.tasks.reduce((total, task) => total + (queue.taskTransferProgress(task.id)?.speedBytesPerSecond ?? 0), 0),
)
const contextTaskView = computed(() => taskViews.value.find((view) => view.id === contextMenu.value?.taskId) ?? null)
const contextActions = computed<TaskActionDescriptor[]>(() => {
  const view = contextTaskView.value
  if (!view) return []
  const actions = [...view.secondaryActions]
  if (view.primaryAction !== 'none') {
    actions.unshift({
      kind: view.primaryAction,
      label: view.primaryActionLabel,
      icon: view.primaryActionIcon,
    })
  }
  return actions
})

const selectedLogs = computed(() => (queue.selectedTaskId ? (queue.logsByTask[queue.selectedTaskId] ?? []) : []))
const selectedLogsLoading = computed(() =>
  queue.selectedTaskId ? Boolean(queue.logsLoadingByTask[queue.selectedTaskId]) : false,
)
const selectedProgress = computed(() => (queue.selectedTask ? queue.taskProgress(queue.selectedTask) : 0))
const selectedTaskSet = computed(() => new Set(queue.selectedTaskIds))
const selectedTasks = computed(() => queue.tasks.filter((task) => selectedTaskSet.value.has(task.id)))
const bulkScopeTasks = computed(() => (selectedTasks.value.length > 0 ? selectedTasks.value : visibleTasks.value))
const pausableTaskIds = computed(() => bulkScopeTasks.value.filter((task) => isPausable(task.status)).map((task) => task.id))
const cancellableTaskIds = computed(() => bulkScopeTasks.value.filter((task) => isCancellable(task.status)).map((task) => task.id))
const resumableTaskIds = computed(() => bulkScopeTasks.value.filter((task) => task.status === 'paused').map((task) => task.id))
const retryableTaskIds = computed(() => bulkScopeTasks.value.filter((task) => isRetryable(task.status)).map((task) => task.id))
const removableTaskIds = computed(() => selectedTasks.value.map((task) => task.id))
const selectedDetailTitle = computed(() => queue.selectedTask?.title ?? '任务详情')
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
  window.addEventListener('keydown', closeContextMenuOnEscape)
  window.addEventListener('blur', closeContextMenu)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', closeContextMenuOnEscape)
  window.removeEventListener('blur', closeContextMenu)
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

const openTaskDetail = (taskId: string) => {
  queue.selectTask(taskId)
  taskDetailOpen.value = true
}

const openContextMenu = (taskId: string, event: MouseEvent) => {
  const menuWidth = 190
  const menuHeight = 260
  contextMenu.value = {
    taskId,
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - menuWidth - 8)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - menuHeight - 8)),
  }
}

function closeContextMenu() {
  contextMenu.value = null
}

function closeContextMenuOnEscape(event: KeyboardEvent) {
  if (event.key === 'Escape') closeContextMenu()
}

const runContextAction = (action: Exclude<TaskActionKind, 'none'>) => {
  const taskId = contextMenu.value?.taskId
  closeContextMenu()
  if (taskId) handleTaskAction(taskId, action)
}

const contextIcon = (icon: string): string => {
  const aliases: Record<string, string> = {
    pause: 'player-pause',
    play: 'player-play',
    refresh: 'refresh',
    x: 'x',
    trash: 'trash',
    file: 'file',
    folder: 'folder',
    copy: 'copy',
  }
  return `i-tabler-${aliases[icon] ?? icon}`
}

const refreshSelectedLogs = () => {
  if (queue.selectedTaskId) {
    void queue.loadLogs(queue.selectedTaskId)
  }
}

const runBulkPause = () => {
  void queue.bulkPause(pausableTaskIds.value)
}

const runBulkCancel = () => {
  void queue.bulkCancel(cancellableTaskIds.value)
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

const isCancellable = (status: TaskStatus): boolean =>
  status === 'waiting' || status === 'parsing' || status === 'downloading' || status === 'muxing' || status === 'paused'

const isRetryable = (status: TaskStatus): boolean => status === 'failed' || status === 'cancelled' || status === 'completed'

const sortTransferTasks = (tasks: typeof queue.tasks, mode: TransferSortMode): typeof queue.tasks => {
  const indexed = tasks.map((task, index) => ({ task, index }))
  if (mode === 'queue') {
    return tasks
  }

  indexed.sort((left, right) => {
    if (mode === 'name_asc') {
      return compareByTitle(left.task.title, right.task.title, left.index, right.index)
    }

    if (mode === 'progress_desc') {
      const progress = queue.taskProgress(right.task) - queue.taskProgress(left.task)
      return progress || left.index - right.index
    }

    if (mode === 'speed_desc') {
      const speed =
        (queue.taskTransferProgress(right.task.id)?.speedBytesPerSecond ?? 0) -
        (queue.taskTransferProgress(left.task.id)?.speedBytesPerSecond ?? 0)
      return speed || left.index - right.index
    }

    const issue = issueRank(left.task.status) - issueRank(right.task.status)
    return issue || left.index - right.index
  })

  return indexed.map((entry) => entry.task)
}

const compareByTitle = (left: string, right: string, leftIndex: number, rightIndex: number): number => {
  const byTitle = left.localeCompare(right, 'zh-Hans-CN', { numeric: true, sensitivity: 'base' })
  return byTitle || leftIndex - rightIndex
}

const issueRank = (status: TaskStatus): number => {
  if (status === 'failed' || status === 'cancelled') {
    return 0
  }
  if (status === 'paused') {
    return 1
  }
  if (status === 'waiting' || status === 'parsing' || status === 'downloading' || status === 'muxing') {
    return 2
  }

  return 3
}
</script>

<template>
  <section class="page-grid transfer-page">
    <section class="panel transfer-main">
      <UiInlineNotice v-if="queue.notice" :tone="queue.notice.tone">
        {{ queue.notice.message }}
      </UiInlineNotice>

      <div class="transfer-health-strip" aria-label="队列状态概览">
        <div>
          <UIcon name="i-tabler-player-play" aria-hidden="true" />
          <span>进行中 <strong>{{ activeTaskCount }}</strong></span>
        </div>
        <div :class="{ attention: attentionTaskCount > 0 }">
          <UIcon name="i-tabler-alert-triangle" aria-hidden="true" />
          <span>需处理 <strong>{{ attentionTaskCount }}</strong></span>
        </div>
        <div>
          <UIcon name="i-tabler-circle-check" aria-hidden="true" />
          <span>已完成 <strong>{{ completedTaskCount }}</strong></span>
        </div>
        <div class="speed-signal">
          <span>总速率</span>
          <strong>{{ formatSpeedLabel(aggregateSpeed) }}</strong>
        </div>
      </div>

      <div class="transfer-toolbar">
        <UiTabs v-model="queueFilter" :tabs="tabs" />
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

      <div v-if="taskViews.length" class="task-list">
        <TransferTaskTable
          :views="taskViews"
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
      <div v-else class="empty-state task-empty-state">
        <div>
          <strong>{{ emptyTitle }}</strong>
          <p v-if="emptyDescription">{{ emptyDescription }}</p>
        </div>
        <UiButton v-if="queue.tasks.length === 0" variant="secondary" @click="ui.setTab('parse')">去解析</UiButton>
      </div>
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

<style scoped>
.transfer-page {
  grid-template-columns: minmax(0, 1fr);
}

.transfer-main {
  min-width: 0;
  min-height: 0;
}

.transfer-main {
  overflow: hidden;
}

.transfer-health-strip {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 0;
  overflow-x: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

.transfer-health-strip > div {
  min-width: max-content;
  height: 38px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: 0 var(--space-sm);
  color: var(--color-muted);
  font-size: var(--font-11);
  font-weight: 620;
}

.transfer-health-strip > div + div {
  border-left: 1px solid var(--color-border);
}

.transfer-health-strip svg {
  width: 15px;
  height: 15px;
  color: var(--color-accent-strong);
}

.transfer-health-strip strong {
  color: var(--color-text-strong);
  font-family: var(--font-display);
  font-size: var(--font-13);
  font-weight: 720;
}

.transfer-health-strip .attention svg,
.transfer-health-strip .attention strong {
  color: var(--color-warning);
}

.transfer-health-strip .speed-signal {
  margin-left: auto;
  background: var(--color-surface);
}

.transfer-list-tools {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(220px, 420px) minmax(150px, 190px);
  align-items: end;
  gap: var(--space-12);
}

.transfer-toolbar {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-12);
}

.task-list {
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
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

.task-detail-dialog {
  min-width: 0;
  min-height: 0;
  height: min(680px, calc(100vh - 180px));
}

.task-detail-dialog :deep(.task-inspector) {
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: auto minmax(0, 1fr);
}

.task-detail-dialog :deep(.ui-tabs),
.task-detail-dialog :deep(.tab-panel) {
  grid-column: auto;
}

.context-menu-scrim {
  position: fixed;
  inset: 0;
  z-index: 90;
}

.task-context-menu {
  position: fixed;
  width: 190px;
  display: grid;
  gap: 2px;
  padding: var(--space-2xs);
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-8);
  background: var(--color-surface-raised);
  box-shadow: var(--shadow-overlay);
  animation: context-menu-in var(--duration-fast) var(--ease-out);
}

.context-menu-heading {
  min-width: 0;
  display: grid;
  gap: 2px;
  margin-bottom: 2px;
  padding: var(--space-xs) var(--space-sm);
  border-bottom: 1px solid var(--color-border);
}

.context-menu-heading span {
  color: var(--color-muted);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-transform: uppercase;
}

.context-menu-heading strong {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text);
  font-size: var(--font-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-context-menu button {
  min-height: 34px;
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-xs);
  border: 0;
  border-radius: var(--radius-4);
  background: transparent;
  color: var(--color-text);
  padding: 0 var(--space-sm);
  text-align: left;
  font-size: var(--font-12);
  font-weight: 620;
}

.task-context-menu button:hover,
.task-context-menu button:focus-visible {
  background: var(--color-panel);
}

.task-context-menu button.danger {
  color: var(--color-danger);
}

.task-context-menu svg {
  width: 16px;
  height: 16px;
}

@keyframes context-menu-in {
  from { opacity: 0; transform: translateY(-4px) scale(0.98); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

@media (max-width: 980px) {
  .transfer-toolbar,
  .transfer-list-tools {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
