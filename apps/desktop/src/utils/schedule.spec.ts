import { describe, expect, it } from 'vitest'

import { scheduledLocalError, toDateTimeLocalValue } from './schedule'

describe('schedule utilities', () => {
  it('rejects an invalid or past local time', () => {
    const now = new Date('2026-07-10T12:00:00Z').getTime()

    expect(scheduledLocalError('invalid', now)).toBe('请选择有效的开始时间')
    expect(scheduledLocalError('2026-07-10T11:00:00Z', now)).toBe('开始时间必须晚于当前时间')
  })

  it('allows optional empty values and requires them when requested', () => {
    expect(scheduledLocalError('', 0)).toBe('')
    expect(scheduledLocalError('', 0, true)).toBe('请选择开始时间')
  })

  it('formats local datetime values for positive and negative UTC offsets', () => {
    const instant = new Date('2026-07-10T12:00:00Z')

    expect(toDateTimeLocalValue(instant, -480)).toBe('2026-07-10T20:00')
    expect(toDateTimeLocalValue(instant, 240)).toBe('2026-07-10T08:00')
  })
})
