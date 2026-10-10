import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { defineStore } from 'pinia'
import { markRaw } from 'vue'

import { useSettingsStore } from './settings'
import { checkAndroidUpdate } from '../api/androidUpdate'
import { openExternalUrl } from '../api/tauri'

export const useUpdateStore = defineStore('update', {
  state: () => ({
    supported: true,
    android: false,
    androidDownloadUrl: null as string | null,
    currentVersion: '0.1.0',
    autoCheck: false,
    checking: false,
    checked: false,
    installing: false,
    progress: 0,
    availableVersion: null as string | null,
    notes: null as string | null,
    error: null as string | null,
    update: null as Update | null,
  }),
  getters: {
    hasUpdate: (state) => Boolean(state.update || state.androidDownloadUrl),
  },
  actions: {
    async initialize(supported = true, android = false) {
      this.android = android
      this.supported = supported || android
      const settings = useSettingsStore()
      await settings.ensureLoaded()
      this.autoCheck = this.supported && settings.saved.auto_check_updates
      try {
        this.currentVersion = await getVersion()
      } catch {
        // Browser previews use the package fallback.
      }
      if (this.autoCheck) void this.checkForUpdate(true)
    },
    async setAutoCheck(value: boolean) {
      if (!this.supported) return
      if (!await useSettingsStore().saveAppPreferences({ auto_check_updates: value })) return
      this.autoCheck = value
      if (value) void this.checkForUpdate(true)
    },
    async checkForUpdate(silent = false) {
      if (!this.supported) return
      if (this.checking || this.installing) return
      this.checking = true
      this.error = null
      try {
        if (this.android) {
          const availableUpdate = await checkAndroidUpdate(this.currentVersion)
          this.androidDownloadUrl = availableUpdate?.downloadUrl ?? null
          this.availableVersion = availableUpdate?.version ?? null
          this.notes = availableUpdate?.notes ?? null
          return
        }
        const availableUpdate = await check()
        this.update = availableUpdate ? markRaw(availableUpdate) : null
        this.availableVersion = this.update?.version ?? null
        this.notes = this.update?.body ?? null
      } catch (error) {
        if (!silent) this.error = error instanceof Error ? error.message : String(error)
      } finally {
        this.checked = true
        this.checking = false
      }
    },
    async install() {
      if (!this.supported) return
      if (this.android) {
        if (!this.androidDownloadUrl || this.installing) return
        this.installing = true
        this.error = null
        try {
          await openExternalUrl(this.androidDownloadUrl)
        } catch (error) {
          this.error = error instanceof Error ? error.message : String(error)
        } finally {
          this.installing = false
        }
        return
      }
      if (!this.update || this.installing) return
      this.installing = true
      this.error = null
      this.progress = 0
      let downloaded = 0
      let total = 0
      try {
        await this.update.downloadAndInstall((event) => {
          if (event.event === 'Started') total = event.data.contentLength ?? 0
          if (event.event === 'Progress') downloaded += event.data.chunkLength
          if (total > 0) this.progress = Math.min(100, Math.round((downloaded / total) * 100))
          if (event.event === 'Finished') this.progress = 100
        })
        await relaunch()
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error)
        this.installing = false
      }
    },
  },
})
