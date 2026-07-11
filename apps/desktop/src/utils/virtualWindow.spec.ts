import { describe, expect, it } from 'vitest'

import { calculateVirtualWindow } from './virtualWindow'

describe('virtual window', () => {
  it('keeps small lists intact', () => {
    expect(calculateVirtualWindow(20, 500, 200, 40)).toEqual({
      start: 0,
      end: 20,
      offset: 0,
      totalSize: 800,
      virtualized: false,
    })
  })

  it('renders the viewport with overscan for large lists', () => {
    expect(calculateVirtualWindow(10_000, 4_000, 400, 40)).toEqual({
      start: 94,
      end: 116,
      offset: 3_760,
      totalSize: 400_000,
      virtualized: true,
    })
  })

  it('clamps windows at the end of a list', () => {
    const result = calculateVirtualWindow(200, Number.MAX_SAFE_INTEGER, 400, 40)
    expect(result.end).toBe(200)
    expect(result.start).toBe(184)
  })
})
