import { describe, expect, it } from 'vitest'

import type { ParseBatchEntry } from './parse'
import {
  allBatchEntriesSelected,
  buildBatchSelectionBySource,
  filterParseBatchEntries,
  selectedBatchSourceIds,
  toggleBatchEntrySelection,
} from './parseBatch'

const entries: ParseBatchEntry[] = [
  { id: 'one', sourceId: 'source:a', partId: 'part:1', title: 'Vue 教程', input: 'BV1' },
  { id: 'two', sourceId: 'source:a', partId: 'part:2', title: 'Rust 教程', input: 'BV2' },
  { id: 'three', sourceId: 'source:b', partId: 'part:3', title: '下载工具', input: 'BV3' },
]

describe('parse batch rules', () => {
  it('filters title and original input case-insensitively', () => {
    expect(filterParseBatchEntries(entries, 'RUST').map((entry) => entry.id)).toEqual(['two'])
    expect(filterParseBatchEntries(entries, 'bv3').map((entry) => entry.id)).toEqual(['three'])
  })

  it('requires every available entry for the all-selected state', () => {
    expect(allBatchEntriesSelected(entries, ['one', 'two', 'three'])).toBe(true)
    expect(allBatchEntriesSelected(entries, ['one', 'two'])).toBe(false)
    expect(allBatchEntriesSelected([], [])).toBe(false)
  })

  it('toggles only identifiers belonging to the current batch', () => {
    expect(toggleBatchEntrySelection(entries, ['one'], 'one')).toEqual([])
    expect(toggleBatchEntrySelection(entries, ['one'], 'two')).toEqual(['one', 'two'])
    expect(toggleBatchEntrySelection(entries, ['one'], 'missing')).toEqual(['one'])
  })

  it('deduplicates selected sources while preserving entry order', () => {
    expect(selectedBatchSourceIds(entries, ['two', 'three', 'one'])).toEqual(['source:a', 'source:b'])
  })

  it('rebuilds part selection for every known source', () => {
    expect(buildBatchSelectionBySource(['source:a', 'source:b', 'source:c'], entries, ['one', 'three'])).toEqual({
      'source:a': ['part:1'],
      'source:b': ['part:3'],
      'source:c': [],
    })
  })
})
