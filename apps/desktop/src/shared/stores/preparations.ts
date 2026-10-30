import { defineStore } from 'pinia'
import { preparationCancel, preparationConfirm, preparationDismiss, preparationList, selectionSubmitTasks, type SelectionCreateTasksRequest } from '../api/tauri'
import type { DuplicateTaskPolicy, NormalizedSourceTree, PreparationJob } from '../api/dto'
import { commandDiagnostic } from '../utils/commandDiagnostic'
import { useQueueStore } from './queue'

export const usePreparationsStore = defineStore('preparations', {
  state: () => ({
    jobs: [] as PreparationJob[],
    submitting: 0,
    busyIds: [] as string[],
    error: null as string | null,
    detail: null as string | null,
    timer: null as ReturnType<typeof setTimeout> | null,
    generation: 0,
    watching: false,
  }),
  getters: {
    pending: state => state.jobs.filter(job => ['queued', 'preparing', 'awaiting_confirmation'].includes(job.status)),
  },
  actions: {
    upsert(job: PreparationJob) {
      // A list requested before this acknowledgement cannot replace it.
      this.generation++
      this.jobs = [job, ...this.jobs.filter(value => value.id !== job.id)]
    },
    async submit(request: SelectionCreateTasksRequest, snapshot: NormalizedSourceTree, origin: string) {
      this.submitting++
      try {
        const job = await selectionSubmitTasks(request, snapshot, origin)
        this.upsert(job)
        this.schedule()
        return job
      } finally { this.submitting-- }
    },
    start() { this.watching = true; void this.load() },
    stop() {
      this.watching = false
      this.generation++
      if (this.timer) clearTimeout(this.timer)
      this.timer = null
    },
    schedule() {
      if (!this.watching || this.timer) return
      if (!this.pending.length) return
      this.timer = setTimeout(() => { this.timer = null; void this.load() }, 750)
    },
    async load() {
      const generation = ++this.generation
      try {
        const jobs = await preparationList()
        if (generation !== this.generation) return
        const completed = jobs.some(job => job.status === 'completed' && this.jobs.find(previous => previous.id === job.id)?.status !== 'completed')
        this.jobs = jobs
        this.error = null
        this.detail = null
        if (completed) void useQueueStore().list()
      } catch (error) {
        if (generation !== this.generation) return
        this.error = '暂时无法读取任务准备状态'
        this.detail = commandDiagnostic(error, '操作: 读取任务准备状态')
      } finally { if (generation === this.generation) this.schedule() }
    },
    async act(id: string, action: 'cancel' | 'dismiss' | DuplicateTaskPolicy) {
      if (this.busyIds.includes(id)) return
      this.busyIds.push(id)
      try {
        this.generation++
        if (action === 'cancel') await preparationCancel(id)
        else if (action === 'dismiss') await preparationDismiss(id)
        else await preparationConfirm(id, action)
        await this.load()
      } catch (error) {
        await this.load()
        this.error = '操作未完成，请重试'
        this.detail = commandDiagnostic(error, `操作: 处理准备任务\n任务: ${id}\n动作: ${action}`)
      } finally { this.busyIds = this.busyIds.filter(value => value !== id) }
    },
  },
})
