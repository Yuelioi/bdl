import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { defineStore } from 'pinia'

import type { AccountSummary } from '../api/dto'
import { accountGet, accountImportCookie, accountLogout, accountVerify } from '../api/tauri'
import { useUiStore } from './ui'

interface AccountState {
  profile: AccountSummary
  loading: boolean
  saving: boolean
  listening: boolean
  unlisten: UnlistenFn | null
}

const loggedOutAccount = (): AccountSummary => ({
  logged_in: false,
  name: null,
  avatar_url: null,
  mid: null,
  vip_label: null,
})

export const useAccountStore = defineStore('account', {
  state: (): AccountState => ({
    profile: loggedOutAccount(),
    loading: false,
    saving: false,
    listening: false,
    unlisten: null,
  }),
  getters: {
    displayName(state): string {
      if (state.profile.name) {
        return state.profile.name
      }

      if (state.profile.mid) {
        return `UID ${state.profile.mid}`
      }

      return '登录'
    },
    avatarLabel(state): string {
      return (state.profile.name ?? state.profile.mid ?? '未').slice(0, 1).toUpperCase()
    },
    statusLabel(state): string {
      return state.profile.logged_in ? '已登录' : '未登录'
    },
  },
  actions: {
    async load() {
      const ui = useUiStore()
      this.loading = true
      try {
        this.profile = await accountGet()
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loading = false
      }
    },
    async startEventListeners() {
      if (this.listening) {
        return
      }

      this.listening = true
      this.unlisten = await listen<AccountSummary>('account://updated', (event) => {
        this.profile = event.payload
      })
    },
    async importCookie(cookie: string): Promise<boolean> {
      const ui = useUiStore()
      const trimmed = cookie.trim()
      if (!trimmed) {
        ui.pushToast('请输入 Cookie', 'warning')
        return false
      }

      this.saving = true
      try {
        this.profile = await accountImportCookie({ cookie: trimmed })
        ui.pushToast('Cookie 已保存', 'success')
        return true
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
        return false
      } finally {
        this.saving = false
      }
    },
    async logout() {
      const ui = useUiStore()
      this.saving = true
      try {
        this.profile = await accountLogout()
        ui.pushToast('已退出登录', 'info')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.saving = false
      }
    },
    async verify() {
      const ui = useUiStore()
      this.loading = true
      try {
        this.profile = await accountVerify()
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.loading = false
      }
    },
  },
})

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
