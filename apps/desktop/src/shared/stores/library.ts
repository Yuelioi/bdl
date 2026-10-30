import { commandDiagnostic } from '../utils/commandDiagnostic'
import { defineStore } from 'pinia'

import type { LibraryCategory, LibraryBrowsePage, LibraryBrowseItem } from '../api/dto'
import { accountLibraryList, parseCreateSource } from '../api/tauri'

const PAGE_SIZE = 20

interface LibraryState {
  activeKind: LibraryCategory
  pages: Record<LibraryCategory, LibraryBrowsePage | null>
  watchLaterItems: LibraryBrowseItem[] | null
  loading: boolean
  errorDetail: string | null
  error: string | null
  requestGeneration: number
}

export const useLibraryStore = defineStore('library', {
  state: (): LibraryState => ({
    activeKind: 'created_favorite',
    pages: {
      watch_later: null,
      created_favorite: null,
      collected_favorite: null,
    },
    loading: false,
    watchLaterItems: null,
    error: null,
    errorDetail: null,
    requestGeneration: 0,
  }),
  getters: {
    activePage(state): LibraryBrowsePage | null {
      return state.pages[state.activeKind]
    },
  },
  actions: {
    async selectKind(kind: LibraryCategory) {
      if (kind === this.activeKind) return
      this.activeKind = kind
      this.error = null
      this.errorDetail = null
      await this.load(kind, 1)
    },
    async load(kind?: LibraryCategory, page = 1) {
      const targetKind = kind ?? this.activeKind
      const generation = this.requestGeneration + 1
      this.requestGeneration = generation
      this.loading = true
      this.error = null
      this.errorDetail = null
      const startedAt = Date.now()
      try {
        let result: LibraryBrowsePage
        if (targetKind === 'watch_later') {
          let entries = this.watchLaterItems
          if (page === 1 || !entries) {
            const tree = await parseCreateSource({ input: 'https://www.bilibili.com/list/watchlater', fetch_streams: false, expand_video_collection: false }, crypto.randomUUID())
            entries = tree.groups.flatMap(group => group.items).map(item => {
                const part = item.parts[0]
                return {
                  kind: 'watch_later' as const, media_id: item.id, title: item.title,
                  description: null, cover_url: item.cover_url, owner_name: item.owner_name,
                  owner_mid: item.owner_mid == null ? null : String(item.owner_mid), media_count: 0,
                  source_url: part?.bvid ? `https://www.bilibili.com/video/${part.bvid}` : `https://www.bilibili.com/video/av${part?.aid}`,
                }
              })
            if (generation === this.requestGeneration) this.watchLaterItems = entries
          }
          const currentPage = Math.min(Math.max(1, page), Math.max(1, Math.ceil(entries.length / PAGE_SIZE)))
          result = { items: entries.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE), total: entries.length, page: currentPage, page_size: PAGE_SIZE, has_more: currentPage * PAGE_SIZE < entries.length }
        } else {
          result = await accountLibraryList({ kind: targetKind, page, page_size: PAGE_SIZE })
        }
        if (generation === this.requestGeneration) this.pages[targetKind] = result
      } catch (error) {
        if (generation === this.requestGeneration) {
          this.error = error instanceof Error ? error.message : String(error)
          this.errorDetail = commandDiagnostic(error, `操作: 加载内容库\n分类: ${targetKind}\n页码: ${page}`)
        }
      } finally {
        // Keep fast refreshes from flashing a skeleton for only one frame.
        const remaining = 200 - (Date.now() - startedAt)
        if (generation === this.requestGeneration && remaining > 0) {
          await new Promise(resolve => setTimeout(resolve, remaining))
        }
        if (generation === this.requestGeneration) this.loading = false
      }
    },
    clear() {
      this.requestGeneration += 1
      this.pages.created_favorite = null
      this.pages.collected_favorite = null
      this.pages.watch_later = null
      this.watchLaterItems = null
      this.loading = false
      this.error = null
      this.errorDetail = null
    },
  },
})
