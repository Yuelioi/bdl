import { defineStore } from 'pinia'
import { FEEDBACK_AUTO_DISMISS_MS } from './feedback'
import type { SettingsDetailPage, SettingsSectionId } from '../features/settings/settingsSections'

export type AppTab = 'parse' | 'library' | 'transfer' | 'settings' | 'about'
export type MobileSettingsPage = 'overview' | SettingsDetailPage
type NavigationGuard = (leave: () => void) => void

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
    mobileSettingsPath: ['overview'] as MobileSettingsPage[],
    mobileSettingsReturnTab: null as 'parse' | 'library' | null,
    drawerOpen: false,
    dialogOpen: false,
    toasts: [] as ToastMessage[],
    nextToastId: 1,
    loginDialogOpen: false,
    environmentDialogOpen: false,
    settingsNavigationGuard: null as NavigationGuard | null,
    settingsSection: 'settings-download' as SettingsSectionId,
    settingsPresetKind: 'download' as 'download' | 'naming',
  }),
  actions: {
    openPresetSettings(kind: 'download' | 'naming') {
      this.settingsSection = 'settings-presets'
      this.settingsPresetKind = kind
      if (this.activeTab === 'settings' && this.mobilePersonalPage === 'settings') {
        if (this.mobileSettingsPath.at(-1) !== 'presets') this.mobileSettingsPath.push('presets')
      } else {
        this.openMobileSettings('presets')
        if (this.activeTab === 'parse' || this.activeTab === 'library') this.mobileSettingsReturnTab = this.activeTab
      }
      this.setTab('settings')
    },
    openMobileSettings(page: MobileSettingsPage) {
      this.mobileSettingsPath = [page]
      this.mobileSettingsReturnTab = null
      this.mobilePersonalPage = 'settings'
    },
    setTab(tab: AppTab) {
      if (tab === this.activeTab) return
      const leave = () => { this.activeTab = tab }
      const guard = this.activeTab === 'settings' ? this.settingsNavigationGuard : null
      if (guard) guard(leave)
      else leave()
    },
    setSettingsNavigationGuard(guard: NavigationGuard | null) {
      this.settingsNavigationGuard = guard
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
