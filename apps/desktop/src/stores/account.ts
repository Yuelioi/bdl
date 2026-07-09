import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { defineStore } from 'pinia'

import type { AccountSummary, QrLoginSession, QrLoginStatus } from '../api/dto'
import {
  accountGet,
  accountImportCookie,
  accountLoginQrPoll,
  accountLoginQrStart,
  accountLogout,
  accountVerify,
} from '../api/tauri'
import { useUiStore } from './ui'

interface AccountState {
  profile: AccountSummary
  qrSession: QrLoginSession | null
  qrStatus: QrLoginStatus | null
  qrMessage: string
  qrLoading: boolean
  qrPolling: boolean
  qrPollTimer: number | null
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
    qrSession: null,
    qrStatus: null,
    qrMessage: '',
    qrLoading: false,
    qrPolling: false,
    qrPollTimer: null,
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
    qrImageSrc(state): string | null {
      if (!state.qrSession) {
        return null
      }

      return `data:image/svg+xml;utf8,${encodeURIComponent(state.qrSession.qr_image_svg)}`
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
    async startQrLogin() {
      const ui = useUiStore()
      this.stopQrPolling()
      this.qrSession = null
      this.qrStatus = null
      this.qrMessage = ''
      this.qrLoading = true
      try {
        this.qrSession = await accountLoginQrStart()
        this.qrStatus = 'waiting'
        this.qrMessage = '等待扫码'
        this.startQrPolling()
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.qrLoading = false
      }
    },
    startQrPolling() {
      if (!this.qrSession) {
        return
      }

      this.stopQrPolling()
      void this.pollQrLogin()
      this.qrPollTimer = window.setInterval(() => {
        void this.pollQrLogin()
      }, 2500)
    },
    async pollQrLogin() {
      const ui = useUiStore()
      if (!this.qrSession || this.qrPolling) {
        return
      }

      this.qrPolling = true
      try {
        const response = await accountLoginQrPoll({
          qrcode_key: this.qrSession.qrcode_key,
        })
        this.qrStatus = response.status
        this.qrMessage = response.message || qrStatusText(response.status)

        if (response.account) {
          this.profile = response.account
          this.stopQrPolling()
          this.qrSession = null
          ui.pushToast('登录成功', 'success')
        } else if (response.status === 'expired') {
          this.stopQrPolling()
        }
      } catch (error) {
        this.stopQrPolling()
        ui.pushToast(errorMessage(error), 'danger')
      } finally {
        this.qrPolling = false
      }
    },
    stopQrPolling() {
      if (this.qrPollTimer !== null) {
        window.clearInterval(this.qrPollTimer)
        this.qrPollTimer = null
      }
    },
    resetQrLogin() {
      this.stopQrPolling()
      this.qrSession = null
      this.qrStatus = null
      this.qrMessage = ''
      this.qrPolling = false
    },
    async logout() {
      const ui = useUiStore()
      this.saving = true
      try {
        this.resetQrLogin()
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

const qrStatusText = (status: QrLoginStatus): string => {
  switch (status) {
    case 'waiting':
      return '等待扫码'
    case 'scanned':
      return '已扫码'
    case 'confirmed':
      return '登录成功'
    case 'expired':
      return '二维码已过期'
    case 'unknown':
      return '未知状态'
  }
}

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
