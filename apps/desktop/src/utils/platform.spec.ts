import { describe, expect, it } from 'vitest'

import { isAndroidPlatform, isMacPlatform, isMobilePlatform } from './platform'

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

describe('mobile platform detection', () => {
  it('recognizes Android user agents', () => {
    const userAgent = 'Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36'
    expect(isAndroidPlatform(userAgent)).toBe(true)
    expect(isMobilePlatform('Linux armv8l', userAgent)).toBe(true)
  })

  it('keeps desktop Windows out of the mobile path', () => {
    expect(isMobilePlatform('Win32', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)')).toBe(false)
  })
})
