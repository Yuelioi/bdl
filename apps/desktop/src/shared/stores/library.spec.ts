import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { AccountLibraryPage } from '../api/dto'
import { useLibraryStore } from './library'

const { accountLibraryList, parseCreateSource } = vi.hoisted(() => ({
  accountLibraryList: vi.fn(),
  parseCreateSource: vi.fn(),
}))

vi.mock('../api/tauri', () => ({ accountLibraryList, parseCreateSource }))

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
    parseCreateSource.mockReset()
  })

  it.each(['created_favorite', 'collected_favorite', 'watch_later'] as const)('retains %s failure details and clears them on logout', async (kind) => {
    const error = Object.assign(new Error('加载失败'), { code: 'bilibili_api_error', detail: 'HTTP 412; SESSDATA=private-cookie' })
    accountLibraryList.mockRejectedValue(error)
    parseCreateSource.mockRejectedValue(error)
    const library = useLibraryStore()
    await library.load(kind)
    expect(library.errorDetail).toContain('HTTP 412')
    expect(library.errorDetail).not.toContain('private-cookie')
    library.clear()
    expect(library.errorDetail).toBeNull()
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

  it('browses a large watch-later list without parsing each video, and clears account data', async () => {
    parseCreateSource.mockResolvedValue({ source: { id: 'watch:1' }, groups: [{ items: Array.from({ length: 1000 }, (_, i) => ({ id: `item:${i}`, title: `视频 ${i}`, owner_name: 'UP', owner_mid: 1, cover_url: null, parts: [{ aid: i + 1, bvid: null }] })) }] })
    const library = useLibraryStore()
    await library.selectKind('watch_later')
    expect(library.activePage?.items).toHaveLength(20)
    expect(library.activePage?.total).toBe(1000)
    expect(library.activePage?.items[0]?.source_url).toBe('https://www.bilibili.com/video/av1')
    await library.load('watch_later', 50)
    expect(library.activePage?.items[0]?.title).toBe('视频 980')
    expect(parseCreateSource).toHaveBeenCalledTimes(1)
    expect(library.watchLaterItems).toHaveLength(1000)
    expect(accountLibraryList).not.toHaveBeenCalled()
    library.clear()
    expect(library.pages.watch_later).toBeNull()
    expect(library.watchLaterItems).toBeNull()
  })

  it('ignores stateless watch-later responses arriving after account clear', async () => {
    let resolve: (value: unknown) => void = () => undefined
    parseCreateSource.mockReturnValue(new Promise(done => { resolve = done }))
    const library = useLibraryStore()
    const loading = library.selectKind('watch_later')
    library.clear()
    resolve({ source: { id: 'watch:old' }, groups: [] })
    await loading
    expect(library.pages.watch_later).toBeNull()
    expect(library.watchLaterItems).toBeNull()
    expect(library.pages.watch_later).toBeNull()
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
    accountLibraryList.mockResolvedValueOnce(fixturePage)
    await library.selectKind('created_favorite')
    rejectCollected(new Error('订阅接口暂时不可用'))
    await collectedRequest

    expect(library.activeKind).toBe('created_favorite')
    expect(library.activePage?.total).toBe(33)
    expect(library.error).toBeNull()
    expect(library.loading).toBe(false)
    expect(accountLibraryList).toHaveBeenCalledTimes(2)
  })
})
