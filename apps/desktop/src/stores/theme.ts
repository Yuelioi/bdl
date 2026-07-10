import { defineStore } from 'pinia'

export type ThemePreference = 'system' | 'light' | 'dark'
export type EffectiveTheme = Exclude<ThemePreference, 'system'>

const THEME_STORAGE_KEY = 'bdl.theme'

type SystemThemeSubscription = {
  query: MediaQueryList
  listener: (event: MediaQueryListEvent) => void
}

const systemThemeSubscriptions = new WeakMap<object, SystemThemeSubscription>()

export const parseThemePreference = (value: string | null): ThemePreference => {
  return value === 'light' || value === 'dark' || value === 'system' ? value : 'system'
}

export const resolveEffectiveTheme = (
  preference: ThemePreference,
  systemDark: boolean,
): EffectiveTheme => {
  return preference === 'system' ? (systemDark ? 'dark' : 'light') : preference
}

export const useThemeStore = defineStore('theme', {
  state: () => ({
    preference: 'system' as ThemePreference,
    systemDark: false,
    initialized: false,
  }),
  getters: {
    effectiveTheme(state): EffectiveTheme {
      return resolveEffectiveTheme(state.preference, state.systemDark)
    },
    preferenceLabel(state): string {
      const labels: Record<ThemePreference, string> = {
        system: '跟随系统',
        light: '浅色',
        dark: '深色',
      }
      return labels[state.preference]
    },
    icon(): string {
      return this.effectiveTheme === 'dark' ? 'moon' : 'sun'
    },
  },
  actions: {
    initialize() {
      if (this.initialized || typeof window === 'undefined') return

      const systemThemeQuery = typeof window.matchMedia === 'function'
        ? window.matchMedia('(prefers-color-scheme: dark)')
        : null
      this.systemDark = systemThemeQuery?.matches ?? false
      this.preference = readPersistedTheme()
      this.apply()

      if (systemThemeQuery) {
        const systemThemeListener = (event: MediaQueryListEvent) => {
          this.systemDark = event.matches
          if (this.preference === 'system') this.apply()
        }
        systemThemeQuery.addEventListener('change', systemThemeListener)
        systemThemeSubscriptions.set(this, {
          query: systemThemeQuery,
          listener: systemThemeListener,
        })
      }
      this.initialized = true
    },
    dispose() {
      const subscription = systemThemeSubscriptions.get(this)
      if (subscription) {
        subscription.query.removeEventListener('change', subscription.listener)
        systemThemeSubscriptions.delete(this)
      }
      this.initialized = false
    },
    setPreference(preference: ThemePreference) {
      this.preference = preference
      persistTheme(preference)
      this.apply()
    },
    apply() {
      if (typeof document === 'undefined') return
      const theme = this.effectiveTheme
      document.documentElement.dataset.theme = theme
      document.documentElement.classList.toggle('dark', theme === 'dark')
      document.documentElement.style.colorScheme = theme
      document
        .querySelector('meta[name="theme-color"]')
        ?.setAttribute('content', theme === 'dark' ? '#17201c' : '#f1f5f2')
    },
  },
})

const readPersistedTheme = (): ThemePreference => {
  try {
    return parseThemePreference(window.localStorage.getItem(THEME_STORAGE_KEY))
  } catch {
    return 'system'
  }
}

const persistTheme = (preference: ThemePreference) => {
  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, preference)
  } catch {
    // The active theme still applies for this session when storage is unavailable.
  }
}
