import { describe, expect, it } from 'vitest'
import { fillMissingDefaults } from './settingsDefaults'

describe('missing setting defaults', () => {
  it('fills nested missing fields without overwriting explicit values or merging arrays', () => {
    const defaults = { nested: { enabled: true, retries: 3, added: 5 }, list: ['default'], path: 'path' as string | null }
    const saved = { nested: { enabled: false, retries: 0 }, list: [], path: null }
    const result = fillMissingDefaults(defaults, saved)
    expect(result).toEqual({ nested: { enabled: false, retries: 0, added: 5 }, list: [], path: null })
    result.nested.added = 9
    result.list.push('new')
    expect(defaults.nested.added).toBe(5)
    expect(saved.list).toEqual([])
  })
})
