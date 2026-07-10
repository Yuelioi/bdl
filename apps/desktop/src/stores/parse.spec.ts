import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useParseStore } from './parse'

const api = vi.hoisted(() => ({
  parseCloseSource: vi.fn(),
  parseCreateSource: vi.fn(),
  parseLoadAll: vi.fn(),
  parseLoadMore: vi.fn(),
  parseRefreshSource: vi.fn(),
  selectionCreateTasks: vi.fn(),
}))

vi.mock('../api/tauri', () => api)

describe('parse store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    Object.values(api).forEach((mock) => mock.mockReset())
  })

  it('requests every remaining page without the legacy 100-item limit', async () => {
    api.parseLoadAll.mockRejectedValue(new Error('request captured'))
    const parse = useParseStore()

    await parse.parseAll('favorite:42')

    expect(api.parseLoadAll).toHaveBeenCalledWith({ source_id: 'favorite:42' })
  })

  it('keeps initial parsing metadata-only so large multi-part videos stay fast', async () => {
    api.parseCreateSource.mockRejectedValue(new Error('request captured'))
    const parse = useParseStore()

    await parse.createSource('https://www.bilibili.com/video/av807178613/')

    expect(api.parseCreateSource).toHaveBeenCalledWith({
      input: 'https://www.bilibili.com/video/av807178613/',
      fetch_streams: false,
    })
  })
})
