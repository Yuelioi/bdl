import { defineStore } from 'pinia'

import type { AccountLibraryFolderKind, AccountLibraryPage } from '../api/dto'
import { accountLibraryList } from '../api/tauri'

const PAGE_SIZE = 20

interface LibraryState {
  activeKind: AccountLibraryFolderKind
  pages: Record<AccountLibraryFolderKind, AccountLibraryPage | null>
  loading: boolean
  error: string | null
  requestGeneration: number
}

export const useLibraryStore = defineStore('library', {
  state: (): LibraryState => ({
    activeKind: 'created_favorite',
    pages: {
      created_favorite: null,
      collected_favorite: null,
    },
    loading: false,
    error: null,
    requestGeneration: 0,
  }),
  getters: {
    activePage(state): AccountLibraryPage | null {
      return state.pages[state.activeKind]
    },
  },
  actions: {
    async selectKind(kind: AccountLibraryFolderKind) {
      if (kind === this.activeKind) return
      this.activeKind = kind
      this.error = null
      if (this.pages[kind]) {
        this.requestGeneration += 1
        this.loading = false
        return
      }
      await this.load(kind, 1)
    },
    async load(kind?: AccountLibraryFolderKind, page = 1) {
      const targetKind = kind ?? this.activeKind
      const generation = this.requestGeneration + 1
      this.requestGeneration = generation
      this.loading = true
      this.error = null
      try {
        const result = await accountLibraryList({ kind: targetKind, page, page_size: PAGE_SIZE })
        if (generation === this.requestGeneration) this.pages[targetKind] = result
      } catch (error) {
        if (generation === this.requestGeneration) {
          this.error = error instanceof Error ? error.message : String(error)
        }
      } finally {
        if (generation === this.requestGeneration) this.loading = false
      }
    },
    clear() {
      this.requestGeneration += 1
      this.pages.created_favorite = null
      this.pages.collected_favorite = null
      this.loading = false
      this.error = null
    },
  },
})
