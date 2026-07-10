import { describe, expect, it } from 'vitest'

import {
  formatSpeedLimit,
  speedLimitMibError,
  toBytesPerSecond,
  toMibPerSecondInput,
} from './speedLimit'

describe('speed limit conversion', () => {
  it('treats an empty value as unlimited', () => {
    expect(toBytesPerSecond('')).toBeUndefined()
    expect(speedLimitMibError('')).toBeNull()
  })

  it('converts MiB per second to exact bytes per second', () => {
    expect(toBytesPerSecond('2.5')).toBe(2_621_440)
    expect(toMibPerSecondInput(2_621_440)).toBe('2.5')
  })

  it('round-trips persisted byte limits without losing small values', () => {
    for (const bytesPerSecond of [1, 64 * 1024, 2_621_441, 10 * 1024 * 1024 * 1024]) {
      expect(toBytesPerSecond(toMibPerSecondInput(bytesPerSecond))).toBe(bytesPerSecond)
    }
  })

  it('rejects zero and values above ten GiB per second', () => {
    expect(speedLimitMibError('0')).toContain('大于 0')
    expect(speedLimitMibError('10241')).toContain('10240')
    expect(speedLimitMibError('0.0000001')).toContain('1 B/s')
    expect(speedLimitMibError('0.0000005')).toContain('1 B/s')
  })

  it('formats task limits for compact queue UI', () => {
    expect(formatSpeedLimit(2_621_440)).toBe('2.5 MiB/s')
    expect(formatSpeedLimit(null)).toBe('不限速')
    expect(formatSpeedLimit(1)).toBe('1 B/s')
  })
})
