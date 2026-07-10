import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useParseStore } from './parse'
import type { NormalizedSourceTree } from '../api/dto'
import { useSettingsStore } from './settings'

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
  groups: [{
    id: `group:${id}`,
    kind: 'video',
    title,
    page: null,
    items: [{
      id: `item:${id}`,
      title,
      owner_name: null,
      cover_url: null,
      duration_seconds: 60,
      parts: [{
        id: `part:${id}`,
        title,
        aid: null,
        bvid: id,
        cid: null,
        duration_seconds: 60,
        streams: [],
        assets: [],
      }],
    }],
  }],
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

  it('parses multiple inputs into selected link-level batch items', async () => {
    api.parseCreateSource.mockImplementation(({ input }: { input: string }) =>
      Promise.resolve(sourceTree(`source:${input}`, input)))
    const parse = useParseStore()

    await parse.createSource('BV1\nBV2')

    expect(api.parseCreateSource).toHaveBeenCalledTimes(2)
    expect(parse.sourceOrder).toEqual(['source:BV1', 'source:BV2'])
    expect(parse.selectedSourceIds).toEqual(['source:BV1', 'source:BV2'])
    expect(parse.selectedBatchEntryIds).toHaveLength(2)
  })

  it('keeps two links to different parts of one source as two video entries', async () => {
    const tree = sourceTree('source:multi', '多 P 视频')
    tree.groups[0]!.items[0]!.parts.push({
      id: 'part:source:multi:2',
      title: '第二 P',
      aid: null,
      bvid: 'source:multi',
      cid: null,
      duration_seconds: 60,
      streams: [],
      assets: [],
    })
    api.parseCreateSource.mockResolvedValue(tree)
    const parse = useParseStore()

    await parse.createSource('https://example.test/video?p=1\nhttps://example.test/video?p=2')

    expect(parse.batchEntries.map((entry) => entry.title)).toEqual(['多 P 视频', '第二 P'])
    expect(parse.selectionBySource['source:multi']).toEqual(['part:source:multi', 'part:source:multi:2'])
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

  it('aggregates duplicate confirmation across selected sources', async () => {
    const parse = useParseStore()
    const settings = useSettingsStore()
    settings.loaded = true
    parse.upsertSource(sourceTree('source:one', '视频一'))
    parse.upsertSource(sourceTree('source:two', '视频二'))
    api.selectionCreateTasks
      .mockResolvedValueOnce({
        created: [],
        duplicates: [{
          proposed_task_id: 'task:one',
          title: '视频一',
          existing_task_id: 'task:existing',
          existing_status: 'completed',
        }],
        requires_confirmation: true,
      })
      .mockResolvedValueOnce({ created: [], duplicates: [], requires_confirmation: false })

    const result = await parse.createTasksForSources(['source:one', 'source:two'])

    expect(api.selectionCreateTasks).toHaveBeenCalledTimes(2)
    expect(result?.pendingSourceIds).toEqual(['source:one'])
  })

  it('passes a per-download naming template to task creation', async () => {
    const parse = useParseStore()
    const settings = useSettingsStore()
    settings.loaded = true
    parse.upsertSource(sourceTree('source:direct', '直接保存'))
    api.selectionCreateTasks.mockResolvedValue({
      created: [],
      duplicates: [],
      requires_confirmation: false,
    })

    await parse.createTasksForSelection('source:direct', {
      namingTemplate: '{title} - P{part_index} - {part_title}.{ext}',
    })

    expect(api.selectionCreateTasks).toHaveBeenCalledWith(expect.objectContaining({
      naming_template: '{title} - P{part_index} - {part_title}.{ext}',
    }))
  })
})
