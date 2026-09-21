import { defineStore } from 'pinia'

import type {
  DownloadMediaMode,
  DuplicateTaskPolicy,
  NormalizedGroup,
  NormalizedItem,
  NormalizedPart,
  NormalizedSourceTree,
  SettingsSnapshot,
  SelectionCreateTasksResult,
} from '../api/dto'
import {
  parseCloseSource,
  parseCreateSource,
  parseLoadMore,
  parseCancel,
  selectionCreateTasks,
} from '../api/tauri'
import { useQueueStore } from './queue'
import { useSettingsStore } from './settings'
import type { InlineNotice, NoticeTone } from './feedback'
import { NOTICE_CLEAR_DELAY } from './feedback'
import { buildBatchSelectionBySource, selectedBatchSourceIds, toggleBatchEntrySelection } from './parseBatch'

const MAX_BATCH_SOURCES = 20
const PARSE_CONCURRENCY = 1
const PACED_PARSE_POLL_INTERVAL_MS = 100

interface ParseState {
  backgroundJob: { sourceId: string; title: string; status: 'running' | 'stopped' | 'completed' | 'failed'; processed: number; created: number; error: string | null } | null
  input: string
  sources: Record<string, NormalizedSourceTree>
  sourceOrder: string[]
  activeSourceId: string | null
  batchMode: boolean
  batchEntries: ParseBatchEntry[]
  selectedBatchEntryIds: string[]
  selectionBySource: Record<string, string[]>
  loadingBySource: Record<string, boolean>
  pacedParsingBySource: Record<string, boolean>
  pacedParsingWaitingBySource: Record<string, boolean>
  pacedParsingStopRequestedBySource: Record<string, boolean>
  errorsBySource: Record<string, string | null>
  notice: InlineNotice | null
  noticeTimer: number | null
}

export interface ParseBatchEntry {
  id: string
  sourceId: string
  partId: string
  title: string
  input: string
}

export interface CreateTaskOptions {
  partIdsBySource?: Record<string, string[]>
  silent?: boolean
  downloadDir?: string | null
  archiveMode?: SettingsSnapshot['archive_mode']
  outputExtension?: SettingsSnapshot['output_extension']
  namingTemplate?: string
  duplicateNamingStrategy?: SettingsSnapshot['duplicate_naming_strategy']
  archiveAssets?: SettingsSnapshot['archive_assets']
  retainRawStreams?: boolean
  embedCover?: boolean
  embedSubtitles?: boolean
  missingQualityPolicy?: SettingsSnapshot['missing_quality_policy']
  mediaMode?: DownloadMediaMode
  quality?: string
  audioQuality?: string
  codec?: SettingsSnapshot['codec']
  mediaPreferences?: SettingsSnapshot['media_preferences']
  duplicatePolicy?: DuplicateTaskPolicy
  scheduledAt?: string
  speedLimitBytesPerSecond?: number
}

export interface CreateTasksForSourcesResult extends SelectionCreateTasksResult {
  pendingSourceIds: string[]
  failedSourceIds: string[]
  failures: Array<{ sourceId: string; message: string }>
}

export type PacedParsingResult = 'completed' | 'stopped' | 'failed'

export const useParseStore = defineStore('parse', {
  state: (): ParseState => ({
    backgroundJob: null,
    input: '',
    sources: {},
    sourceOrder: [],
    activeSourceId: null,
    batchMode: false,
    batchEntries: [],
    selectedBatchEntryIds: [],
    selectionBySource: {},
    loadingBySource: {},
    pacedParsingBySource: {},
    pacedParsingWaitingBySource: {},
    pacedParsingStopRequestedBySource: {},
    errorsBySource: {},
    notice: null,
    noticeTimer: null,
  }),
  getters: {
    orderedSources(state): NormalizedSourceTree[] {
      return state.sourceOrder.map((id) => state.sources[id]).filter(Boolean)
    },
    activeSource(state): NormalizedSourceTree | null {
      return state.activeSourceId ? (state.sources[state.activeSourceId] ?? null) : null
    },
    activeSelection(state): string[] {
      return state.activeSourceId ? (state.selectionBySource[state.activeSourceId] ?? []) : []
    },
    selectedSourceIds(state): string[] {
      if (state.batchMode) {
        return selectedBatchSourceIds(state.batchEntries, state.selectedBatchEntryIds)
      }
      return state.sourceOrder.filter((sourceId) => (state.selectionBySource[sourceId]?.length ?? 0) > 0)
    },
    isBatch(state): boolean {
      return state.batchMode
    },
  },
  actions: {
    setNotice(message: string, tone: NoticeTone = 'info', actionLabel?: string) {
      this.notice = { message, tone, actionLabel }
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
    async createSource(input?: string): Promise<boolean> {
      const inputs = splitParseInputs(input ?? this.input)

      if (inputs.length === 0) {
        this.setNotice('请输入链接或 BV/AV', 'warning')
        return false
      }

      if (inputs.length > MAX_BATCH_SOURCES) {
        this.setNotice(`一次最多解析 ${MAX_BATCH_SOURCES} 个链接`, 'warning')
        return false
      }

      const batch = inputs.length > 1
      this.loadingBySource.__create__ = true
      try {
        const outcomes = await mapWithConcurrency(inputs, PARSE_CONCURRENCY, async (sourceInput) => {
          try {
            const tree = await parseCreateSource({
              input: sourceInput,
              fetch_streams: false,
              expand_video_collection: !batch,
            })
            return { input: sourceInput, tree, error: null }
          } catch (error) {
            return { input: sourceInput, tree: null, error: errorMessage(error) }
          }
        })
        const failures = outcomes.filter((outcome) => !outcome.tree)
        const validOutcomes: Array<(typeof outcomes)[number] & { tree: NormalizedSourceTree }> = []
        for (const outcome of outcomes) {
          if (!outcome.tree) continue
          if (batch && outcome.tree.source.kind !== 'video') {
            failures.push({ ...outcome, tree: null, error: '批量模式只支持视频链接' })
            if (!this.sources[outcome.tree.source.id]) {
              try {
                await parseCloseSource(outcome.tree.source.id)
              } catch {
                // The rejected container is not retained in the local workspace.
              }
            }
            continue
          }
          validOutcomes.push({ ...outcome, tree: outcome.tree })
        }
        const trees = uniqueSourceTrees(validOutcomes.map((outcome) => outcome.tree))
        if (trees.length === 0) {
          this.setNotice(failures[0]?.error ?? '解析失败', 'danger')
          return false
        }

        const nextSourceIds = new Set(trees.map((tree) => tree.source.id))
        const previousSourceIds = this.sourceOrder.filter((sourceId) => !nextSourceIds.has(sourceId))
        for (const sourceId of previousSourceIds) {
          await this.removeSource(sourceId)
        }

        for (const tree of trees) {
          const sourceId = tree.source.id
          this.sources[sourceId] = tree
          this.errorsBySource[sourceId] = null
          this.selectionBySource[sourceId] = batch ? [] : defaultSelection(tree)
        }
        this.sourceOrder = trees.map((tree) => tree.source.id)
        this.activeSourceId = this.sourceOrder[0] ?? null
        this.batchMode = batch
        this.batchEntries = batch
          ? validOutcomes.flatMap((outcome, index) => {
              const part = batchPart(outcome.tree, outcome.input)
              return part
                ? [
                    {
                      id: `batch:${index}:${outcome.tree.source.id}:${part.id}`,
                      sourceId: outcome.tree.source.id,
                      partId: part.id,
                      title:
                        outcome.tree.groups.flatMap((group) => group.items).flatMap((item) => item.parts).length > 1
                          ? part.title
                          : outcome.tree.source.title,
                      input: outcome.input,
                    },
                  ]
                : []
            })
          : []
        this.selectedBatchEntryIds = this.batchEntries.map((entry) => entry.id)
        this.syncBatchSelection()
        this.input = failures.map((failure) => failure.input).join('\n')
        if (failures.length > 0) {
          this.setNotice(`已解析 ${validOutcomes.length} 个链接，${failures.length} 个失败已保留`, 'warning')
        } else {
          this.clearNotice()
        }
        return true
      } finally {
        this.loadingBySource.__create__ = false
      }
    },
    async loadMore(sourceId: string) {
      this.pacedParsingStopRequestedBySource[sourceId] = false
      this.loadingBySource[sourceId] = true
      try {
        const tree = await parseLoadMore({ source_id: sourceId })
        this.upsertSource(tree)
      } catch (error) {
        if (!this.pacedParsingStopRequestedBySource[sourceId]) this.errorsBySource[sourceId] = errorMessage(error)
      } finally {
        this.loadingBySource[sourceId] = false
      }
    },
    async loadChunk(sourceId: string, chunkSize: number): Promise<boolean> {
      const current = this.sources[sourceId]
      if (!current?.source.has_more || this.loadingBySource[sourceId]) return false
      const loadedBefore = current.source.loaded_count
      const limit = loadedBefore + Math.max(1, Math.floor(chunkSize))
      if (!this.pacedParsingBySource[sourceId]) this.pacedParsingStopRequestedBySource[sourceId] = false
      this.loadingBySource[sourceId] = true
      this.errorsBySource[sourceId] = null
      try {
        while (this.sources[sourceId]?.source.has_more && this.sources[sourceId].source.loaded_count < limit) {
          if (this.pacedParsingStopRequestedBySource[sourceId]) break
          const before = this.sources[sourceId].source.loaded_count
          const tree = await parseLoadMore({ source_id: sourceId })
          if (!this.sources[sourceId]) break
          this.upsertSource(tree)
          if (tree.source.loaded_count <= before) break
        }
        return (this.sources[sourceId]?.source.loaded_count ?? 0) > loadedBefore
      } catch (error) {
        if (!this.pacedParsingStopRequestedBySource[sourceId]) this.errorsBySource[sourceId] = errorMessage(error)
        return false
      } finally {
        this.loadingBySource[sourceId] = false
      }
    },
    async startBackgroundDownload(sourceId: string): Promise<void> {
      if (this.backgroundJob?.status === 'running' || this.pacedParsingBySource[sourceId] || this.loadingBySource[sourceId]) return
      const source = this.sources[sourceId]
      if (!source) return
      const job = { sourceId, title: source.source.title, status: 'running' as 'running' | 'stopped' | 'completed' | 'failed', processed: 0, created: 0, error: null as string | null }
      this.backgroundJob = job
      this.pacedParsingBySource[sourceId] = true
      this.pacedParsingStopRequestedBySource[sourceId] = false
      this.errorsBySource[sourceId] = null
      const done = new Set<string>()
      const finishedItems = new Set<string>()
      const stopped = () => Boolean(this.pacedParsingStopRequestedBySource[sourceId]) || !this.sources[sourceId]
      try {
        const settings = useSettingsStore()
        await settings.ensureLoaded()
        const defaults: SettingsSnapshot = JSON.parse(JSON.stringify(settings.saved))
        const options: CreateTaskOptions = {
          downloadDir: defaults.download_dir ?? 'downloads', archiveMode: defaults.archive_mode,
          outputExtension: defaults.output_extension, namingTemplate: defaults.naming_template,
          duplicateNamingStrategy: defaults.duplicate_naming_strategy, archiveAssets: defaults.archive_assets,
          retainRawStreams: defaults.retain_raw_streams, embedCover: defaults.embed_cover,
          embedSubtitles: defaults.embed_subtitles, missingQualityPolicy: defaults.missing_quality_policy,
          quality: defaults.media_preferences.video.length ? 'best' : defaults.quality,
          codec: defaults.media_preferences.video.length ? 'auto' : defaults.codec,
          audioQuality: defaults.media_preferences.audio.length ? 'best' : defaults.audio_quality,
          mediaPreferences: defaults.media_preferences, duplicatePolicy: 'skip', silent: true,
        }
        while (!stopped()) {
          const pending = this.sources[sourceId].groups.flatMap((group) => group.items).filter((item) => !finishedItems.has(item.id)).flatMap((item) => item.parts.map((part) => part.id)).filter((id) => !done.has(id))
          // Enqueue each resolved selection immediately so downloading starts early.
          for (let index = 0; index < pending.length && !stopped(); index += 1) {
            const ids = pending.slice(index, index + 1)
            const beforeIds = new Set(collectPartIds(this.sources[sourceId]))
            const result = await this.createTasksForSources([sourceId], { ...options, partIdsBySource: { [sourceId]: ids } })
            if (stopped() && (!result || result.failures.length)) break
            if (!result || result.failures.length || result.requires_confirmation) throw new Error(result?.failures[0]?.message ?? '后台创建下载任务失败')
            ids.forEach((id) => done.add(id))
            // Hydration can replace placeholder IDs with real part IDs.
            collectPartIds(this.sources[sourceId]).filter((id) => !beforeIds.has(id)).forEach((id) => done.add(id))
            if (this.backgroundJob) {
              this.backgroundJob.processed += ids.length
              this.backgroundJob.created += result.created.length
            }
          }
          if (this.sources[sourceId]) {
            for (const item of this.sources[sourceId].groups.flatMap((group) => group.items)) {
              if (item.parts.every((part) => done.has(part.id))) finishedItems.add(item.id)
            }
          }
          if (stopped() || !this.sources[sourceId]?.source.has_more) break
          const progressed = await this.loadChunk(sourceId, 1)
          if (!progressed && !stopped()) throw new Error(this.errorsBySource[sourceId] ?? '分页没有返回新内容，已停止后台解析')
        }
        if (this.backgroundJob) this.backgroundJob.status = stopped() ? 'stopped' : 'completed'
      } catch (error) {
        if (this.backgroundJob) {
          this.backgroundJob.status = stopped() ? 'stopped' : 'failed'
          this.backgroundJob.error = stopped() ? null : errorMessage(error)
        }
        if (!stopped()) this.errorsBySource[sourceId] = errorMessage(error)
      } finally {
        this.pacedParsingBySource[sourceId] = false
        this.pacedParsingStopRequestedBySource[sourceId] = false
        this.loadingBySource[sourceId] = false
      }
    },
    async parseAllPaced(sourceId: string, chunkSize = 50, delayMs = 0): Promise<PacedParsingResult> {
      const source = this.sources[sourceId]
      if (!source?.source.has_more) return 'completed'
      if (this.pacedParsingBySource[sourceId]) return 'stopped'

      this.pacedParsingBySource[sourceId] = true
      this.pacedParsingWaitingBySource[sourceId] = false
      this.pacedParsingStopRequestedBySource[sourceId] = false
      try {
        while (this.sources[sourceId]?.source.has_more) {
          if (this.pacedParsingStopRequestedBySource[sourceId]) {
            return 'stopped'
          }

          const progressed = await this.loadChunk(sourceId, chunkSize)
          if (!progressed) {
            return this.pacedParsingStopRequestedBySource[sourceId] ? 'stopped' : this.sources[sourceId]?.source.has_more ? 'failed' : 'completed'
          }
          if (!this.sources[sourceId]?.source.has_more) break

          this.pacedParsingWaitingBySource[sourceId] = true
          const shouldContinue = await waitForPacedParsing(delayMs, () =>
            Boolean(this.pacedParsingStopRequestedBySource[sourceId]),
          )
          this.pacedParsingWaitingBySource[sourceId] = false
          if (!shouldContinue) {
            return 'stopped'
          }
        }

        return 'completed'
      } finally {
        this.pacedParsingBySource[sourceId] = false
        this.pacedParsingWaitingBySource[sourceId] = false
        this.pacedParsingStopRequestedBySource[sourceId] = false
      }
    },
    stopPacedParsing(sourceId: string) {
      if (this.pacedParsingBySource[sourceId] || this.loadingBySource[sourceId]) {
        this.pacedParsingStopRequestedBySource[sourceId] = true
        void parseCancel(sourceId).catch((error) => {
          this.setNotice(`停止请求未送达：${errorMessage(error)}`, 'warning')
        })
      }
    },
    async clearWorkspace() {
      if (this.backgroundJob?.status === 'running') {
        this.setNotice('请先停止后台解析下载', 'warning')
        return
      }
      const sourceIds = [...this.sourceOrder]
      sourceIds.forEach((sourceId) => this.stopPacedParsing(sourceId))

      this.input = ''
      this.clearNotice()
      this.sources = {}
      this.sourceOrder = []
      this.activeSourceId = null
      this.batchMode = false
      this.batchEntries = []
      this.selectedBatchEntryIds = []
      this.selectionBySource = {}
      this.loadingBySource = {}
      this.pacedParsingBySource = {}
      this.pacedParsingWaitingBySource = {}
      this.pacedParsingStopRequestedBySource = {}
      this.errorsBySource = {}

      await Promise.allSettled(sourceIds.map((sourceId) => parseCloseSource(sourceId)))
    },
    async removeSource(sourceId: string) {
      if (this.backgroundJob?.status === 'running' && this.backgroundJob.sourceId === sourceId) return
      try {
        await parseCloseSource(sourceId)
      } finally {
        delete this.sources[sourceId]
        delete this.selectionBySource[sourceId]
        delete this.errorsBySource[sourceId]
        delete this.pacedParsingBySource[sourceId]
        delete this.pacedParsingWaitingBySource[sourceId]
        delete this.pacedParsingStopRequestedBySource[sourceId]
        this.sourceOrder = this.sourceOrder.filter((id) => id !== sourceId)
        this.batchEntries = this.batchEntries.filter((entry) => entry.sourceId !== sourceId)
        this.selectedBatchEntryIds = this.selectedBatchEntryIds.filter((entryId) =>
          this.batchEntries.some((entry) => entry.id === entryId),
        )
        if (this.activeSourceId === sourceId) {
          this.activeSourceId = this.sourceOrder[0] ?? null
        }
      }
    },
    toggleNode(sourceId: string, nodeId: string) {
      const tree = this.sources[sourceId]
      if (!tree) {
        return
      }

      const partIds = partIdsForNode(tree, nodeId)
      if (partIds.length === 0) {
        return
      }

      const current = new Set(this.selectionBySource[sourceId] ?? [])
      const allSelected = partIds.every((id) => current.has(id))
      partIds.forEach((id) => {
        if (allSelected) {
          current.delete(id)
        } else {
          current.add(id)
        }
      })
      this.selectionBySource[sourceId] = [...current]
    },
    selectAllLoaded(sourceId: string) {
      const tree = this.sources[sourceId]
      if (!tree) {
        return
      }

      this.selectionBySource[sourceId] = collectPartIds(tree)
    },
    clearSelection(sourceId: string) {
      if (this.sources[sourceId]) {
        this.selectionBySource[sourceId] = []
      }
    },
    toggleBatchEntry(entryId: string) {
      this.selectedBatchEntryIds = toggleBatchEntrySelection(this.batchEntries, this.selectedBatchEntryIds, entryId)
      this.syncBatchSelection()
    },
    selectAllBatchEntries() {
      this.selectedBatchEntryIds = this.batchEntries.map((entry) => entry.id)
      this.syncBatchSelection()
    },
    clearBatchSelection() {
      this.selectedBatchEntryIds = []
      this.syncBatchSelection()
    },
    syncBatchSelection() {
      this.selectionBySource = buildBatchSelectionBySource(
        this.sourceOrder,
        this.batchEntries,
        this.selectedBatchEntryIds,
      )
    },
    async removeBatchEntry(entryId: string) {
      const entry = this.batchEntries.find((candidate) => candidate.id === entryId)
      if (!entry) return
      this.batchEntries = this.batchEntries.filter((candidate) => candidate.id !== entryId)
      this.selectedBatchEntryIds = this.selectedBatchEntryIds.filter((candidate) => candidate !== entryId)
      if (!this.batchEntries.some((candidate) => candidate.sourceId === entry.sourceId)) {
        await this.removeSource(entry.sourceId)
      }
      this.batchMode = this.batchEntries.length > 0
      this.syncBatchSelection()
    },
    selectPartIds(sourceId: string, partIds: string[], mode: 'add' | 'replace' = 'add') {
      if (!this.sources[sourceId]) {
        return
      }

      if (mode === 'replace') {
        this.selectionBySource[sourceId] = uniquePartIds(partIds)
        return
      }

      this.selectionBySource[sourceId] = uniquePartIds([...(this.selectionBySource[sourceId] ?? []), ...partIds])
    },
    async createTasksForSelection(
      sourceId: string,
      options: CreateTaskOptions = {},
    ): Promise<CreateTasksForSourcesResult | null> {
      return this.createTasksForSources([sourceId], options)
    },
    async createTasksForSources(
      sourceIds: string[],
      options: CreateTaskOptions = {},
    ): Promise<CreateTasksForSourcesResult | null> {
      const settings = useSettingsStore()
      const queue = useQueueStore()
      const targets = uniquePartIds(sourceIds).filter((sourceId) => ((options.partIdsBySource?.[sourceId] ?? this.selectionBySource[sourceId])?.length ?? 0) > 0)

      if (targets.length === 0) {
        this.setNotice('请选择要下载的内容', 'warning')
        return null
      }

      targets.forEach((sourceId) => {
        this.loadingBySource[sourceId] = true
      })
      try {
        await settings.ensureLoaded()
        const downloadDir =
          options.downloadDir !== undefined ? options.downloadDir?.trim() : settings.saved.download_dir?.trim()
        const aggregate: CreateTasksForSourcesResult = {
          created: [],
          duplicates: [],
          skipped_existing: 0,
          requires_confirmation: false,
          pendingSourceIds: [],
          failedSourceIds: [],
          failures: [],
        }

        for (const sourceId of targets) {
          if (this.pacedParsingBySource[sourceId] && this.pacedParsingStopRequestedBySource[sourceId]) break
          try {
            const result = await selectionCreateTasks({
              source_id: sourceId,
              part_ids: options.partIdsBySource?.[sourceId] ?? this.selectionBySource[sourceId] ?? [],
              output_dir: downloadDir || undefined,
              archive_mode: options.archiveMode ?? settings.saved.archive_mode,
              output_extension: options.outputExtension ?? settings.saved.output_extension,
              naming_template: options.namingTemplate ?? settings.saved.naming_template,
              duplicate_naming_strategy: options.duplicateNamingStrategy ?? settings.saved.duplicate_naming_strategy,
              archive_assets: options.archiveAssets ?? settings.saved.archive_assets,
              retain_raw_streams: options.retainRawStreams ?? settings.saved.retain_raw_streams,
              embed_cover: options.embedCover ?? settings.saved.embed_cover,
              embed_subtitles: options.embedSubtitles ?? settings.saved.embed_subtitles,
              missing_quality_policy: options.missingQualityPolicy ?? settings.saved.missing_quality_policy,
              media_mode: options.mediaMode ?? 'audio_video',
              quality: options.quality ?? settings.saved.quality,
              audio_quality: options.audioQuality ?? settings.saved.audio_quality,
              codec: options.codec ?? settings.saved.codec,
              media_preferences: options.mediaPreferences ?? settings.saved.media_preferences,
              duplicate_policy: options.duplicatePolicy ?? 'ask',
              scheduled_at: options.scheduledAt,
              speed_limit_bytes_per_second: options.speedLimitBytesPerSecond,
            })
            aggregate.created.push(...result.created)
            aggregate.duplicates.push(...result.duplicates)
            aggregate.skipped_existing += result.skipped_existing
            if (result.requires_confirmation) {
              aggregate.requires_confirmation = true
              aggregate.pendingSourceIds.push(sourceId)
            }
            queue.applyCreatedTasks(result.created)
          } catch (error) {
            const message = errorMessage(error)
            aggregate.failedSourceIds.push(sourceId)
            aggregate.failures.push({ sourceId, message })
          }
        }

        if (aggregate.requires_confirmation) {
          return aggregate
        }
        if (options.silent) return aggregate
        const duplicateSuffix = aggregate.duplicates.length > 0 ? `，处理 ${aggregate.duplicates.length} 个重复项` : ''
        const skippedSuffix = aggregate.skipped_existing > 0 ? `，跳过 ${aggregate.skipped_existing} 个已有文件` : ''
        if (aggregate.created.length > 0) {
          this.setNotice(
            `已创建 ${aggregate.created.length} 个任务${skippedSuffix}${duplicateSuffix}`,
            'success',
            '查看传输',
          )
        } else if (aggregate.skipped_existing > 0 && aggregate.failedSourceIds.length === 0) {
          this.setNotice(`已跳过 ${aggregate.skipped_existing} 个已有文件`, 'info')
        } else if (aggregate.failedSourceIds.length === 0) {
          this.setNotice('所选内容已在传输中', 'info', '查看传输')
        }
        return aggregate
      } finally {
        targets.forEach((sourceId) => {
          this.loadingBySource[sourceId] = false
        })
      }
    },
    upsertSource(tree: NormalizedSourceTree) {
      const sourceId = tree.source.id
      this.sources[sourceId] = tree
      if (!this.sourceOrder.includes(sourceId)) {
        this.sourceOrder.unshift(sourceId)
      }
      if (!this.activeSourceId) this.activeSourceId = sourceId
      this.errorsBySource[sourceId] = null
      this.selectionBySource[sourceId] = this.selectionBySource[sourceId] ?? defaultSelection(tree)
    },
  },
})

const collectPartIds = (tree: NormalizedSourceTree): string[] =>
  tree.groups.flatMap((group) => group.items.flatMap((item) => item.parts.map((part) => part.id)))

const uniquePartIds = (partIds: string[]): string[] => Array.from(new Set(partIds))

const waitForPacedParsing = async (delayMs: number, stopRequested: () => boolean): Promise<boolean> => {
  let remaining = Math.max(0, delayMs)
  while (remaining > 0) {
    if (stopRequested()) return false
    const waitMs = Math.min(PACED_PARSE_POLL_INTERVAL_MS, remaining)
    await new Promise<void>((resolve) => window.setTimeout(resolve, waitMs))
    remaining -= waitMs
  }
  return !stopRequested()
}

const uniqueSourceTrees = (trees: NormalizedSourceTree[]): NormalizedSourceTree[] => {
  const seen = new Set<string>()
  return trees.filter((tree) => {
    if (seen.has(tree.source.id)) {
      return false
    }
    seen.add(tree.source.id)
    return true
  })
}

const batchPart = (tree: NormalizedSourceTree, input: string): NormalizedPart | null => {
  const parts = tree.groups.flatMap((group) => group.items.flatMap((item) => item.parts))
  if (parts.length === 0) {
    return null
  }

  const requestedPart = /[?&]p=(\d+)/i.exec(input)?.[1]
  const requestedIndex = requestedPart ? Number.parseInt(requestedPart, 10) - 1 : 0
  return parts[requestedIndex] ?? parts[0]
}

const mapWithConcurrency = async <Input, Output>(
  inputs: Input[],
  concurrency: number,
  mapper: (input: Input) => Promise<Output>,
): Promise<Output[]> => {
  const results = new Array<Output>(inputs.length)
  let nextIndex = 0
  const workers = Array.from({ length: Math.min(concurrency, inputs.length) }, async () => {
    while (nextIndex < inputs.length) {
      const index = nextIndex
      nextIndex += 1
      results[index] = await mapper(inputs[index])
    }
  })
  await Promise.all(workers)
  return results
}

const splitParseInputs = (input: string): string[] =>
  Array.from(
    new Set(
      input
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean),
    ),
  )

const defaultSelection = (tree: NormalizedSourceTree): string[] =>
  tree.source.kind === 'video' ? collectPartIds(tree) : []

const partIdsForNode = (tree: NormalizedSourceTree, nodeId: string): string[] => {
  for (const group of tree.groups) {
    if (group.id === nodeId) {
      return partIdsForGroup(group)
    }

    for (const item of group.items) {
      if (item.id === nodeId) {
        return partIdsForItem(item)
      }

      const part = item.parts.find((candidate) => candidate.id === nodeId)
      if (part) {
        return [part.id]
      }
    }
  }

  return []
}

const partIdsForGroup = (group: NormalizedGroup): string[] => group.items.flatMap(partIdsForItem)

const partIdsForItem = (item: NormalizedItem): string[] => item.parts.map((part: NormalizedPart) => part.id)

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
