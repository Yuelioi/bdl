import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { defineStore } from 'pinia'
import { markRaw } from 'vue'

const AUTO_CHECK_KEY = 'bdl.update.auto-check'

export const useUpdateStore = defineStore('update', {
  state: () => ({
    supported: true,
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
    hasUpdate: (state) => Boolean(state.update),
  },
  actions: {
    async initialize(supported = true) {
      this.supported = supported
      this.autoCheck = supported && localStorage.getItem(AUTO_CHECK_KEY) === 'true'
      try {
        this.currentVersion = await getVersion()
      } catch {
        // Browser previews use the package fallback.
      }
      if (this.autoCheck) void this.checkForUpdate(true)
    },
    setAutoCheck(value: boolean) {
      if (!this.supported) return
      this.autoCheck = value
      localStorage.setItem(AUTO_CHECK_KEY, String(value))
      if (value) void this.checkForUpdate(true)
    },
    async checkForUpdate(silent = false) {
      if (!this.supported) return
      if (this.checking || this.installing) return
      this.checking = true
      this.error = null
      try {
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
