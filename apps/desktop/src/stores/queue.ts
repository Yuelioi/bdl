import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { defineStore } from 'pinia'

import type {
  BulkQueueResult,
  DownloadTask,
  QueueLogEntry,
  QueueProgressEntry,
  StartupRecoverySnapshot,
} from '../api/dto'
import {
  queueBulkPause,
  queueBulkCancel,
  queueBulkRefreshUrlsAndRetry,
  queueBulkRemove,
  queueBulkResume,
  queueBulkRetry,
  queueCancel,
  queueClearCompleted,
  queueDismissStartupRecovery,
  queueList,
  queueLogs,
  queueOpenDir,
  queueOpenFile,
  queuePause,
  queueRefreshUrlsAndRetry,
  queueRemove,
  queueResume,
  queueSchedule,
  queueRetry,
  queueStartupRecovery,
  queueUnschedule,
} from '../api/tauri'
import { useUiStore } from './ui'
import type { InlineNotice, NoticeTone } from './feedback'
import { NOTICE_CLEAR_DELAY } from './feedback'
import {
  defaultQueueFilter,
  filterTaskByWorkflow,
  type QueueFilter,
  type TransferProgressSnapshot,
} from './transferView'

export type { QueueFilter } from './transferView'

interface QueueState {
  tasks: DownloadTask[]
  activeFilter: QueueFilter
  filterTouched: boolean
  selectedTaskId: string | null
  selectedTaskIds: string[]
  logsByTask: Record<string, QueueLogEntry[]>
  logsLoadingByTask: Record<string, boolean>
  progressByTask: Record<string, QueueTaskProgressState>
  startupRecovery: StartupRecoverySnapshot | null
  startupRecoveryLoading: boolean
  startupRecoveryDismissed: boolean
  loading: boolean
  notice: InlineNotice | null
  noticeTimer: number | null
  listening: boolean
  unlisten: UnlistenFn[]
}

interface ResourceProgressState {
  downloadedBytes: number
  totalBytes: number | null
}

interface QueueTaskProgressState extends TransferProgressSnapshot {
  resources: Record<string, ResourceProgressState>
  speedSamples: SpeedSample[]
}

interface SpeedSample {
  downloadedBytes: number
  at: number
}

const SPEED_SAMPLE_WINDOW_MS = 5_000
const MIN_SPEED_SAMPLE_INTERVAL_MS = 150

export const useQueueStore = defineStore('queue', {
  state: (): QueueState => ({
    tasks: [],
    activeFilter: 'active',
    filterTouched: false,
    selectedTaskId: null,
    selectedTaskIds: [],
    logsByTask: {},
    logsLoadingByTask: {},
    progressByTask: {},
    startupRecovery: null,
    startupRecoveryLoading: false,
    startupRecoveryDismissed: false,
    loading: false,
    notice: null,
    noticeTimer: null,
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
      this.unlisten.push(
        await listen<QueueProgressEntry>('queue://progress-updated', (event) => {
          this.applyProgress(event.payload)
        }),
      )
    },
    setFilter(filter: QueueFilter) {
      this.activeFilter = filter
      this.filterTouched = true
      this.ensureSelectedTask(true)
    },
    setNotice(message: string, tone: NoticeTone = 'info') {
      this.notice = { message, tone }
      if (this.noticeTimer !== null && typeof window !== 'undefined') {
        window.clearTimeout(this.noticeTimer)
        this.noticeTimer = null
      }

      if (tone !== 'danger' && typeof window !== 'undefined') {
        this.noticeTimer = window.setTimeout(() => {
          this.notice = null
          this.noticeTimer = null
        }, NOTICE_CLEAR_DELAY)
      }
    },
    clearNotice() {
      this.notice = null
      if (this.noticeTimer !== null && typeof window !== 'undefined') {
        window.clearTimeout(this.noticeTimer)
        this.noticeTimer = null
      }
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
      if (task.status === 'waiting' && task.resources.every((resource) => resource.status === 'pending')) {
        delete this.progressByTask[task.id]
      }
      this.applyDefaultFilter()
      this.selectedTaskId = this.selectedTaskId ?? task.id
    },
    applyCreatedTasks(tasks: DownloadTask[]) {
      for (const task of tasks) {
        this.upsertTask(task)
      }
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

      const transferProgress = this.progressByTask[task.id]
      if (transferProgress?.totalBytes && transferProgress.totalBytes > 0) {
        return Math.min(99, Math.round((transferProgress.downloadedBytes / transferProgress.totalBytes) * 100))
      }

      if (task.resources.length === 0) {
        return 0
      }

      const completed = task.resources.filter((resource) => resource.status === 'completed').length
      return Math.round((completed / task.resources.length) * 100)
    },
    taskTransferProgress(taskId: string): TransferProgressSnapshot | null {
      return this.progressByTask[taskId] ?? null
    },
    totalSpeedBytesPerSecond(): number {
      const activeTaskIds = new Set(
        this.tasks
          .filter((task) => task.status === 'downloading')
          .map((task) => task.id),
      )

      return Object.entries(this.progressByTask).reduce((total, [taskId, progress]) => {
        if (!activeTaskIds.has(taskId)) {
          return total
        }

        return total + progress.speedBytesPerSecond
      }, 0)
    },
    applyProgress(entry: QueueProgressEntry) {
      const existing = this.progressByTask[entry.task_id]
      const resources = {
        ...(existing?.resources ?? {}),
        [entry.resource_id]: {
          downloadedBytes: entry.downloaded_bytes,
          totalBytes: entry.total_bytes,
        },
      }
      const downloadedBytes = sumDownloadedBytes(resources)
      const totalBytes = sumKnownTotalBytes(resources)
      const updatedAt = parseEventTime(entry.created_at)
      const previousDownloaded = existing?.downloadedBytes ?? downloadedBytes
      const previousSamples = downloadedBytes < previousDownloaded ? [] : (existing?.speedSamples ?? [])
      const speedSamples = [
        ...previousSamples.filter((sample) => sample.at >= updatedAt - SPEED_SAMPLE_WINDOW_MS),
        { downloadedBytes, at: updatedAt },
      ]
      const baseline = speedSamples.find((sample) => updatedAt - sample.at >= MIN_SPEED_SAMPLE_INTERVAL_MS)
      const elapsedSeconds = baseline ? (updatedAt - baseline.at) / 1000 : 0
      const downloadedDelta = baseline ? downloadedBytes - baseline.downloadedBytes : 0
      const speedBytesPerSecond = elapsedSeconds > 0 && downloadedDelta >= 0
        ? downloadedDelta / elapsedSeconds
        : existing?.speedBytesPerSecond ?? 0

      this.progressByTask[entry.task_id] = {
        resources,
        speedSamples,
        downloadedBytes,
        totalBytes,
        speedBytesPerSecond,
        updatedAt,
      }
    },
    async loadStartupRecovery(): Promise<StartupRecoverySnapshot | null> {
      const ui = useUiStore()
      this.startupRecoveryLoading = true
      try {
        const recovery = await queueStartupRecovery()
        this.startupRecovery = recovery
        return recovery
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
        return null
      } finally {
        this.startupRecoveryLoading = false
      }
    },
    async resumeStartupRecovery() {
      const taskIds = this.startupRecovery?.task_ids ?? []
      if (taskIds.length === 0) {
        return
      }

      await this.bulkResume(taskIds)
      await this.dismissStartupRecovery(false)
    },
    async dismissStartupRecovery(showError = true) {
      const ui = useUiStore()
      this.startupRecoveryDismissed = true
      try {
        this.startupRecovery = await queueDismissStartupRecovery()
      } catch (error) {
        if (showError) {
          ui.pushToast(errorMessage(error), 'danger')
        }
      }
    },
    async pause(taskId: string) {
      await this.runTaskCommand(() => queuePause(taskId), '已暂停')
    },
    async resume(taskId: string) {
      await this.runTaskCommand(() => queueResume(taskId), '已恢复到队列')
    },
    async unschedule(taskId: string) {
      await this.runTaskCommand(() => queueUnschedule(taskId), '已取消定时并恢复到队列')
    },
    async schedule(taskId: string, scheduledAt: string): Promise<boolean> {
      return this.runTaskCommand(() => queueSchedule(taskId, scheduledAt), '已更新任务开始时间')
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
    async bulkCancel(taskIds: string[]) {
      await this.runBulkCommand(() => queueBulkCancel({ task_ids: taskIds }), '取消')
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
        delete this.progressByTask[taskId]
        this.ensureSelectedTask(true)
        if (this.selectedTaskId) {
          void this.loadLogs(this.selectedTaskId)
        }
        this.setNotice('已移除任务', 'info')
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
    async copySource(taskId: string) {
      const ui = useUiStore()
      const task = this.tasks.find((task) => task.id === taskId)
      if (!task) {
        this.setNotice('任务不存在', 'warning')
        return
      }

      try {
        await navigator.clipboard.writeText(sourceReference(task.source_id))
        this.setNotice('已复制来源', 'success')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      }
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
    async runTaskCommand(command: () => Promise<DownloadTask>, successMessage: string): Promise<boolean> {
      const ui = useUiStore()
      try {
        const task = await command()
        this.upsertTask(task)
        this.setNotice(successMessage, 'success')
        return true
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
        return false
      }
    },
    async runBulkCommand(command: () => Promise<BulkQueueResult>, actionLabel: string) {
      const ui = useUiStore()
      try {
        const result = await command()
        this.applyBulkResult(result)

        const succeeded = result.updated.length + result.removed.length
        if (succeeded > 0) {
          const suffix = result.failed.length ? `，${result.failed.length} 个失败` : ''
          this.setNotice(`${actionLabel} ${succeeded} 个任务${suffix}`, result.failed.length ? 'warning' : 'success')
        } else if (result.failed.length > 0) {
          this.setNotice(`${result.failed.length} 个任务处理失败`, 'warning')
        } else {
          this.setNotice('没有可处理的任务', 'info')
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
          delete this.progressByTask[taskId]
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

const sumDownloadedBytes = (resources: Record<string, ResourceProgressState>): number =>
  Object.values(resources).reduce((total, resource) => total + resource.downloadedBytes, 0)

const sumKnownTotalBytes = (resources: Record<string, ResourceProgressState>): number | null => {
  const values = Object.values(resources)
    .map((resource) => resource.totalBytes)
    .filter((value): value is number => typeof value === 'number')

  if (!values.length) {
    return null
  }

  return values.reduce((total, value) => total + value, 0)
}

const parseEventTime = (value: string): number => {
  const parsed = Date.parse(value)
  return Number.isNaN(parsed) ? Date.now() : parsed
}

export const sourceReference = (sourceId: string): string => {
  if (sourceId.startsWith('video:')) {
    const id = sourceId.slice('video:'.length)
    if (/^BV/i.test(id) || /^av\d+/i.test(id)) {
      return `https://www.bilibili.com/video/${id}`
    }
  }

  const uploader = sourceId.match(/^uploader:(\d+):videos$/)
  if (uploader) {
    return `https://space.bilibili.com/${uploader[1]}/video`
  }

  const bangumi = sourceId.match(/^bangumi:(\d+)$/)
  if (bangumi) {
    return `https://www.bilibili.com/bangumi/play/ss${bangumi[1]}`
  }

  const cheese = sourceId.match(/^cheese:(\d+)$/)
  if (cheese) {
    return `https://www.bilibili.com/cheese/play/ss${cheese[1]}`
  }

  return sourceId
}

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
