<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import type { TaskStatus } from '../api/dto'
import { sourceReference, useQueueStore, type QueueFilter } from '../stores/queue'
import { createTransferTaskView, type TaskActionDescriptor, type TaskActionKind } from '../stores/transferView'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiDialog from '../ui/Dialog.vue'
import UiEmptyState from '../ui/EmptyState.vue'
import UiInlineNotice from '../ui/InlineNotice.vue'
import UiSelect from '../ui/Select.vue'
import UiTabs from '../ui/Tabs.vue'
import UiTextField from '../ui/TextField.vue'
import BulkActionBar from '../ui/BulkActionBar.vue'
import TaskInspector from '../ui/TaskInspector.vue'
import TransferTaskTable from '../ui/TransferTaskTable.vue'
import { scheduledLocalError, toDateTimeLocalValue, toScheduledIso } from '../utils/schedule'
import { speedLimitMibError, toBytesPerSecond, toMibPerSecondInput } from '../utils/speedLimit'

type TransferSortMode = 'queue' | 'name_asc' | 'progress_desc' | 'speed_desc' | 'issue_first'

const queue = useQueueStore()
const ui = useUiStore()
const completedSearch = ref('')
const transferSort = ref<TransferSortMode>('queue')
const taskDetailOpen = ref(false)
const scheduleDialogOpen = ref(false)
const scheduleTaskId = ref<string | null>(null)
const scheduleLocal = ref('')
const scheduleMin = ref('')
const scheduleValidationNow = ref(Date.now())
const speedLimitDialogOpen = ref(false)
const speedLimitTaskId = ref<string | null>(null)
const speedLimitMib = ref('')
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
const completedTaskCount = computed(() => queue.tasks.filter((task) => task.status === 'completed').length)
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
const pausableTaskIds = computed(() =>
  bulkScopeTasks.value.filter((task) => isPausable(task.status)).map((task) => task.id),
)
const cancellableTaskIds = computed(() =>
  bulkScopeTasks.value.filter((task) => isCancellable(task.status)).map((task) => task.id),
)
const resumableTaskIds = computed(() =>
  bulkScopeTasks.value.filter((task) => task.status === 'paused').map((task) => task.id),
)
const retryableTaskIds = computed(() =>
  bulkScopeTasks.value.filter((task) => isRetryable(task.status)).map((task) => task.id),
)
const removableTaskIds = computed(() => selectedTasks.value.map((task) => task.id))
const selectedDetailTitle = computed(() => queue.selectedTask?.title ?? '任务详情')
const scheduleError = computed(() => {
  return scheduledLocalError(scheduleLocal.value, scheduleValidationNow.value, true)
})
const speedLimitError = computed(() => speedLimitMibError(speedLimitMib.value))
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
const emptyDescription = computed(() => (queue.tasks.length === 0 ? '在解析页选择视频后，任务会出现在这里。' : ''))

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
  if (action === 'unschedule') {
    void queue.unschedule(taskId)
    return
  }
  if (action === 'schedule') {
    openScheduleDialog(taskId)
    return
  }
  if (action === 'speed_limit') {
    openSpeedLimitDialog(taskId)
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

const openScheduleDialog = (taskId: string) => {
  const task = queue.tasks.find((task) => task.id === taskId)
  scheduleValidationNow.value = Date.now()
  const minimum = new Date(scheduleValidationNow.value + 60_000)
  scheduleTaskId.value = taskId
  scheduleMin.value = toDateTimeLocalValue(minimum)
  scheduleLocal.value = toDateTimeLocalValue(
    task?.scheduled_at ? new Date(task.scheduled_at) : new Date(Date.now() + 300_000),
  )
  scheduleDialogOpen.value = true
}

const submitSchedule = async () => {
  scheduleValidationNow.value = Date.now()
  if (!scheduleTaskId.value || scheduleError.value) return
  const updated = await queue.schedule(scheduleTaskId.value, toScheduledIso(scheduleLocal.value))
  if (updated) {
    scheduleDialogOpen.value = false
  }
}

const openSpeedLimitDialog = (taskId: string) => {
  const task = queue.tasks.find((task) => task.id === taskId)
  speedLimitTaskId.value = taskId
  speedLimitMib.value = toMibPerSecondInput(task?.speed_limit_bytes_per_second)
  speedLimitDialogOpen.value = true
}

const submitSpeedLimit = async () => {
  if (!speedLimitTaskId.value || speedLimitError.value) return
  const updated = await queue.setSpeedLimit(speedLimitTaskId.value, toBytesPerSecond(speedLimitMib.value))
  if (updated) {
    speedLimitDialogOpen.value = false
  }
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
  status === 'waiting' || status === 'parsing' || status === 'downloading'

const isCancellable = (status: TaskStatus): boolean =>
  status === 'waiting' || status === 'parsing' || status === 'downloading' || status === 'paused'

const isRetryable = (status: TaskStatus): boolean =>
  status === 'failed' || status === 'cancelled' || status === 'completed'

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
