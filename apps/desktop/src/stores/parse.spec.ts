import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { useParseStore } from './parse'
import type { NormalizedSourceTree } from '../api/dto'
import { useSettingsStore } from './settings'

const api = vi.hoisted(() => ({
  parseCloseSource: vi.fn(),
  parseCreateSource: vi.fn(),
  parseLoadAll: vi.fn(),
  parseLoadMore: vi.fn(),
  parseCancel: vi.fn(),
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
  groups: [
    {
      id: `group:${id}`,
      kind: 'video',
      title,
      page: null,
      items: [
        {
          id: `item:${id}`,
          title,
          owner_name: null,
          cover_url: null,
          duration_seconds: 60,
          parts: [
            {
              id: `part:${id}`,
              title,
              aid: null,
              bvid: id,
              cid: null,
              duration_seconds: 60,
              streams: [],
              assets: [],
            },
          ],
        },
      ],
    },
  ],
})

vi.mock('../api/tauri', () => api)

describe('parse store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    Object.values(api).forEach((mock) => mock.mockReset())
    api.parseCancel.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('parses all remaining items in paced chunks', async () => {
    vi.useFakeTimers()
    const firstPage = sourceTree('favorite:42', '大型收藏夹')
    firstPage.source.kind = 'favorite'
    firstPage.source.loaded_count = 40
    firstPage.source.total_count = 440
    firstPage.source.has_more = true
    const secondPage = structuredClone(firstPage)
    secondPage.source.loaded_count = 240
    const finalPage = structuredClone(firstPage)
    finalPage.source.loaded_count = 440
    finalPage.source.has_more = false
    api.parseLoadMore.mockResolvedValueOnce(secondPage).mockResolvedValueOnce(finalPage)
    const parse = useParseStore()
    parse.upsertSource(firstPage)

    const parsing = parse.parseAllPaced('favorite:42', 200, 3_000)
    await vi.advanceTimersByTimeAsync(3_000)

    await expect(parsing).resolves.toBe('completed')
    expect(api.parseLoadMore.mock.calls).toEqual([
      [{ source_id: 'favorite:42' }],
      [{ source_id: 'favorite:42' }],
    ])
    expect(parse.pacedParsingBySource['favorite:42']).toBe(false)
    vi.useRealTimers()
  })

  it('stops paced parsing before the next chunk', async () => {
    vi.useFakeTimers()
    const firstPage = sourceTree('favorite:42', '大型收藏夹')
    firstPage.source.kind = 'favorite'
    firstPage.source.loaded_count = 40
    firstPage.source.total_count = 440
    firstPage.source.has_more = true
    const secondPage = structuredClone(firstPage)
    secondPage.source.loaded_count = 240
    api.parseLoadMore.mockResolvedValueOnce(secondPage)
    const parse = useParseStore()
    parse.upsertSource(firstPage)

    const parsing = parse.parseAllPaced('favorite:42', 200, 3_000)
    await vi.advanceTimersByTimeAsync(0)
    parse.stopPacedParsing('favorite:42')
    await vi.advanceTimersByTimeAsync(100)

    await expect(parsing).resolves.toBe('stopped')
    expect(api.parseLoadMore).toHaveBeenCalledOnce()
    expect(parse.sources['favorite:42']?.source.loaded_count).toBe(240)
    vi.useRealTimers()
  })

  it('cancels an in-flight backend wait without losing loaded results or showing failure', async () => {
    const tree = sourceTree('favorite:wait', '等待中的收藏夹')
    tree.source.loaded_count = 100
    tree.source.total_count = 200
    tree.source.has_more = true
    let rejectWait!: (error: Error) => void
    api.parseLoadMore.mockImplementation(() => new Promise((_resolve, reject) => { rejectWait = reject }))
    api.parseCancel.mockImplementation(async () => { rejectWait(new Error('解析已停止')) })
    const parse = useParseStore()
    parse.upsertSource(tree)
    const parsing = parse.parseAllPaced(tree.source.id)
    parse.stopPacedParsing(tree.source.id)
    expect(api.parseCancel).toHaveBeenCalledWith(tree.source.id)
    await expect(parsing).resolves.toBe('stopped')
    expect(parse.sources[tree.source.id]?.source.loaded_count).toBe(100)
    expect(parse.errorsBySource[tree.source.id]).toBeNull()
  })

  it('loads only the next page when parsing more from a large source', async () => {
    const firstPage = sourceTree('uploader:42', '大型 UP 主空间')
    firstPage.source.kind = 'uploader'
    firstPage.source.loaded_count = 50
    firstPage.source.total_count = 20_000
    firstPage.source.has_more = true
    const secondPage = structuredClone(firstPage)
    secondPage.source.loaded_count = 100
    api.parseLoadMore.mockResolvedValue(secondPage)
    const parse = useParseStore()
    parse.upsertSource(firstPage)

    await parse.loadMore('uploader:42')

    expect(api.parseLoadMore).toHaveBeenCalledOnce()
    expect(api.parseLoadMore).toHaveBeenCalledWith({ source_id: 'uploader:42' })
    expect(parse.activeSource?.source.loaded_count).toBe(100)
    expect(parse.notice).toBeNull()
  })

  it('loads a configurable chunk relative to the currently loaded uploader items', async () => {
    const firstPage = sourceTree('uploader:42', '大型 UP 主空间')
    firstPage.source.kind = 'uploader'
    firstPage.source.loaded_count = 30
    firstPage.source.total_count = 20_000
    firstPage.source.has_more = true
    const chunk = structuredClone(firstPage)
    chunk.source.loaded_count = 240
    api.parseLoadMore.mockResolvedValue(chunk)
    const parse = useParseStore()
    parse.upsertSource(firstPage)

    await parse.loadChunk('uploader:42', 200)

    expect(api.parseLoadMore).toHaveBeenCalledWith({ source_id: 'uploader:42' })
    expect(parse.notice).toBeNull()
  })

  it('keeps initial parsing metadata-only so large multi-part videos stay fast', async () => {
    api.parseCreateSource.mockRejectedValue(new Error('request captured'))
    const parse = useParseStore()

    await parse.createSource('https://www.bilibili.com/video/av807178613/')

    expect(api.parseCreateSource).toHaveBeenCalledWith({
      input: 'https://www.bilibili.com/video/av807178613/',
      fetch_streams: false,
      expand_video_collection: true,
    })
  })

  it('rejects an empty submission without treating an existing result as a new parse', async () => {
    const parse = useParseStore()
    parse.upsertSource(sourceTree('source:existing', '已有结果'))

    const parsed = await parse.createSource('   ')

    expect(parsed).toBe(false)
    expect(api.parseCreateSource).not.toHaveBeenCalled()
    expect(parse.activeSource?.source.id).toBe('source:existing')
    expect(parse.notice?.message).toBe('请输入链接或 BV/AV')
  })

  it('uses the button transition as success feedback instead of a redundant completion notice', async () => {
    api.parseCreateSource.mockResolvedValue(sourceTree('source:new', '新来源'))
    const parse = useParseStore()

    const parsed = await parse.createSource('BV-new')

    expect(parsed).toBe(true)
    expect(parse.notice).toBeNull()
  })

  it('parses multiple inputs into selected link-level batch items', async () => {
    api.parseCreateSource.mockImplementation(({ input }: { input: string }) =>
      Promise.resolve(sourceTree(`source:${input}`, input)),
    )
    const parse = useParseStore()

    await parse.createSource('BV1\nBV2')

    expect(api.parseCreateSource).toHaveBeenCalledTimes(2)
    expect(api.parseCreateSource).toHaveBeenNthCalledWith(1, {
      input: 'BV1',
      fetch_streams: false,
      expand_video_collection: false,
    })
    expect(api.parseCreateSource).toHaveBeenNthCalledWith(2, {
      input: 'BV2',
      fetch_streams: false,
      expand_video_collection: false,
    })
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

  it('clears the entire parse workspace when returning to the source step', async () => {
    api.parseCloseSource.mockResolvedValue(undefined)
    const parse = useParseStore()
    parse.input = 'BV-old'
    parse.upsertSource(sourceTree('source:one', '来源一'))
    parse.upsertSource(sourceTree('source:two', '来源二'))
    parse.selectionBySource['source:one'] = ['part:source:one']
    parse.errorsBySource['source:one'] = '旧错误'

    await parse.clearWorkspace()

    expect(parse.input).toBe('')
    expect(parse.sourceOrder).toEqual([])
    expect(parse.sources).toEqual({})
    expect(parse.selectionBySource).toEqual({})
    expect(parse.errorsBySource).toEqual({})
    expect(parse.activeSource).toBeNull()
    expect(api.parseCloseSource).toHaveBeenCalledTimes(2)
    expect(api.parseCloseSource.mock.calls.flat()).toEqual(expect.arrayContaining(['source:one', 'source:two']))
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
        duplicates: [
          {
            proposed_task_id: 'task:one',
            title: '视频一',
            existing_task_id: 'task:existing',
            existing_status: 'completed',
          },
        ],
        skipped_existing: 0,
        requires_confirmation: true,
      })
      .mockResolvedValueOnce({ created: [], duplicates: [], skipped_existing: 0, requires_confirmation: false })

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
      skipped_existing: 0,
      requires_confirmation: false,
    })

    await parse.createTasksForSelection('source:direct', {
      namingTemplate: '{title} - P{part_index} - {part_title}.{ext}',
    })

    expect(api.selectionCreateTasks).toHaveBeenCalledWith(
      expect.objectContaining({
        naming_template: '{title} - P{part_index} - {part_title}.{ext}',
      }),
    )
  })

  it('reports existing final outputs skipped during task creation', async () => {
    const parse = useParseStore()
    const settings = useSettingsStore()
    settings.loaded = true
    parse.upsertSource(sourceTree('source:skip-existing', '已有下载'))
    parse.selectPartIds('source:skip-existing', ['part:source:skip-existing'])
    api.selectionCreateTasks.mockResolvedValue({
      created: [],
      duplicates: [],
      skipped_existing: 1,
      requires_confirmation: false,
    })

    const result = await parse.createTasksForSelection('source:skip-existing')

    expect(result?.skipped_existing).toBe(1)
    expect(parse.notice?.message).toBe('已跳过 1 个已有文件')
  })

  it('returns task-creation failures without polluting parse-page feedback', async () => {
    const parse = useParseStore()
    const settings = useSettingsStore()
    settings.loaded = true
    parse.upsertSource(sourceTree('source:failed-download', '下载失败来源'))
    api.selectionCreateTasks.mockRejectedValue(
      new Error('Bilibili 暂时拒绝了请求（HTTP 412），请稍后重试。'),
    )

    const result = await parse.createTasksForSources(['source:failed-download'])

    expect(result?.failures).toEqual([
      {
        sourceId: 'source:failed-download',
        message: 'Bilibili 暂时拒绝了请求（HTTP 412），请稍后重试。',
      },
    ])
    expect(parse.notice).toBeNull()
    expect(parse.errorsBySource['source:failed-download']).toBeNull()
  })
})


describe('background parse and download', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    Object.values(api).forEach((mock) => mock.mockReset())
    useSettingsStore().loaded = true
    api.selectionCreateTasks.mockResolvedValue({ created: [], duplicates: [], skipped_existing: 1, requires_confirmation: false })
  })

  it('downloads loaded content before paging, snapshots defaults, and leaves selection intact', async () => {
    const parse = useParseStore()
    const settings = useSettingsStore()
    const first = sourceTree('source:background', '后台来源')
    first.source.loaded_count = 1
    first.source.has_more = true
    const next = structuredClone(first)
    next.source.loaded_count = 2
    next.source.has_more = false
    next.groups[0].items.push({ ...next.groups[0].items[0], id: 'item:second', parts: [{ ...next.groups[0].items[0].parts[0], id: 'part:second' }] })
    parse.upsertSource(first)
    parse.selectionBySource[first.source.id] = []
    settings.saved.naming_template = '{title}.{ext}'
    const order: string[] = []
    api.selectionCreateTasks.mockImplementation(async () => {
      order.push('download')
      settings.saved.naming_template = '{bvid}.{ext}'
      return { created: [], duplicates: [], skipped_existing: 1, requires_confirmation: false }
    })
    api.parseLoadMore.mockImplementation(async () => { order.push('page'); return next })
    await parse.startBackgroundDownload(first.source.id)
    expect(order).toEqual(['download', 'page', 'download'])
    expect(api.selectionCreateTasks.mock.calls.map(([request]) => request.part_ids)).toEqual([['part:source:background'], ['part:second']])
    expect(api.selectionCreateTasks.mock.calls.every(([request]) => request.naming_template === '{title}.{ext}' && request.duplicate_policy === 'skip')).toBe(true)
    expect(parse.selectionBySource[first.source.id]).toEqual([])
    expect(parse.backgroundJob).toMatchObject({ status: 'completed', processed: 2 })
  })

  it('stops after an in-flight task group without paging or cancelling downloads', async () => {
    const parse = useParseStore()
    const first = sourceTree('source:stop', '停止')
    first.source.has_more = true
    parse.upsertSource(first)
    api.selectionCreateTasks.mockImplementation(async () => {
      parse.stopPacedParsing(first.source.id)
      return { created: [], duplicates: [], skipped_existing: 0, requires_confirmation: false }
    })
    await parse.startBackgroundDownload(first.source.id)
    expect(parse.backgroundJob?.status).toBe('stopped')
    expect(api.parseLoadMore).not.toHaveBeenCalled()
  })

  it('retains completed pages and stops on a pagination error', async () => {
    const parse = useParseStore()
    const first = sourceTree('source:failure', '失败')
    first.source.loaded_count = 20
    first.source.has_more = true
    const next = structuredClone(first)
    next.source.loaded_count = 40
    parse.upsertSource(first)
    api.parseLoadMore.mockResolvedValueOnce(next).mockRejectedValueOnce(new Error('HTTP 412'))
    expect(await parse.loadChunk(first.source.id, 100)).toBe(false)
    expect(parse.sources[first.source.id].source.loaded_count).toBe(40)
    expect(parse.errorsBySource[first.source.id]).toContain('412')
    expect(api.parseLoadMore).toHaveBeenCalledTimes(2)
  })
})
