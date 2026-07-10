import { describe, expect, it } from 'vitest'

import type { NormalizedSourceTree } from '../../api/dto'
import { filterTreeNodes, flattenVisibleParts, numberVisibleParts, parseRangeExpression, toTreeNodes } from './parseResultTree'

const tree: NormalizedSourceTree = {
  source: { id: 'source-1', kind: 'video', input: 'BV1', title: '示例视频', loaded_count: 2, total_count: 2, has_more: false },
  groups: [{
    id: 'group-1',
    kind: 'video',
    title: '示例视频',
    page: null,
    items: [{
      id: 'item-1',
      title: '示例视频',
      owner_name: '测试 UP',
      cover_url: null,
      duration_seconds: 180,
      parts: [
        { id: 'part-1', title: '开场', aid: 1, bvid: 'BV1', cid: 11, duration_seconds: 60, streams: [], assets: [] },
        { id: 'part-2', title: '正片', aid: 1, bvid: 'BV1', cid: 12, duration_seconds: 120, streams: [], assets: [] },
      ],
    }],
  }],
}

describe('parse result tree', () => {
  it('numbers the visible video parts in display order', () => {
    const entries = flattenVisibleParts(numberVisibleParts(toTreeNodes(tree)))
    expect(entries.map((entry) => entry.label)).toEqual(['01  开场', '02  正片'])
  })

  it('filters by title and identifiers without changing the source tree', () => {
    const nodes = toTreeNodes(tree)
    expect(flattenVisibleParts(filterTreeNodes(nodes, '正片')).map((entry) => entry.id)).toEqual(['part-2'])
  })

  it('parses reversed and comma-separated ranges into zero-based indexes', () => {
    expect(parseRangeExpression('3-1, 5', 5)).toEqual([0, 1, 2, 4])
  })

  it('rejects a range outside the visible result count', () => {
    expect(() => parseRangeExpression('1-6', 5)).toThrow('范围超出当前结果数量')
  })
})
