import { defineStore } from 'pinia'
import { FEEDBACK_AUTO_DISMISS_MS } from './feedback'

export type AppTab = 'parse' | 'library' | 'transfer' | 'settings' | 'about'

export interface ToastMessage {
  id: number
  tone: 'info' | 'success' | 'warning' | 'danger'
  message: string
  action?: ToastAction
}

export interface ToastAction {
  label: string
  tab: AppTab
}

export const useUiStore = defineStore('ui', {
  state: () => ({
    activeTab: 'parse' as AppTab,
    mobilePersonalPage: 'home' as 'home' | 'settings' | 'about',
    drawerOpen: false,
    dialogOpen: false,
    toasts: [] as ToastMessage[],
    nextToastId: 1,
    loginDialogOpen: false,
    environmentDialogOpen: false,
  }),
  actions: {
    setTab(tab: AppTab) {
      this.activeTab = tab
    },
    openLoginDialog() {
      this.loginDialogOpen = true
    },
    openEnvironmentDialog() {
      this.environmentDialogOpen = true
    },
    pushToast(message: string, tone: ToastMessage['tone'] = 'info', action?: ToastAction) {
      const id = this.nextToastId
      this.toasts.push({ id, message, tone, action })
      this.nextToastId += 1

      if (tone !== 'danger' && typeof window !== 'undefined') {
        window.setTimeout(() => {
          this.removeToast(id)
        }, FEEDBACK_AUTO_DISMISS_MS)
      }
    },
    removeToast(id: number) {
      this.toasts = this.toasts.filter((toast) => toast.id !== id)
    },
  },
})
