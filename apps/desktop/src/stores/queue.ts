import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { defineStore } from 'pinia'

import type { DownloadTask, QueueLogEntry, TaskStatus } from '../api/dto'
import {
  queueCancel,
  queueList,
  queueLogs,
  queueOpenDir,
  queueOpenFile,
  queuePause,
  queueRemove,
  queueResume,
  queueRetry,
} from '../api/tauri'
import { useUiStore } from './ui'

export type QueueFilter = 'downloading' | 'queued' | 'paused' | 'failed' | 'completed' | 'all'

interface QueueState {
  tasks: DownloadTask[]
  activeFilter: QueueFilter
  selectedTaskId: string | null
  logsByTask: Record<string, QueueLogEntry[]>
  loading: boolean
  listening: boolean
  unlisten: UnlistenFn[]
}

export const useQueueStore = defineStore('queue', {
  state: (): QueueState => ({
    tasks: [],
    activeFilter: 'downloading',
    selectedTaskId: null,
    logsByTask: {},
    loading: false,
    listening: false,
    unlisten: [],
  }),
  getters: {
    filteredTasks(state): DownloadTask[] {
      return state.tasks.filter((task) => filterTask(task.status, state.activeFilter))
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
        this.selectedTaskId = this.selectedTaskId ?? this.tasks[0]?.id ?? null
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
      this.selectedTaskId = this.selectedTaskId ?? task.id
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
    },
    async remove(taskId: string) {
      const ui = useUiStore()
      try {
        await queueRemove(taskId)
        this.tasks = this.tasks.filter((task) => task.id !== taskId)
        delete this.logsByTask[taskId]
        this.selectedTaskId = this.tasks[0]?.id ?? null
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
      try {
        const logs = await queueLogs(taskId, LOG_LIMIT)
        this.logsByTask[taskId] = mergeLogs(logs, this.logsByTask[taskId] ?? [])
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
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

const filterTask = (status: TaskStatus, filter: QueueFilter): boolean => {
  switch (filter) {
    case 'downloading':
      return status === 'downloading' || status === 'parsing' || status === 'muxing'
    case 'queued':
      return status === 'waiting'
    case 'paused':
      return status === 'paused'
    case 'failed':
      return status === 'failed' || status === 'cancelled'
    case 'completed':
      return status === 'completed'
    case 'all':
      return true
  }
}

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
