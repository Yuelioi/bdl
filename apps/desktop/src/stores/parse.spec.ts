import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useParseStore } from './parse'
import type { NormalizedSourceTree } from '../api/dto'

const api = vi.hoisted(() => ({
  parseCloseSource: vi.fn(),
  parseCreateSource: vi.fn(),
  parseLoadAll: vi.fn(),
  parseLoadMore: vi.fn(),
  parseRefreshSource: vi.fn(),
  selectionCreateTasks: vi.fn(),
}))

const sourceTree = (id: string, title: string): NormalizedSourceTree => ({
  source: {
    id,
    kind: 'video',
    input: id,
    title,
    loaded_count: 0,
    total_count: 0,
    has_more: false,
  },
  groups: [],
})

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

  it('rejects multiple inputs instead of creating hidden source workspaces', async () => {
    const parse = useParseStore()

    await parse.createSource('BV1\nBV2')

    expect(api.parseCreateSource).not.toHaveBeenCalled()
    expect(parse.notice?.message).toContain('一次只能解析一个来源')
  })

  it('replaces the previous workspace after a new source parses successfully', async () => {
    api.parseCreateSource.mockResolvedValue(sourceTree('source:new', '新来源'))
    api.parseCloseSource.mockResolvedValue(undefined)
    const parse = useParseStore()
    parse.upsertSource(sourceTree('source:old', '旧来源'))

    await parse.createSource('BV-new')

    expect(api.parseCloseSource).toHaveBeenCalledWith('source:old')
    expect(parse.sourceOrder).toEqual(['source:new'])
    expect(parse.activeSource?.source.title).toBe('新来源')
  })
})
