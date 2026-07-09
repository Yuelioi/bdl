import { defineStore } from 'pinia'

export type AppTab = 'parse' | 'transfer' | 'history' | 'settings'

export interface ToastMessage {
  id: number
  tone: 'info' | 'success' | 'warning' | 'danger'
  message: string
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
    pushToast(message: string, tone: ToastMessage['tone'] = 'info') {
      this.toasts.push({ id: this.nextToastId, message, tone })
      this.nextToastId += 1
    },
    removeToast(id: number) {
      this.toasts = this.toasts.filter((toast) => toast.id !== id)
    },
  },
})
