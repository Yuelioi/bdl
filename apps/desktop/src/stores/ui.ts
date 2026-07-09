import { defineStore } from 'pinia'

export type AppTab = 'parse' | 'transfer' | 'settings'

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
    drawerOpen: false,
    dialogOpen: false,
    toasts: [] as ToastMessage[],
    nextToastId: 1,
  }),
  actions: {
    setTab(tab: AppTab) {
      this.activeTab = tab
    },
    pushToast(message: string, tone: ToastMessage['tone'] = 'info', action?: ToastAction) {
      this.toasts.push({ id: this.nextToastId, message, tone, action })
      this.nextToastId += 1
    },
    removeToast(id: number) {
      this.toasts = this.toasts.filter((toast) => toast.id !== id)
    },
  },
})
