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
  qrError: string
  qrLoading: boolean
  qrPolling: boolean
  qrPollTimer: number | null
  qrRequestId: number
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
    qrError: '',
    qrLoading: false,
    qrPolling: false,
    qrPollTimer: null,
    qrRequestId: 0,
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
        this.qrMessage = '请输入 Cookie'
        return false
      }

      this.saving = true
      try {
        this.profile = await accountImportCookie({ cookie: trimmed })
        await this.verify()
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
      const requestId = this.qrRequestId + 1
      this.qrRequestId = requestId
      this.stopQrPolling()
      this.qrSession = null
      this.qrStatus = null
      this.qrMessage = ''
      this.qrError = ''
      this.qrLoading = true
      try {
        const session = await accountLoginQrStart()
        if (requestId !== this.qrRequestId) {
          return
        }

        this.qrSession = session
        this.qrStatus = 'waiting'
        this.qrMessage = '等待扫码'
        this.startQrPolling()
      } catch (error) {
        if (requestId !== this.qrRequestId) {
          return
        }

        this.qrError = errorMessage(error)
        this.qrMessage = '二维码获取失败，请检查网络后重试'
        ui.pushToast(this.qrError, 'danger')
      } finally {
        if (requestId === this.qrRequestId) {
          this.qrLoading = false
        }
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
          await this.verify()
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
      this.qrRequestId += 1
      this.stopQrPolling()
      this.qrSession = null
      this.qrStatus = null
      this.qrMessage = ''
      this.qrError = ''
      this.qrLoading = false
      this.qrPolling = false
    },
    async logout() {
      const ui = useUiStore()
      this.saving = true
      try {
        this.resetQrLogin()
        this.profile = await accountLogout()
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
