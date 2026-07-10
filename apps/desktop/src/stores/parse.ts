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

interface ParseState {
  input: string
  sources: Record<string, NormalizedSourceTree>
  sourceOrder: string[]
  activeSourceId: string | null
  selectionBySource: Record<string, string[]>
  loadingBySource: Record<string, boolean>
  errorsBySource: Record<string, string | null>
  notice: InlineNotice | null
  noticeTimer: number | null
}

export interface CreateTaskOptions {
  downloadDir?: string | null
  archiveMode?: SettingsSnapshot['archive_mode']
  outputExtension?: SettingsSnapshot['output_extension']
  mediaMode?: DownloadMediaMode
  quality?: string
  audioQuality?: string
  codec?: SettingsSnapshot['codec']
  duplicatePolicy?: DuplicateTaskPolicy
  scheduledAt?: string
  speedLimitBytesPerSecond?: number
}

export const useParseStore = defineStore('parse', {
  state: (): ParseState => ({
    input: '',
    sources: {},
    sourceOrder: [],
    activeSourceId: null,
    selectionBySource: {},
    loadingBySource: {},
    errorsBySource: {},
    notice: null,
    noticeTimer: null,
  }),
  getters: {
    activeSource(state): NormalizedSourceTree | null {
      return state.activeSourceId ? (state.sources[state.activeSourceId] ?? null) : null
    },
    activeSelection(state): string[] {
      return state.activeSourceId ? (state.selectionBySource[state.activeSourceId] ?? []) : []
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

      if (inputs.length > 1) {
        this.setNotice('一次只能解析一个来源，请保留一个链接或编号', 'warning')
        return
      }

      this.loadingBySource.__create__ = true
      try {
        const tree = await parseCreateSource({ input: inputs[0], fetch_streams: false })
        const previousSourceIds = this.sourceOrder.filter((sourceId) => sourceId !== tree.source.id)
        for (const sourceId of previousSourceIds) {
          await this.removeSource(sourceId)
        }
        this.upsertSource(tree)
        this.input = ''
        this.setNotice('解析完成', 'success')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
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
        this.setNotice('已解析更多', 'success')
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
    ): Promise<SelectionCreateTasksResult | null> {
      const ui = useUiStore()
      const settings = useSettingsStore()
      const queue = useQueueStore()
      const partIds = this.selectionBySource[sourceId] ?? []

      if (partIds.length === 0) {
        this.setNotice('请选择要下载的分集', 'warning')
        return null
      }

      this.loadingBySource[sourceId] = true
      try {
        await settings.ensureLoaded()
        const downloadDir =
          options.downloadDir !== undefined ? options.downloadDir?.trim() : settings.saved.download_dir?.trim()
        const result = await selectionCreateTasks({
          source_id: sourceId,
          part_ids: partIds,
          output_dir: downloadDir || undefined,
          archive_mode: options.archiveMode ?? settings.saved.archive_mode,
          output_extension: options.outputExtension ?? settings.saved.output_extension,
          media_mode: options.mediaMode ?? 'audio_video',
          quality: options.quality ?? settings.saved.quality,
          audio_quality: options.audioQuality ?? settings.saved.audio_quality,
          codec: options.codec ?? settings.saved.codec,
          duplicate_policy: options.duplicatePolicy ?? 'ask',
          scheduled_at: options.scheduledAt,
          speed_limit_bytes_per_second: options.speedLimitBytesPerSecond,
        })
        this.errorsBySource[sourceId] = null
        if (result.requires_confirmation) {
          return result
        }
        if (result.created.length === 0) {
          this.setNotice('所选内容已在传输中', 'info', '查看传输')
          return result
        }

        queue.applyCreatedTasks(result.created)
        const duplicateSuffix = result.duplicates.length > 0 ? `，处理 ${result.duplicates.length} 个重复项` : ''
        this.setNotice(`已创建 ${result.created.length} 个任务${duplicateSuffix}`, 'success', '查看传输')
        return result
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
        return null
      } finally {
        this.loadingBySource[sourceId] = false
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
