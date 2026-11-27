import preparationBlockingCodes from '../../../../../crates/bdl-tauri/src/preparation-blocking-codes.json'
import { problemMessage } from '../utils/publicError'
import { defineStore } from 'pinia'

import { commandDiagnostic } from '../utils/commandDiagnostic'
import { classifyFailure } from '../utils/failureDiagnostic'

import type {
  DocumentTreeDirectory,
  DownloadMediaMode,
  DuplicateTaskPolicy,
  NormalizedItem,
  NormalizedPart,
  NormalizedSourceTree,
  SettingsSnapshot,
  SelectionCreateTasksResult,
  SelectionSizeEstimate,
} from '../api/dto'
import {
  parseCreateSource,
  parseLoadMore,
  parseLoadPage,
  parseCancel,
  type ParseCancelReason,
  type SelectionCreateTasksRequest,
  selectionCreateTasks,
  selectionEnqueueBatch,
  selectionEstimateSize,
  selectionPrepare,
} from '../api/tauri'
import { useQueueStore } from './queue'
import { useSettingsStore } from './settings'
import { useUiStore } from './ui'
import { usePreparationsStore } from './preparations'
import { isSingleVideoInput } from '../utils/bilibiliLinks'
import type { InlineNotice, NoticeTone } from './feedback'
import { NOTICE_CLEAR_DELAY } from './feedback'
import { buildBatchSelectionBySource, selectedBatchSourceIds, toggleBatchEntrySelection } from './parseBatch'
import { selectsWholePosts, selectableParts } from '../features/parse/parseResultTree'


interface BackgroundParseJob {
  sourceId: string
  title: string
  status: 'running' | 'stopped' | 'completed' | 'failed'
  processed: number
  created: number
  skipped: number
  error: string | null
  detail?: string
  failures: Array<{ partId: string; title: string; message: string; detail?: string }>
  donePartIds: string[]
  options: CreateTaskOptions
}

interface SourceOutcome {
  input: string
  tree: NormalizedSourceTree | null
  error: string | null
  detail: string
}

interface ParseState {
  operationsBySource: Record<string, string[]>
  sourceEpochById: Record<string, string>
  preparationBySource: Record<string, { total: number; processed: number; failures: Array<{ itemId: string; title: string; message: string; detail?: string; partIds: string[] }> }>
  createGeneration: number
  stoppedBatchInput: string | null
  backgroundJob: BackgroundParseJob | null
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
  pacedParsingStopRequestedBySource: Record<string, boolean>
  errorDetailsBySource: Record<string, string | null>
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
  downloadPresetId?: string
  partIdsBySource?: Record<string, string[]>
  silent?: boolean
  downloadDir?: string | null
  documentTreeOutput?: DocumentTreeDirectory | null
  archiveMode?: SettingsSnapshot['archive_mode']
  outputExtension?: 'mp4' | 'mkv' | 'm4s' | 'mp3'
  subtitleFormat?: SettingsSnapshot['subtitle_format']
  danmakuFormat?: SettingsSnapshot['danmaku_format']
  namingVersion?: 1 | 2
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
  failures: Array<{ sourceId: string; message: string; detail?: string; stopBackground?: boolean }>
}

const mediaPreferencesForTaskRequest = (
  options: CreateTaskOptions,
  settings: SettingsSnapshot,
): SettingsSnapshot['media_preferences'] => {
  const source = options.mediaPreferences ?? settings.media_preferences
  const preferences: SettingsSnapshot['media_preferences'] = {
    video: source.video.map((rule) => ({ ...rule })),
    audio: [...source.audio],
  }
  if (
    (options.quality !== undefined && options.quality !== 'best') ||
    (options.codec !== undefined && options.codec !== 'auto')
  ) {
    preferences.video = []
  }
  if (options.audioQuality !== undefined && options.audioQuality !== 'best') {
    preferences.audio = []
  }
  return preferences
}

export type PacedParsingResult = 'completed' | 'stopped' | 'failed'

export const useParseStore = defineStore('parse', {
  state: (): ParseState => ({
    operationsBySource: {},
    sourceEpochById: {},
    createGeneration: 0,
    stoppedBatchInput: null,
    preparationBySource: {},
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
    pacedParsingStopRequestedBySource: {},
    errorsBySource: {},
    errorDetailsBySource: {},
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
    beginOperation(sourceId: string) {
      const id = crypto.randomUUID()
      this.operationsBySource[sourceId] = [...(this.operationsBySource[sourceId] ?? []), id]
      return id
    },
    endOperation(sourceId: string, id: string) {
      if (this.operationsBySource[sourceId]) {
        this.operationsBySource[sourceId] = this.operationsBySource[sourceId].filter(value => value !== id)
        if (!this.operationsBySource[sourceId].length) delete this.operationsBySource[sourceId]
      }
    },
    ownsSource(sourceId: string, generation: number, epoch: string | undefined) {
      return generation === this.createGeneration && epoch !== undefined && this.sourceEpochById[sourceId] === epoch
    },
    recordSourceFailure(sourceId: string, error: unknown, operation = '解析来源') {
      if (!this.sources[sourceId]) return
      this.errorsBySource[sourceId] = errorMessage(error)
      this.errorDetailsBySource[sourceId] = commandDiagnostic(error, `操作: ${operation}\n来源: ${this.sources[sourceId].source.title}\n链接: ${this.sources[sourceId].source.input}`)
    },
    setNotice(message: string, tone: NoticeTone = 'info', actionLabel?: string, detail?: string) {
      this.notice = { message, tone, actionLabel, detail }
      if (this.noticeTimer !== null && typeof window !== 'undefined') {
        window.clearTimeout(this.noticeTimer)
        this.noticeTimer = null
      }

      if (tone !== 'danger' && !detail && typeof window !== 'undefined') {
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
    async createSource(input?: string, options: { singleVideo?: boolean; expandVideoCollection?: boolean } = {}): Promise<boolean> {
      const inputs = splitParseInputs(input ?? this.input)

      if (inputs.length === 0) {
        this.setNotice('请输入链接或 BV/AV', 'warning')
        return false
      }

      if (options.singleVideo && (inputs.length !== 1 || !isSingleVideoInput(inputs[0]))) {
        this.setNotice('请输入一个视频链接或 BV / AV；合集、收藏夹等来源请切换到批量解析。', 'warning')
        return false
      }

      const resuming = !options.singleVideo && this.stoppedBatchInput === inputs.join('\n')
      const previousEntries = resuming ? this.batchEntries : []
      const previousSelection = new Set(this.selectedBatchEntryIds)
      this.stoppedBatchInput = null
      this.pacedParsingStopRequestedBySource.__create__ = false
      const generation = ++this.createGeneration
      const batch = resuming || inputs.length > 1
      this.loadingBySource.__create__ = true
      try {
        const outcomes: SourceOutcome[] = []
        while (outcomes.length < inputs.length && !this.pacedParsingStopRequestedBySource.__create__) {
          if (generation !== this.createGeneration) return false
          const sourceInput = inputs[outcomes.length]
          const operationId = this.beginOperation('__create__')
          try {
            const tree = await parseCreateSource({
              input: sourceInput,
              fetch_streams: false,
              expand_video_collection: !batch && !options.singleVideo && options.expandVideoCollection !== false,
            }, operationId)
            outcomes.push({ input: sourceInput, tree, error: null, detail: '' })
          } catch (error) {
            const cancelled = error != null && typeof error === 'object' && 'code' in error && error.code === 'parse_cancelled'
            if (!this.pacedParsingStopRequestedBySource.__create__ || !cancelled) {
              outcomes.push({
                input: sourceInput,
                tree: null,
                error: errorMessage(error),
                detail: commandDiagnostic(error, `操作: 解析链接\n来源: ${sourceInput}`),
              })
            }
          } finally { this.endOperation('__create__', operationId) }
        }
        if (generation !== this.createGeneration) {
          return false
        }
        const failures = outcomes.filter((outcome) => !outcome.tree)
        const validOutcomes: Array<(typeof outcomes)[number] & { tree: NormalizedSourceTree }> = []
        for (const entry of previousEntries) {
          const tree = this.sources[entry.sourceId]
          if (tree) validOutcomes.push({ input: entry.input, tree, error: null, detail: '' })
        }
        for (const outcome of outcomes) {
          if (!outcome.tree) continue
          if ((batch || options.singleVideo) && outcome.tree.source.kind !== 'video') {
            failures.push({ ...outcome, tree: null, error: options.singleVideo ? '该链接不是单个视频，请切换到批量解析。' : '批量模式只支持视频链接' })
            continue
          }
          validOutcomes.push({ ...outcome, tree: outcome.tree })
        }
        const trees = uniqueSourceTrees(validOutcomes.map((outcome) => outcome.tree))
        if (trees.length === 0) {
          if (this.pacedParsingStopRequestedBySource.__create__ && failures.length === 0) return false
          this.setNotice(failures[0]?.error ?? '解析失败', 'danger', undefined, failures.map((failure) => failure.detail).filter(Boolean).join('\n\n'))
          return false
        }

        const nextSourceIds = new Set(trees.map((tree) => tree.source.id))
        const previousSourceIds = this.sourceOrder.filter((sourceId) => !nextSourceIds.has(sourceId))
        for (const sourceId of previousSourceIds) {
          await this.removeSource(sourceId)
          if (generation !== this.createGeneration) return false
        }

        if (generation !== this.createGeneration) return false
        for (const tree of trees) {
          const sourceId = tree.source.id
          this.loadingBySource[sourceId] = false
          this.pacedParsingBySource[sourceId] = false
          this.sources[sourceId] = tree
          this.sourceEpochById[sourceId] = crypto.randomUUID()
          this.errorsBySource[sourceId] = null
          this.errorDetailsBySource[sourceId] = null
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
        const unselectedInputs = new Set(previousEntries.filter(entry => !previousSelection.has(entry.id)).map(entry => entry.input))
        this.selectedBatchEntryIds = this.batchEntries.filter(entry => !unselectedInputs.has(entry.input)).map(entry => entry.id)
        this.syncBatchSelection()
        if (!options.singleVideo) this.input = [...failures.map((failure) => failure.input), ...inputs.slice(outcomes.length)].join('\n')
        if (batch && this.pacedParsingStopRequestedBySource.__create__ && this.input) this.stoppedBatchInput = this.input
        if (failures.length > 0) {
          this.setNotice(`已解析 ${validOutcomes.length} 个链接，${failures.length} 个失败已保留`, 'warning', undefined, failures.map((failure) => failure.detail).filter(Boolean).join('\n\n'))
        } else {
          this.clearNotice()
        }
        return true
      } finally {
        if (generation === this.createGeneration) this.loadingBySource.__create__ = false
      }
    },
    async loadPage(sourceId: string, pageNumber: number): Promise<string[] | null> {
      if (!this.sources[sourceId] || this.loadingBySource[sourceId] || this.pacedParsingBySource[sourceId]) return null
      const generation = this.createGeneration
      const epoch = this.sourceEpochById[sourceId]
      const operationId = this.beginOperation(sourceId)
      this.loadingBySource[sourceId] = true
      this.errorsBySource[sourceId] = null
      this.errorDetailsBySource[sourceId] = null
      try {
        const result = await parseLoadPage({ source_id: sourceId, snapshot: cloneTree(this.sources[sourceId]), operation_id: operationId, page_number: pageNumber })
        if (!this.ownsSource(sourceId, generation, epoch)) return null
        this.upsertSource(result.tree)
        return result.item_ids
      } catch (error) {
        if (this.ownsSource(sourceId, generation, epoch)) this.recordSourceFailure(sourceId, error, `加载第 ${pageNumber} 页`)
        return null
      } finally {
        this.endOperation(sourceId, operationId)
        if (this.ownsSource(sourceId, generation, epoch)) this.loadingBySource[sourceId] = false
      }
    },
    async loadNextPage(sourceId: string): Promise<boolean> {
      const current = this.sources[sourceId]
      if (!current?.source.has_more || this.loadingBySource[sourceId]) return false
      const epoch = this.sourceEpochById[sourceId]
      const operationId = this.beginOperation(sourceId)
      this.loadingBySource[sourceId] = true
      this.errorsBySource[sourceId] = null
      this.errorDetailsBySource[sourceId] = null
      try {
        const tree = await parseLoadMore({ source_id: sourceId, snapshot: cloneTree(current), operation_id: operationId })
        if (this.sourceEpochById[sourceId] !== epoch) return false
        const pageNumber = (source: NormalizedSourceTree) => source.groups.find(group => group.page)?.page?.page_number
        if (tree.source.has_more && tree.source.loaded_count <= current.source.loaded_count && pageNumber(tree) === pageNumber(current)) {
          throw new Error('来源未返回下一页，已保留当前解析结果。')
        }
        this.upsertSource(tree)
        return true
      } catch (error) {
        if (this.sourceEpochById[sourceId] === epoch && !this.pacedParsingStopRequestedBySource[sourceId]) this.recordSourceFailure(sourceId, error)
        return false
      } finally {
        this.endOperation(sourceId, operationId)
        if (this.sourceEpochById[sourceId] === epoch) this.loadingBySource[sourceId] = false
      }
    },
    async prepareLoadedVideos(sourceId: string, mode: 'selected' | 'next' | 'all' | 'retry', batchSize = 50) {
      const source = this.sources[sourceId]
      if (!source || this.loadingBySource[sourceId] || this.pacedParsingBySource[sourceId]) return
      const selected = new Set(this.selectionBySource[sourceId] ?? [])
      const failures = this.preparationBySource[sourceId]?.failures ?? []
      const failedItems = new Set(failures.map(failure => failure.itemId))
      const ready = (part: NormalizedPart) => part.cid != null && part.streams.length > 0
      let targets = source.groups.flatMap(group => group.items).flatMap(item => {
        const retry = failures.find(failure => failure.itemId === item.id)
        if (mode === 'retry') return retry ? [retry] : []
        if (mode !== 'selected' && failedItems.has(item.id)) return []
        const parts = item.parts.filter(part => !ready(part) && (mode !== 'selected' || selected.has(part.id)))
        return parts.length ? [{ itemId: item.id, title: item.title, partIds: parts.map(part => part.id), message: '' }] : []
      })
      if (mode === 'next') targets = targets.slice(0, Math.max(1, Math.floor(batchSize)))
      if (!targets.length) return
      const generation = this.createGeneration
      const epoch = this.sourceEpochById[sourceId]
      const owns = () => this.ownsSource(sourceId, generation, epoch)
      const job = { total: targets.length, processed: 0, failures: [...failures] }
      this.preparationBySource[sourceId] = job
      const progress = this.preparationBySource[sourceId]!
      this.pacedParsingBySource[sourceId] = true
      this.pacedParsingStopRequestedBySource[sourceId] = false
      this.errorsBySource[sourceId] = null
      this.errorDetailsBySource[sourceId] = null
      const stopped = () => !owns() || this.pacedParsingStopRequestedBySource[sourceId]
      try {
        for (const target of targets) {
          if (stopped()) break
          try {
            const operationId = this.beginOperation(sourceId)
            let tree: NormalizedSourceTree
            try { tree = await selectionPrepare(sourceId, target.partIds, cloneTree(this.sources[sourceId]), operationId) }
            finally { this.endOperation(sourceId, operationId) }
            if (!owns()) break
            this.upsertSource(tree)
            progress.failures = progress.failures.filter(failure => failure.itemId !== target.itemId)
          } catch (error) {
            if (stopped()) break
            const message = errorMessage(error)
            progress.failures = progress.failures.filter(failure => failure.itemId !== target.itemId)
            progress.failures.push({ ...target, message, detail: commandDiagnostic(error, `操作: 解析视频\n来源: ${source.source.title}\n视频: ${target.title}`) })
            if (shouldStopBackgroundParsing(error)) {
              this.recordSourceFailure(sourceId, error)
              progress.processed++
              break
            }
          }
          progress.processed++
        }
      } finally {
        if (owns()) this.pacedParsingBySource[sourceId] = false
      }
    },
    async startBackgroundDownload(sourceId: string, retryFailed = false, confirmedOptions?: CreateTaskOptions): Promise<void> {
      if (this.backgroundJob?.status === 'running' || this.pacedParsingBySource[sourceId] || this.loadingBySource[sourceId]) return
      const source = this.sources[sourceId]
      if (!source) return
      const previous = this.backgroundJob?.sourceId === sourceId ? this.backgroundJob : null
      if (retryFailed && !previous?.failures.length) return
      let job: BackgroundParseJob = previous ?? { sourceId, title: source.source.title, status: 'running', processed: 0, created: 0, skipped: 0, error: null, failures: [], donePartIds: [], options: {} }
      job.status = 'running'
      job.error = null
      job.detail = undefined
      this.backgroundJob = job
      job = this.backgroundJob
      this.pacedParsingBySource[sourceId] = true
      this.pacedParsingStopRequestedBySource[sourceId] = false
      this.errorsBySource[sourceId] = null
      this.errorDetailsBySource[sourceId] = null
      const done = new Set(job.donePartIds)
      const retryIds = retryFailed ? job.failures.map((failure) => failure.partId) : null
      retryIds?.forEach((id) => done.delete(id))
      const stopped = () => Boolean(this.pacedParsingStopRequestedBySource[sourceId]) || !this.sources[sourceId]
      try {
        const settings = useSettingsStore()
        await settings.ensureLoaded()
        const defaults: SettingsSnapshot = JSON.parse(JSON.stringify(settings.saved))
        const options: CreateTaskOptions = confirmedOptions ?? (previous && Object.keys(job.options).length ? job.options : {
          downloadDir: defaults.download_dir ?? 'downloads', archiveMode: defaults.archive_mode,
          mediaMode: defaults.media_mode,
          downloadPresetId: defaults.selected_download_preset,
          outputExtension: defaults.media_mode === 'audio_only' ? defaults.audio_output_format : defaults.output_extension, namingTemplate: defaults.naming_template, namingVersion: defaults.naming_template_version,
          subtitleFormat: defaults.subtitle_format, danmakuFormat: defaults.danmaku_format,
          duplicateNamingStrategy: defaults.duplicate_naming_strategy, archiveAssets: defaults.archive_assets,
          retainRawStreams: defaults.retain_raw_streams, embedCover: defaults.embed_cover,
          embedSubtitles: defaults.embed_subtitles, missingQualityPolicy: defaults.missing_quality_policy,
          quality: defaults.media_preferences.video.length ? 'best' : defaults.quality,
          codec: defaults.media_preferences.video.length ? 'auto' : defaults.codec,
          audioQuality: defaults.media_preferences.audio.length ? 'best' : defaults.audio_quality,
          mediaPreferences: defaults.media_preferences, duplicatePolicy: 'skip', silent: true,
        })
        job.options = options
        while (!stopped()) {
          const tree = this.sources[sourceId]
          const parts = indexSourceParts(tree)
          const byPost = selectsWholePosts(tree.source.kind)
          const page = tree.groups.find(group => (group.page?.page_size ?? 0) > 0)?.page
          const batchSize = page?.page_size ?? 20
          const latestPosition = !retryIds && byPost && tree.source.has_more && page
            ? page.page_number * batchSize
            : Infinity
          const pending = (retryIds ?? [...parts.keys()]).filter(id =>
            !done.has(id) && (parts.get(id)?.item.nfo_metadata?.index ?? 0) <= latestPosition,
          )
          for (let start = 0; start < pending.length && !stopped();) {
            const first = parts.get(pending[start])
            let end = start + 1
            if (byPost || first?.part.cid != null) {
              const firstPosition = first?.item.nfo_metadata?.index
              const endOfPage = byPost && firstPosition ? Math.ceil(firstPosition / batchSize) * batchSize : Infinity
              while (end < pending.length && end - start < batchSize) {
                const next = parts.get(pending[end])
                if ((!byPost && next?.part.cid == null) || (next?.item.nfo_metadata?.index ?? 0) > endOfPage) break
                end++
              }
            }
            const ids = pending.slice(start, end)
            start = end
            const item = ids.length === 1 ? first : undefined
            let snapshot = tree
            if (byPost) {
              const selectedItems = new Set(ids.map(id => parts.get(id)?.item))
              snapshot = {
                source: tree.source,
                groups: tree.groups.flatMap(group => {
                  const items = group.items.filter(item => selectedItems.has(item))
                  return items.length ? [{ ...group, items }] : []
                }),
              }
            } else if (item) {
              snapshot = { source: tree.source, groups: [{ ...item.group, items: [item.item] }] }
            }
            const operationId = this.beginOperation(sourceId)
            let result: Awaited<ReturnType<typeof selectionEnqueueBatch>>
            try {
              result = await selectionEnqueueBatch(taskRequest(sourceId, ids, options, defaults), cloneTree(snapshot), operationId, (byPost ? first : item)?.itemIndex ?? 0)
            } finally {
              this.endOperation(sourceId, operationId)
            }
            useQueueStore().applyCreatedTasks(result.created)
            job.created += result.created.length
            job.skipped += result.skipped
            const processed = new Set(result.processed_part_ids)
            const previousFailures = new Map(job.failures.map(failure => [failure.partId, failure]))
            const failed = new Set(result.failures.map(failure => failure.part_id))
            job.failures = job.failures.filter(failure => !processed.has(failure.partId))
            for (const { part_id: partId, error } of result.failures) {
              const title = parts.get(partId)?.part.title ?? previousFailures.get(partId)?.title ?? partId
              job.failures.push({ partId, title, message: problemMessage(error), detail: commandDiagnostic(error, `来源: ${source.source.title}\n条目: ${title}`) })
            }
            for (const id of processed) {
              done.add(id)
              if (!previousFailures.has(id)) job.processed++
            }
            if (this.sources[sourceId] !== tree) {
              const expandedItems = new Set(result.processed_part_ids.filter(id => !failed.has(id)).map(id => parts.get(id)?.item.id))
              for (const item of this.sources[sourceId].groups.flatMap(group => group.items)) {
                if (expandedItems.has(item.id)) item.parts.filter(part => !parts.has(part.id)).forEach(part => done.add(part.id))
              }
            }
            if (result.blocked) throw result.blocked
          }
          if (retryFailed || stopped() || !this.sources[sourceId]?.source.has_more) break
          const progressed = await this.loadNextPage(sourceId)
          if (!progressed && !stopped()) throw Object.assign(new Error(this.errorsBySource[sourceId] ?? '没有加载到更多内容，已停止添加'), { detail: this.errorDetailsBySource[sourceId] })
        }
        const remaining = retryFailed && this.sources[sourceId] && (this.sources[sourceId].source.has_more || collectPartIds(this.sources[sourceId]).some((id) => !done.has(id)))
        if (this.backgroundJob) this.backgroundJob.status = stopped() || remaining ? 'stopped' : 'completed'
      } catch (error) {
        if (this.backgroundJob) {
          this.backgroundJob.status = stopped() ? 'stopped' : 'failed'
          this.backgroundJob.error = stopped() ? null : errorMessage(error)
          this.backgroundJob.detail = stopped() ? undefined : commandDiagnostic(error, `操作: 后台解析下载\n来源: ${source.source.title}`)
        }
        if (!stopped()) this.recordSourceFailure(sourceId, error)
      } finally {
        job.donePartIds = [...done]
        this.pacedParsingBySource[sourceId] = false
        this.pacedParsingStopRequestedBySource[sourceId] = false
        this.loadingBySource[sourceId] = false
      }
    },
    async parseAllPages(sourceId: string): Promise<PacedParsingResult> {
      if (!this.sources[sourceId]?.source.has_more) return 'completed'
      if (this.pacedParsingBySource[sourceId] || this.loadingBySource[sourceId]) return 'stopped'
      const epoch = this.sourceEpochById[sourceId]
      this.pacedParsingBySource[sourceId] = true
      this.pacedParsingStopRequestedBySource[sourceId] = false
      try {
        while (this.sources[sourceId]?.source.has_more) {
          const loaded = await this.loadNextPage(sourceId)
          if (this.sourceEpochById[sourceId] !== epoch || this.pacedParsingStopRequestedBySource[sourceId]) return 'stopped'
          if (!loaded) return 'failed'
        }
        return 'completed'
      } finally {
        if (this.sourceEpochById[sourceId] === epoch) {
          this.pacedParsingBySource[sourceId] = false
          this.pacedParsingStopRequestedBySource[sourceId] = false
        }
      }
    },
    stopPacedParsing(sourceId: string, reason: ParseCancelReason = 'stop_requested') {
      if (this.pacedParsingBySource[sourceId] || this.loadingBySource[sourceId]) this.pacedParsingStopRequestedBySource[sourceId] = true
      return Promise.allSettled((this.operationsBySource[sourceId] ?? []).map(id => parseCancel(id, reason).catch((error) => {
        this.setNotice(`停止请求未送达：${errorMessage(error)}`, 'warning', undefined, commandDiagnostic(error, `操作: 停止解析\n来源: ${sourceId}\n请求: ${id}`))
      })))
    },
    async clearWorkspace(reason: ParseCancelReason = 'workspace_cleared') {
      if (this.backgroundJob?.status === 'running') { this.setNotice('还有内容正在添加，请先停止添加。', 'warning'); return }
      this.createGeneration++
      this.stoppedBatchInput = null
      const stopping = Object.keys(this.operationsBySource).map(id => this.stopPacedParsing(id, reason))
      this.input = ''
      this.clearNotice()
      this.sources = {}
      this.sourceEpochById = {}
      this.operationsBySource = {}
      this.preparationBySource = {}
      this.backgroundJob = null
      this.sourceOrder = []
      this.activeSourceId = null
      this.batchMode = false
      this.batchEntries = []
      this.selectedBatchEntryIds = []
      this.selectionBySource = {}
      this.loadingBySource = {}
      this.pacedParsingBySource = {}
      this.pacedParsingStopRequestedBySource = {}
      this.errorsBySource = {}
      this.errorDetailsBySource = {}
      await Promise.allSettled(stopping)
    },
    async removeSource(sourceId: string) {
      if (this.backgroundJob?.status === 'running' && this.backgroundJob.sourceId === sourceId) return
      const epoch = this.sourceEpochById[sourceId]
      await this.stopPacedParsing(sourceId, 'workspace_cleared')
      if (this.sourceEpochById[sourceId] !== epoch) return
      if (this.backgroundJob?.sourceId === sourceId) this.backgroundJob = null
      delete this.sources[sourceId]
      delete this.sourceEpochById[sourceId]
      delete this.preparationBySource[sourceId]
      delete this.selectionBySource[sourceId]
      delete this.errorsBySource[sourceId]
      delete this.errorDetailsBySource[sourceId]
      delete this.pacedParsingBySource[sourceId]
      delete this.pacedParsingStopRequestedBySource[sourceId]
      this.sourceOrder = this.sourceOrder.filter(id => id !== sourceId)
      this.batchEntries = this.batchEntries.filter(entry => entry.sourceId !== sourceId)
      this.selectedBatchEntryIds = this.selectedBatchEntryIds.filter(id => this.batchEntries.some(entry => entry.id === id))
      if (this.activeSourceId === sourceId) this.activeSourceId = this.sourceOrder[0] ?? null
    },
    toggleNode(sourceId: string, nodeId: string) {
      const tree = this.sources[sourceId]
      if (!tree) {
        return
      }

      const partIds = partIdsForNode(tree, nodeId)
      this.togglePartIds(sourceId, partIds)
    },
    togglePartIds(sourceId: string, partIds: string[]) {
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
    async estimateDownloadSizeForSources(
      sourceIds: string[],
      options: CreateTaskOptions = {},
    ): Promise<SelectionSizeEstimate> {
      const settings = useSettingsStore()
      const targets = uniquePartIds(sourceIds).filter(
        (sourceId) => ((options.partIdsBySource?.[sourceId] ?? this.selectionBySource[sourceId])?.length ?? 0) > 0,
      )
      const snapshots = targets.map(sourceId => ({ sourceId, snapshot: cloneTree(this.sources[sourceId]), partIds: [...(options.partIdsBySource?.[sourceId] ?? this.selectionBySource[sourceId] ?? [])] }))
      await settings.ensureLoaded()
      const mediaPreferences = mediaPreferencesForTaskRequest(options, settings.saved)
      const aggregate: SelectionSizeEstimate = {
        estimated_bytes: 0,
        estimated_parts: 0,
        unknown_streams: 0,
      }
      for (const { sourceId, snapshot, partIds } of snapshots) {
        const estimate = await selectionEstimateSize({
          download_preset_id: options.downloadPresetId,
          source_id: sourceId,
          part_ids: partIds,
          missing_quality_policy: options.missingQualityPolicy ?? settings.saved.missing_quality_policy,
          media_mode: options.mediaMode ?? settings.saved.media_mode,
          quality: options.quality ?? settings.saved.quality,
          audio_quality: options.audioQuality ?? settings.saved.audio_quality,
          codec: options.codec ?? settings.saved.codec,
          media_preferences: mediaPreferences,
        }, snapshot)
        aggregate.estimated_bytes += estimate.estimated_bytes
        aggregate.estimated_parts += estimate.estimated_parts
        aggregate.unknown_streams += estimate.unknown_streams
      }
      return aggregate
    },
    async submitTasksForSources(sourceIds: string[], options: CreateTaskOptions = {}) {
      const settings = useSettingsStore()
      const origin = useUiStore().activeTab
      const sources = uniquePartIds(sourceIds).flatMap(sourceId => {
        const tree = this.sources[sourceId]
        const partIds = [...(options.partIdsBySource?.[sourceId] ?? this.selectionBySource[sourceId] ?? [])]
        if (!tree || !partIds.length) return []
        const selected = new Set(partIds)
        const snapshot = cloneTree(tree)
        for (const group of snapshot.groups) group.items = group.items.map(item => item.parts.some(part => selected.has(part.id)) ? item : { id: '', title: '', owner_name: null, cover_url: null, duration_seconds: null, parts: [] })
        return [{ sourceId, partIds, snapshot }]
      })
      if (!sources.length) return null
      await settings.ensureLoaded()
      const requests = sources.map(source => ({ ...source, request: taskRequest(source.sourceId, source.partIds, options, settings.saved) }))
      const failures: Array<{ message: string; detail: string }> = []
      let submitted = 0
      for (const { sourceId, snapshot, request } of requests) {
        try { await usePreparationsStore().submit(request, snapshot, origin); submitted++ }
        catch (error) { failures.push({ message: errorMessage(error), detail: commandDiagnostic(error, `操作: 提交下载准备\n提交入口: ${origin}\n来源: ${snapshot.source.title}\n链接: ${snapshot.source.input}\n来源ID: ${sourceId}\n所选分集数: ${request.part_ids.length}\n所选分集: ${request.part_ids.join(', ')}`) }) }
      }
      if (submitted) this.setNotice('正在添加下载任务', 'info', '查看传输')
      return { submitted, failures }
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

      const requests = targets.map((sourceId) => {
        const source = this.sources[sourceId]?.source
        const partIds = [...(options.partIdsBySource?.[sourceId] ?? this.selectionBySource[sourceId] ?? [])]
        const entry = useUiStore().activeTab
        const failureContext = `操作: 创建下载任务\n提交入口: ${entry}\n来源类型: ${source?.kind ?? 'unknown'}\n来源: ${source?.title ?? sourceId}\n链接: ${source?.input ?? sourceId}\n所选分集数: ${partIds.length}\n所选分集: ${partIds.join(', ')}\n预设: ${options.downloadPresetId ?? '自定义'}`
        return { sourceId, partIds, failureContext, snapshot: cloneTree(this.sources[sourceId]), generation: this.createGeneration, epoch: this.sourceEpochById[sourceId] }
      })
      targets.forEach((sourceId) => {
        this.loadingBySource[sourceId] = true
      })
      try {
        await settings.ensureLoaded()
        const aggregate: CreateTasksForSourcesResult = {
          created: [],
          duplicates: [],
          skipped_existing: 0,
          requires_confirmation: false,
          pendingSourceIds: [],
          failedSourceIds: [],
          failures: [],
        }

        for (const { sourceId, partIds, failureContext, snapshot } of requests) {
          if (this.pacedParsingBySource[sourceId] && this.pacedParsingStopRequestedBySource[sourceId]) return null
          try {
            const operationId = this.beginOperation(sourceId)
            let result: SelectionCreateTasksResult
            try { result = await selectionCreateTasks(taskRequest(sourceId, partIds, options, settings.saved), snapshot, operationId) }
            finally { this.endOperation(sourceId, operationId) }
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
            aggregate.failures.push({ sourceId, message, stopBackground: shouldStopBackgroundParsing(error), detail: commandDiagnostic(error, failureContext) })
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
        requests.forEach(({ sourceId, generation, epoch }) => {
          if (this.ownsSource(sourceId, generation, epoch)) this.loadingBySource[sourceId] = false
        })
      }
    },
    upsertSource(tree: NormalizedSourceTree) {
      const sourceId = tree.source.id
      const previous = this.sources[sourceId]
      if (!previous) this.sourceEpochById[sourceId] = crypto.randomUUID()
      const selection = this.selectionBySource[sourceId]
      if (previous && selection) {
        const currentIds = new Set(collectPartIds(tree))
        const items = new Map(tree.groups.flatMap(group => group.items.map(item => [item.id, item] as const)))
        const oldParts = new Map(previous.groups.flatMap(group => group.items.flatMap(item => item.parts.map(part => [part.id, { item, part }] as const))))
        this.selectionBySource[sourceId] = uniquePartIds(selection.flatMap(id => {
          if (currentIds.has(id)) return [id]
          const old = oldParts.get(id)
          const currentItem = old && items.get(old.item.id)
          if (currentItem && selectsWholePosts(tree.source.kind)) {
            const first = currentItem.parts[0]
            return first ? [first.id] : []
          }
          // Only whole-video placeholders expand; explicit P selections stay scoped.
          if (old && old.part.cid == null) return items.get(old.item.id)?.parts.map(part => part.id) ?? [id]
          return [id]
        }))
      }
      this.sources[sourceId] = tree
      if (!this.sourceOrder.includes(sourceId)) {
        this.sourceOrder.unshift(sourceId)
      }
      if (!this.activeSourceId) this.activeSourceId = sourceId
      this.errorsBySource[sourceId] = null
      this.errorDetailsBySource[sourceId] = null
      this.selectionBySource[sourceId] = this.selectionBySource[sourceId] ?? defaultSelection(tree)
    },
  },
})

const cloneTree = (tree: NormalizedSourceTree): NormalizedSourceTree => JSON.parse(JSON.stringify(tree)) as NormalizedSourceTree

const indexSourceParts = (tree: NormalizedSourceTree) => {
  const parts = new Map<string, { item: NormalizedItem; part: NormalizedPart; group: NormalizedSourceTree['groups'][number]; itemIndex: number }>()
  let itemIndex = 0
  for (const group of tree.groups) {
    for (const item of group.items) {
      for (const part of selectableParts(item, tree.source.kind)) {
        if (!parts.has(part.id)) parts.set(part.id, { item, part, group, itemIndex })
      }
      itemIndex++
    }
  }
  if (!selectsWholePosts(tree.source.kind)) return parts
  return new Map([...parts].sort(([, left], [, right]) =>
    (left.item.nfo_metadata?.index || left.itemIndex + 1) - (right.item.nfo_metadata?.index || right.itemIndex + 1),
  ))
}

const taskRequest = (
  sourceId: string,
  partIds: string[],
  options: CreateTaskOptions,
  saved: SettingsSnapshot,
): SelectionCreateTasksRequest => {
  const downloadDir = options.downloadDir !== undefined ? options.downloadDir?.trim() : saved.download_dir?.trim()
  const documentTreeOutput =
    options.documentTreeOutput !== undefined ? options.documentTreeOutput : saved.document_tree_output
  const mediaPreferences = mediaPreferencesForTaskRequest(options, saved)
  const mediaMode = options.mediaMode ?? saved.media_mode
  const audioOnly = mediaMode === 'audio_only'
  return {
    source_id: sourceId,
    part_ids: partIds,
    output_dir: downloadDir || undefined,
    document_tree_output: documentTreeOutput ?? undefined,
    naming_template_version: options.namingVersion ?? saved.naming_template_version ?? 2,
    naming_template: options.namingTemplate ?? saved.naming_template,
    duplicate_naming_strategy: options.duplicateNamingStrategy ?? saved.duplicate_naming_strategy,
    ...(options.downloadPresetId
      ? { download_preset_id: options.downloadPresetId }
      : {
          archive_mode: options.archiveMode ?? saved.archive_mode,
          output_extension: options.outputExtension ?? (audioOnly ? saved.audio_output_format : saved.output_extension),
          subtitle_format: options.subtitleFormat ?? saved.subtitle_format,
          danmaku_format: options.danmakuFormat ?? saved.danmaku_format,
          archive_assets: options.archiveAssets ?? saved.archive_assets,
          retain_raw_streams: options.retainRawStreams ?? saved.retain_raw_streams,
          embed_cover: !audioOnly && (options.embedCover ?? saved.embed_cover),
          embed_subtitles: !audioOnly && (options.embedSubtitles ?? saved.embed_subtitles),
          missing_quality_policy: options.missingQualityPolicy ?? saved.missing_quality_policy,
          media_mode: mediaMode,
          quality: options.quality ?? saved.quality,
          audio_quality: options.audioQuality ?? saved.audio_quality,
          codec: options.codec ?? saved.codec,
          media_preferences: mediaPreferences,
        }),
    duplicate_policy: options.duplicatePolicy ?? 'ask',
    scheduled_at: options.scheduledAt,
    speed_limit_bytes_per_second: options.speedLimitBytesPerSecond,
  }
}

export const collectPartIds = (tree: NormalizedSourceTree): string[] =>
  tree.groups.flatMap((group) => group.items.flatMap((item) => selectableParts(item, tree.source.kind).map((part) => part.id)))

const uniquePartIds = (partIds: string[]): string[] => Array.from(new Set(partIds))

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
      return group.items.flatMap(item => selectableParts(item, tree.source.kind).map(part => part.id))
    }

    for (const item of group.items) {
      if (item.id === nodeId) {
        return selectableParts(item, tree.source.kind).map(part => part.id)
      }

      const part = item.parts.find((candidate) => candidate.id === nodeId)
      if (part) {
        return [part.id]
      }
    }
  }

  return []
}


const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}

const errorCause = (error: unknown): string => {
  const detail = error && typeof error === 'object' && 'detail' in error ? error.detail : null
  return `${errorMessage(error)}${typeof detail === 'string' ? `\n${detail}` : ''}`
}

const shouldStopBackgroundParsing = (error: unknown): boolean => {
  if (error && typeof error === 'object' && 'problem' in error) {
    const problem = error.problem
    if (problem && typeof problem === 'object' && 'code' in problem && typeof problem.code === 'string') {
      return preparationBlockingCodes.includes(problem.code)
    }
  }
  const message = errorCause(error)
  // Content titles may themselves contain restriction words or HTTP-like numbers.
  if (message.startsWith('planning error:') && /缺少(?:视频|音频)流|没有符合.*偏好|没有可用的 SDR/.test(message)) return false
  const code = classifyFailure(message)?.code
  if (code && ['file_not_found', 'file_path_invalid', 'file_access_denied', 'file_read_only', 'disk_full', 'local_io_failed', 'database_error', 'data_format_error', 'bilibili_request_failed', 'network_request_failed', 'network_timeout', 'parse_cancelled', 'login_required', 'source_rate_limited', 'source_verification_required', 'source_access_restricted', 'remote_forbidden'].includes(code)) return true
  return /(?:^|[^\d])(?:-352|-412|-509|-799|401|403|412|429)(?:[^\d]|$)|限流|验证|风控|captcha|too many requests|解析已因|源站限制|登录状态可能已失效|storage error|io error|platform error|保存目录|磁盘|空间不足|后台创建下载任务失败/i.test(message)
}
