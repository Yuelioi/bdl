import { describe, expect, it } from 'vitest'

import { displayPartDuration, formatDuration } from './duration'

describe('duration display', () => {
  it('uses the part duration instead of the multi-part item total', () => {
    expect(displayPartDuration(92, 91_084, 190)).toBe(92)
  })

  it('does not repeat an item total when a multi-part duration is unavailable', () => {
    expect(displayPartDuration(null, 91_084, 190)).toBeNull()
  })

  it('formats durations longer than one hour with an hour segment', () => {
    expect(formatDuration(91_084)).toBe('25:18:04')
  })
})
