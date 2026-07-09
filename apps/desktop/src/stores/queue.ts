import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { defineStore } from 'pinia'

import type { BulkQueueResult, DownloadTask, QueueLogEntry } from '../api/dto'
import {
  queueBulkPause,
  queueBulkRefreshUrlsAndRetry,
  queueBulkRemove,
  queueBulkResume,
  queueBulkRetry,
  queueCancel,
  queueClearCompleted,
  queueList,
  queueLogs,
  queueOpenDir,
  queueOpenFile,
  queuePause,
  queueRefreshUrlsAndRetry,
  queueRemove,
  queueResume,
  queueRetry,
} from '../api/tauri'
import { useUiStore } from './ui'
import { defaultQueueFilter, filterTaskByWorkflow, type QueueFilter } from './transferView'

export type { QueueFilter } from './transferView'

interface QueueState {
  tasks: DownloadTask[]
  activeFilter: QueueFilter
  filterTouched: boolean
  selectedTaskId: string | null
  selectedTaskIds: string[]
  logsByTask: Record<string, QueueLogEntry[]>
  logsLoadingByTask: Record<string, boolean>
  loading: boolean
  listening: boolean
  unlisten: UnlistenFn[]
}

export const useQueueStore = defineStore('queue', {
  state: (): QueueState => ({
    tasks: [],
    activeFilter: 'active',
    filterTouched: false,
    selectedTaskId: null,
    selectedTaskIds: [],
    logsByTask: {},
    logsLoadingByTask: {},
    loading: false,
    listening: false,
    unlisten: [],
  }),
  getters: {
    filteredTasks(state): DownloadTask[] {
      return state.tasks.filter((task) => filterTaskByWorkflow(task.status, state.activeFilter))
    },
    selectedTask(state): DownloadTask | null {
      return state.selectedTaskId ? (state.tasks.find((task) => task.id === state.selectedTaskId) ?? null) : null
    },
  },
  actions: {
    async list() {
      const ui = useUiStore()
      this.loading = true
      try {
        this.tasks = await queueList()
        this.applyDefaultFilter()
        this.selectedTaskIds = this.selectedTaskIds.filter((taskId) =>
          this.tasks.some((task) => task.id === taskId),
        )
        this.ensureSelectedTask(true)
        if (this.selectedTaskId) {
          await this.loadLogs(this.selectedTaskId)
        }
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loading = false
      }
    },
    async startEventListeners() {
      if (this.listening) {
        return
      }

      this.listening = true
      this.unlisten.push(
        await listen<DownloadTask>('queue://task-updated', (event) => {
          this.upsertTask(event.payload)
        }),
      )
      this.unlisten.push(
        await listen<QueueLogEntry>('queue://log-appended', (event) => {
          const list = this.logsByTask[event.payload.task_id] ?? []
          this.logsByTask[event.payload.task_id] = mergeLogs([event.payload], list)
        }),
      )
    },
    setFilter(filter: QueueFilter) {
      this.activeFilter = filter
      this.filterTouched = true
      this.ensureSelectedTask(true)
    },
    selectTask(taskId: string) {
      this.selectedTaskId = taskId
      void this.loadLogs(taskId)
    },
    upsertTask(task: DownloadTask) {
      const index = this.tasks.findIndex((candidate) => candidate.id === task.id)
      if (index === -1) {
        this.tasks.unshift(task)
      } else {
        this.tasks[index] = task
      }
      this.applyDefaultFilter()
      this.selectedTaskId = this.selectedTaskId ?? task.id
    },
    countByFilter(filter: QueueFilter): number {
      return this.tasks.filter((task) => filterTaskByWorkflow(task.status, filter)).length
    },
    ensureSelectedTask(preferVisible = false) {
      if (
        this.selectedTaskId &&
        this.tasks.some((task) => task.id === this.selectedTaskId) &&
        (!preferVisible || this.filteredTasks.some((task) => task.id === this.selectedTaskId))
      ) {
        return
      }

      this.selectedTaskId = this.filteredTasks[0]?.id ?? this.tasks[0]?.id ?? null
    },
    applyDefaultFilter() {
      if (!this.filterTouched) {
        this.activeFilter = defaultQueueFilter(this.tasks)
      }
    },
    toggleTaskSelection(taskId: string) {
      if (this.selectedTaskIds.includes(taskId)) {
        this.selectedTaskIds = this.selectedTaskIds.filter((selectedTaskId) => selectedTaskId !== taskId)
        return
      }

      this.selectedTaskIds = [...this.selectedTaskIds, taskId]
    },
    setVisibleTaskSelection(taskIds: string[], selected: boolean) {
      const visible = new Set(taskIds)
      if (!selected) {
        this.selectedTaskIds = this.selectedTaskIds.filter((taskId) => !visible.has(taskId))
        return
      }

      this.selectedTaskIds = Array.from(new Set([...this.selectedTaskIds, ...taskIds]))
    },
    taskProgress(task: DownloadTask): number {
      if (task.status === 'completed') {
        return 100
      }

      if (task.resources.length === 0) {
        return 0
      }

      const completed = task.resources.filter((resource) => resource.status === 'completed').length
      return Math.round((completed / task.resources.length) * 100)
    },
    async pause(taskId: string) {
      await this.runTaskCommand(() => queuePause(taskId), '已暂停')
    },
    async resume(taskId: string) {
      await this.runTaskCommand(() => queueResume(taskId), '已恢复到队列')
    },
    async cancel(taskId: string) {
      await this.runTaskCommand(() => queueCancel(taskId), '已取消')
    },
    async retry(taskId: string) {
      await this.runTaskCommand(() => queueRetry(taskId), '已重新入队')
      await this.loadLogs(taskId)
    },
    async refreshUrlsAndRetry(taskId: string) {
      await this.runTaskCommand(() => queueRefreshUrlsAndRetry(taskId), '已刷新链接并重新入队')
      await this.loadLogs(taskId)
    },
    async bulkPause(taskIds: string[]) {
      await this.runBulkCommand(() => queueBulkPause({ task_ids: taskIds }), '暂停')
    },
    async bulkResume(taskIds: string[]) {
      await this.runBulkCommand(() => queueBulkResume({ task_ids: taskIds }), '继续')
    },
    async bulkRetry(taskIds: string[]) {
      await this.runBulkCommand(() => queueBulkRetry({ task_ids: taskIds }), '重试')
    },
    async bulkRefreshUrlsAndRetry(taskIds: string[]) {
      await this.runBulkCommand(() => queueBulkRefreshUrlsAndRetry({ task_ids: taskIds }), '刷新链接并重试')
    },
    async bulkRemove(taskIds: string[]) {
      await this.runBulkCommand(() => queueBulkRemove({ task_ids: taskIds }), '移除')
    },
    async clearCompleted() {
      await this.runBulkCommand(() => queueClearCompleted(), '清理已完成')
    },
    async remove(taskId: string) {
      const ui = useUiStore()
      try {
        await queueRemove(taskId)
        this.tasks = this.tasks.filter((task) => task.id !== taskId)
        this.selectedTaskIds = this.selectedTaskIds.filter((selectedTaskId) => selectedTaskId !== taskId)
        delete this.logsByTask[taskId]
        delete this.logsLoadingByTask[taskId]
        this.ensureSelectedTask(true)
        if (this.selectedTaskId) {
          void this.loadLogs(this.selectedTaskId)
        }
        ui.pushToast('已移除任务', 'info')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      }
    },
    async openFile(taskId: string) {
      await this.runVoidCommand(() => queueOpenFile(taskId))
    },
    async openDir(taskId: string) {
      await this.runVoidCommand(() => queueOpenDir(taskId))
    },
    async loadLogs(taskId: string) {
      const ui = useUiStore()
      this.logsLoadingByTask[taskId] = true
      try {
        const logs = await queueLogs(taskId, LOG_LIMIT)
        this.logsByTask[taskId] = mergeLogs(logs, this.logsByTask[taskId] ?? [])
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.logsLoadingByTask[taskId] = false
      }
    },
    async runTaskCommand(command: () => Promise<DownloadTask>, successMessage: string) {
      const ui = useUiStore()
      try {
        const task = await command()
        this.upsertTask(task)
        ui.pushToast(successMessage, 'success')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      }
    },
    async runBulkCommand(command: () => Promise<BulkQueueResult>, actionLabel: string) {
      const ui = useUiStore()
      try {
        const result = await command()
        this.applyBulkResult(result)

        const succeeded = result.updated.length + result.removed.length
        if (succeeded > 0) {
          ui.pushToast(`${actionLabel} ${succeeded} 个任务`, result.failed.length ? 'warning' : 'success')
        }
        if (result.failed.length > 0) {
          ui.pushToast(`${result.failed.length} 个任务处理失败`, 'danger')
        }
        if (succeeded === 0 && result.failed.length === 0) {
          ui.pushToast('没有可处理的任务', 'info')
        }
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      }
    },
    applyBulkResult(result: BulkQueueResult) {
      for (const task of result.updated) {
        const index = this.tasks.findIndex((candidate) => candidate.id === task.id)
        if (index === -1) {
          this.tasks.unshift(task)
        } else {
          this.tasks[index] = task
        }
      }

      const removed = new Set(result.removed)
      if (removed.size > 0) {
        this.tasks = this.tasks.filter((task) => !removed.has(task.id))
        this.selectedTaskIds = this.selectedTaskIds.filter((taskId) => !removed.has(taskId))
        for (const taskId of removed) {
          delete this.logsByTask[taskId]
          delete this.logsLoadingByTask[taskId]
        }
      }

      this.applyDefaultFilter()
      this.ensureSelectedTask(true)
    },
    async runVoidCommand(command: () => Promise<void>) {
      const ui = useUiStore()
      try {
        await command()
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      }
    },
  },
})

const LOG_LIMIT = 200

const mergeLogs = (...sources: QueueLogEntry[][]): QueueLogEntry[] => {
  const seen = new Set<string>()
  const merged: QueueLogEntry[] = []

  for (const logs of sources) {
    for (const log of logs) {
      const key = logKey(log)
      if (seen.has(key)) {
        continue
      }

      seen.add(key)
      merged.push(log)
    }
  }

  return merged.sort((a, b) => b.created_at.localeCompare(a.created_at)).slice(0, LOG_LIMIT)
}

const logKey = (log: QueueLogEntry): string =>
  `${log.task_id}\n${log.created_at}\n${log.level}\n${log.message}`

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
