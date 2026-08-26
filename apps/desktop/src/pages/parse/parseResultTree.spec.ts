import { describe, expect, it } from 'vitest'

import type { NormalizedSourceTree } from '../../api/dto'
import { flattenResultRows, toTreeNodes } from './parseResultTree'

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
  it('flattens video parts into table rows without mixing sequence numbers into titles', () => {
    const rows = flattenResultRows(toTreeNodes(tree))
    expect(rows.map((row) => row.title)).toEqual(['开场', '正片'])
    expect(rows.map((row) => row.partIds)).toEqual([['part-1'], ['part-2']])
    expect(rows.map((row) => row.meta)).toEqual(['测试 UP', '测试 UP'])
  })

  it('keeps the UP column empty instead of substituting part durations when the owner is unavailable', () => {
    const ownerlessTree: NormalizedSourceTree = {
      ...tree,
      groups: tree.groups.map((group) => ({
        ...group,
        items: group.items.map((item) => ({ ...item, owner_name: null })),
      })),
    }

    const rows = flattenResultRows(toTreeNodes(ownerlessTree))
    expect(rows.map((row) => row.meta)).toEqual(['', ''])
  })
})
