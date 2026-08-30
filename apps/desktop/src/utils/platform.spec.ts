import { describe, expect, it } from 'vitest'

import { isMacPlatform } from './platform'

describe('isMacPlatform', () => {
  it.each([
    ['MacIntel', 'Mozilla/5.0'],
    ['MacARM', 'Mozilla/5.0'],
    ['Linux armv8l', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 15_5)'],
  ])('recognizes macOS from %s', (platform, userAgent) => {
    expect(isMacPlatform(platform, userAgent)).toBe(true)
  })

  it('does not classify Windows as macOS', () => {
    expect(isMacPlatform('Win32', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)')).toBe(false)
  })
})
