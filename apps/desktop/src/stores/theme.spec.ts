import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { parseThemePreference, resolveEffectiveTheme, useThemeStore } from './theme'

describe('theme preference', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
    document.documentElement.classList.remove('dark')
    delete document.documentElement.dataset.theme
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })
  it('follows the operating system when preference is system', () => {
    expect(resolveEffectiveTheme('system', true)).toBe('dark')
    expect(resolveEffectiveTheme('system', false)).toBe('light')
  })

  it('keeps an explicit user choice regardless of the system', () => {
    expect(resolveEffectiveTheme('dark', false)).toBe('dark')
    expect(resolveEffectiveTheme('light', true)).toBe('light')
  })

  it('falls back to system for missing or damaged persisted values', () => {
    expect(parseThemePreference(null)).toBe('system')
    expect(parseThemePreference('sepia')).toBe('system')
  })

  it('persists and applies an explicit dark preference', () => {
    const theme = useThemeStore()

    theme.setPreference('dark')

    expect(localStorage.getItem('bdl.theme')).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('reacts to operating-system changes while following the system', () => {
    let listener: ((event: MediaQueryListEvent) => void) | undefined
    vi.stubGlobal('matchMedia', vi.fn(() => ({
      matches: true,
      addEventListener: (_type: string, callback: (event: MediaQueryListEvent) => void) => {
        listener = callback
      },
    })))
    const theme = useThemeStore()

    theme.initialize()
    listener?.({ matches: false } as MediaQueryListEvent)

    expect(theme.effectiveTheme).toBe('light')
    expect(document.documentElement.dataset.theme).toBe('light')
  })

  it('removes the operating-system listener when disposed', () => {
    const removeEventListener = vi.fn()
    const listener = vi.fn()
    vi.stubGlobal('matchMedia', vi.fn(() => ({
      matches: false,
      addEventListener: (_type: string, callback: (event: MediaQueryListEvent) => void) => {
        listener.mockImplementation(callback)
      },
      removeEventListener,
    })))
    const theme = useThemeStore()

    theme.initialize()
    theme.dispose()

    expect(removeEventListener).toHaveBeenCalledWith('change', expect.any(Function))
    expect(theme.initialized).toBe(false)
  })
})
