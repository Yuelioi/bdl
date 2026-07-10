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
  parseLoadAll,
  parseLoadMore,
  parseRefreshSource,
  selectionCreateTasks,
} from '../api/tauri'
import { useQueueStore } from './queue'
import { useSettingsStore } from './settings'
import { useUiStore } from './ui'
import type { InlineNotice, NoticeTone } from './feedback'
import { NOTICE_CLEAR_DELAY } from './feedback'

const MAX_BATCH_SOURCES = 20
const PARSE_CONCURRENCY = 4

interface ParseState {
  input: string
  sources: Record<string, NormalizedSourceTree>
  sourceOrder: string[]
  activeSourceId: string | null
  batchMode: boolean
  batchEntries: ParseBatchEntry[]
  selectedBatchEntryIds: string[]
  selectionBySource: Record<string, string[]>
  loadingBySource: Record<string, boolean>
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
  downloadDir?: string | null
  archiveMode?: SettingsSnapshot['archive_mode']
  outputExtension?: SettingsSnapshot['output_extension']
  namingTemplate?: string
  mediaMode?: DownloadMediaMode
  quality?: string
  audioQuality?: string
  codec?: SettingsSnapshot['codec']
  duplicatePolicy?: DuplicateTaskPolicy
  scheduledAt?: string
  speedLimitBytesPerSecond?: number
}

export interface CreateTasksForSourcesResult extends SelectionCreateTasksResult {
  pendingSourceIds: string[]
  failedSourceIds: string[]
}

export const useParseStore = defineStore('parse', {
  state: (): ParseState => ({
    input: '',
    sources: {},
    sourceOrder: [],
    activeSourceId: null,
    batchMode: false,
    batchEntries: [],
    selectedBatchEntryIds: [],
    selectionBySource: {},
    loadingBySource: {},
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
        const selectedEntries = new Set(state.selectedBatchEntryIds)
        return Array.from(new Set(
          state.batchEntries
            .filter((entry) => selectedEntries.has(entry.id))
            .map((entry) => entry.sourceId),
        ))
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
    async createSource(input?: string) {
      const ui = useUiStore()
      const inputs = splitParseInputs(input ?? this.input)

      if (inputs.length === 0) {
        this.setNotice('请输入链接或 BV/AV', 'warning')
        return
      }

      if (inputs.length > MAX_BATCH_SOURCES) {
        this.setNotice(`一次最多解析 ${MAX_BATCH_SOURCES} 个链接`, 'warning')
        return
      }

      this.loadingBySource.__create__ = true
      try {
        const outcomes = await mapWithConcurrency(inputs, PARSE_CONCURRENCY, async (sourceInput) => {
          try {
            const tree = await parseCreateSource({ input: sourceInput, fetch_streams: false })
            return { input: sourceInput, tree, error: null }
          } catch (error) {
            return { input: sourceInput, tree: null, error: errorMessage(error) }
          }
        })
        const batch = inputs.length > 1
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
          ui.pushToast(failures[0]?.error ?? '解析失败', 'danger')
          return
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
              return part ? [{
                id: `batch:${index}:${outcome.tree.source.id}:${part.id}`,
                sourceId: outcome.tree.source.id,
                partId: part.id,
                title: outcome.tree.groups.flatMap((group) => group.items).flatMap((item) => item.parts).length > 1
                  ? part.title
                  : outcome.tree.source.title,
                input: outcome.input,
              }] : []
            })
          : []
        this.selectedBatchEntryIds = this.batchEntries.map((entry) => entry.id)
        this.syncBatchSelection()
        this.input = failures.map((failure) => failure.input).join('\n')
        this.setNotice(
          failures.length > 0
            ? `已解析 ${validOutcomes.length} 个链接，${failures.length} 个失败已保留`
            : batch ? `已解析 ${validOutcomes.length} 个链接` : '解析完成',
          failures.length > 0 ? 'warning' : 'success',
        )
      } finally {
        this.loadingBySource.__create__ = false
      }
    },
    async loadMore(sourceId: string) {
      const ui = useUiStore()
      this.loadingBySource[sourceId] = true
      try {
        const tree = await parseLoadMore({ source_id: sourceId })
        this.upsertSource(tree)
        this.setNotice(loadedCountMessage(tree), 'success')
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loadingBySource[sourceId] = false
      }
    },
    async loadChunk(sourceId: string, chunkSize: number) {
      const ui = useUiStore()
      const current = this.sources[sourceId]
      if (!current?.source.has_more) return

      const limit = current.source.loaded_count + Math.max(1, Math.floor(chunkSize))
      this.loadingBySource[sourceId] = true
      try {
        const tree = await parseLoadAll({ source_id: sourceId, limit })
        this.upsertSource(tree)
        this.setNotice(loadedCountMessage(tree), 'success')
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loadingBySource[sourceId] = false
      }
    },
    async parseAll(sourceId: string) {
      const ui = useUiStore()
      this.loadingBySource[sourceId] = true
      try {
        const tree = await parseLoadAll({ source_id: sourceId })
        this.upsertSource(tree)
        this.setNotice('已批量解析', 'success')
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loadingBySource[sourceId] = false
      }
    },
    async closeSource(sourceId: string) {
      const ui = useUiStore()
      this.loadingBySource[sourceId] = true
      try {
        await parseCloseSource(sourceId)
        delete this.sources[sourceId]
        delete this.selectionBySource[sourceId]
        delete this.errorsBySource[sourceId]
        this.sourceOrder = this.sourceOrder.filter((id) => id !== sourceId)
        this.batchEntries = this.batchEntries.filter((entry) => entry.sourceId !== sourceId)
        this.selectedBatchEntryIds = this.selectedBatchEntryIds.filter((entryId) =>
          this.batchEntries.some((entry) => entry.id === entryId))
        this.batchMode = this.batchMode && this.batchEntries.length > 0
        this.activeSourceId = this.sourceOrder[0] ?? null
        this.setNotice('已关闭解析源', 'info')
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loadingBySource[sourceId] = false
      }
    },
    async removeSource(sourceId: string) {
      try {
        await parseCloseSource(sourceId)
      } finally {
        delete this.sources[sourceId]
        delete this.selectionBySource[sourceId]
        delete this.errorsBySource[sourceId]
        this.sourceOrder = this.sourceOrder.filter((id) => id !== sourceId)
        this.batchEntries = this.batchEntries.filter((entry) => entry.sourceId !== sourceId)
        this.selectedBatchEntryIds = this.selectedBatchEntryIds.filter((entryId) =>
          this.batchEntries.some((entry) => entry.id === entryId))
        if (this.activeSourceId === sourceId) {
          this.activeSourceId = this.sourceOrder[0] ?? null
        }
      }
    },
    async refreshSource(sourceId: string) {
      const ui = useUiStore()
      this.loadingBySource[sourceId] = true
      try {
        this.upsertSource(await parseRefreshSource({ source_id: sourceId }))
        this.setNotice('已刷新来源', 'success')
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loadingBySource[sourceId] = false
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
      const selected = new Set(this.selectedBatchEntryIds)
      if (selected.has(entryId)) {
        selected.delete(entryId)
      } else if (this.batchEntries.some((entry) => entry.id === entryId)) {
        selected.add(entryId)
      }
      this.selectedBatchEntryIds = [...selected]
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
      for (const sourceId of this.sourceOrder) {
        this.selectionBySource[sourceId] = []
      }
      const selected = new Set(this.selectedBatchEntryIds)
      for (const entry of this.batchEntries) {
        if (selected.has(entry.id)) {
          this.selectionBySource[entry.sourceId] = uniquePartIds([
            ...(this.selectionBySource[entry.sourceId] ?? []),
            entry.partId,
          ])
        }
      }
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
      const ui = useUiStore()
      const settings = useSettingsStore()
      const queue = useQueueStore()
      const targets = uniquePartIds(sourceIds).filter((sourceId) => (this.selectionBySource[sourceId]?.length ?? 0) > 0)

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
          requires_confirmation: false,
          pendingSourceIds: [],
          failedSourceIds: [],
        }

        for (const sourceId of targets) {
          try {
            const result = await selectionCreateTasks({
              source_id: sourceId,
              part_ids: this.selectionBySource[sourceId] ?? [],
              output_dir: downloadDir || undefined,
              archive_mode: options.archiveMode ?? settings.saved.archive_mode,
              output_extension: options.outputExtension ?? settings.saved.output_extension,
              naming_template: options.namingTemplate,
              media_mode: options.mediaMode ?? 'audio_video',
              quality: options.quality ?? settings.saved.quality,
              audio_quality: options.audioQuality ?? settings.saved.audio_quality,
              codec: options.codec ?? settings.saved.codec,
              duplicate_policy: options.duplicatePolicy ?? 'ask',
              scheduled_at: options.scheduledAt,
              speed_limit_bytes_per_second: options.speedLimitBytesPerSecond,
            })
            this.errorsBySource[sourceId] = null
            aggregate.created.push(...result.created)
            aggregate.duplicates.push(...result.duplicates)
            if (result.requires_confirmation) {
              aggregate.requires_confirmation = true
              aggregate.pendingSourceIds.push(sourceId)
            }
            queue.applyCreatedTasks(result.created)
          } catch (error) {
            this.errorsBySource[sourceId] = errorMessage(error)
            aggregate.failedSourceIds.push(sourceId)
          }
        }

        if (aggregate.requires_confirmation) {
          return aggregate
        }
        const duplicateSuffix = aggregate.duplicates.length > 0 ? `，处理 ${aggregate.duplicates.length} 个重复项` : ''
        const failureSuffix = aggregate.failedSourceIds.length > 0 ? `，${aggregate.failedSourceIds.length} 个链接失败` : ''
        if (aggregate.created.length > 0) {
          this.setNotice(`已创建 ${aggregate.created.length} 个任务${duplicateSuffix}${failureSuffix}`, failureSuffix ? 'warning' : 'success', '查看传输')
        } else if (aggregate.failedSourceIds.length > 0) {
          ui.pushToast('所选链接创建任务失败', 'danger')
        } else {
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
      this.activeSourceId = sourceId
      this.errorsBySource[sourceId] = null
      this.selectionBySource[sourceId] = this.selectionBySource[sourceId] ?? defaultSelection(tree)
    },
  },
})

const collectPartIds = (tree: NormalizedSourceTree): string[] =>
  tree.groups.flatMap((group) => group.items.flatMap((item) => item.parts.map((part) => part.id)))

const uniquePartIds = (partIds: string[]): string[] => Array.from(new Set(partIds))

const loadedCountMessage = (tree: NormalizedSourceTree): string => {
  const total = tree.source.total_count
  return total === null ? `已加载 ${tree.source.loaded_count} 项` : `已加载 ${tree.source.loaded_count} / ${total} 项`
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

const defaultSelection = (tree: NormalizedSourceTree): string[] => (tree.source.kind === 'video' ? collectPartIds(tree) : [])

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
