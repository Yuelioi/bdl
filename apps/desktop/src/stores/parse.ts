import { defineStore } from 'pinia'

import type { NormalizedGroup, NormalizedItem, NormalizedPart, NormalizedSourceTree } from '../api/dto'
import {
  parseCloseSource,
  parseCreateSource,
  parseLoadAll,
  parseLoadMore,
  parseRefreshSource,
  selectionCreateTasks,
} from '../api/tauri'
import { useUiStore } from './ui'

interface ParseState {
  input: string
  sources: Record<string, NormalizedSourceTree>
  sourceOrder: string[]
  activeSourceId: string | null
  selectionBySource: Record<string, string[]>
  loadingBySource: Record<string, boolean>
  errorsBySource: Record<string, string | null>
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
  },
  actions: {
    setActiveSource(sourceId: string) {
      if (this.sources[sourceId]) {
        this.activeSourceId = sourceId
      }
    },
    async createSource(input?: string) {
      const ui = useUiStore()
      const trimmed = (input ?? this.input).trim()

      if (!trimmed) {
        ui.pushToast('请输入链接或 BV/AV', 'warning')
        return
      }

      this.loadingBySource.__create__ = true
      try {
        const tree = await parseCreateSource({ input: trimmed, fetch_streams: true })
        this.upsertSource(tree)
        this.input = ''
        ui.pushToast('解析完成', 'success')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loadingBySource.__create__ = false
      }
    },
    async loadMore(sourceId: string) {
      await this.runSourceAction(sourceId, async () => {
        await parseLoadMore()
      })
    },
    async parseAll(sourceId: string, _limit = 100) {
      await this.runSourceAction(sourceId, async () => {
        await parseLoadAll()
      })
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
        ui.pushToast('已关闭解析源', 'info')
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loadingBySource[sourceId] = false
      }
    },
    async refreshSource(sourceId: string) {
      await this.runSourceAction(sourceId, async () => {
        await parseRefreshSource()
      })
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
    async createTasksForSelection(sourceId: string) {
      const ui = useUiStore()
      const partIds = this.selectionBySource[sourceId] ?? []

      if (partIds.length === 0) {
        ui.pushToast('请选择要下载的分集', 'warning')
        return
      }

      this.loadingBySource[sourceId] = true
      try {
        const tasks = await selectionCreateTasks({
          source_id: sourceId,
          part_ids: partIds,
          archive_mode: 'fast',
          output_extension: 'mp4',
        })
        ui.pushToast(`已创建 ${tasks.length} 个任务`, 'success', { label: '查看传输', tab: 'transfer' })
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
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
    async runSourceAction(sourceId: string, action: () => Promise<void>) {
      const ui = useUiStore()
      this.loadingBySource[sourceId] = true
      try {
        await action()
      } catch (error) {
        this.errorsBySource[sourceId] = errorMessage(error)
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loadingBySource[sourceId] = false
      }
    },
  },
})

const collectPartIds = (tree: NormalizedSourceTree): string[] =>
  tree.groups.flatMap((group) => group.items.flatMap((item) => item.parts.map((part) => part.id)))

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
