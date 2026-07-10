import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { AccountLibraryPage } from '../api/dto'
import { useLibraryStore } from './library'

const { accountLibraryList } = vi.hoisted(() => ({
  accountLibraryList: vi.fn(),
}))

vi.mock('../api/tauri', () => ({ accountLibraryList }))

const fixturePage: AccountLibraryPage = {
  items: [],
  total: 33,
  page: 1,
  page_size: 20,
  has_more: true,
}

describe('library store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    accountLibraryList.mockReset()
  })

  it('loads and caches the selected account folder category', async () => {
    accountLibraryList.mockResolvedValue(fixturePage)
    const library = useLibraryStore()

    await library.selectKind('collected_favorite')

    expect(accountLibraryList).toHaveBeenCalledWith({
      kind: 'collected_favorite',
      page: 1,
      page_size: 20,
    })
    expect(library.activePage?.total).toBe(33)
    expect(library.error).toBeNull()
  })

  it('keeps an actionable error when account content cannot load', async () => {
    accountLibraryList.mockRejectedValue(new Error('登录状态已失效'))
    const library = useLibraryStore()

    await library.load()

    expect(library.loading).toBe(false)
    expect(library.error).toBe('登录状态已失效')
  })

  it('discards a previous account response after the account cache is cleared', async () => {
    let resolveOldRequest: (page: AccountLibraryPage) => void = () => undefined
    accountLibraryList.mockReturnValueOnce(new Promise<AccountLibraryPage>((resolve) => {
      resolveOldRequest = resolve
    }))
    const library = useLibraryStore()
    const oldRequest = library.load()

    library.clear()
    accountLibraryList.mockResolvedValueOnce({ ...fixturePage, total: 7 })
    await library.load()
    resolveOldRequest({ ...fixturePage, total: 99 })
    await oldRequest

    expect(library.activePage?.total).toBe(7)
    expect(library.loading).toBe(false)
  })

  it('discards a pending category error after returning to a cached category', async () => {
    let rejectCollected: (error: Error) => void = () => undefined
    const library = useLibraryStore()
    library.pages.created_favorite = fixturePage
    accountLibraryList.mockReturnValueOnce(new Promise<AccountLibraryPage>((_resolve, reject) => {
      rejectCollected = reject
    }))

    const collectedRequest = library.selectKind('collected_favorite')
    await Promise.resolve()
    await library.selectKind('created_favorite')
    rejectCollected(new Error('订阅接口暂时不可用'))
    await collectedRequest

    expect(library.activeKind).toBe('created_favorite')
    expect(library.activePage?.total).toBe(33)
    expect(library.error).toBeNull()
    expect(library.loading).toBe(false)
  })
})
