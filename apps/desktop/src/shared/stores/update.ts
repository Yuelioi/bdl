import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { defineStore } from 'pinia'
import { markRaw } from 'vue'

import { useSettingsStore } from './settings'
import { checkAndroidUpdate } from '../api/androidUpdate'
import { distributionUpdate, checkMirror, openUpdateDownload, mirrorTemplate, mirrorUrl, newer, type DistributionUpdate } from '../api/updateSources'

export const useUpdateStore = defineStore('update', {
  state: () => ({
    supported: true,
    source: 'github' as 'github' | 'mirror' | 'quark',
    customMirror: '',
    distribution: null as DistributionUpdate | null,
    manualDownloadUrl: null as string | null,
    downloadMirror: undefined as { template: string; officialUrl: string } | undefined,
    warning: null as string | null,
    android: false,
    androidDownloadUrl: null as string | null,
    currentVersion: '0.1.0',
    autoCheck: false,
    checking: false,
    openingQuark: false,
    quarkError: null as string | null,
    checked: false,
    installing: false,
    progress: 0,
    availableVersion: null as string | null,
    notes: null as string | null,
    error: null as string | null,
    update: null as Update | null,
  }),
  getters: {
    hasUpdate: (state) => Boolean(state.update || state.androidDownloadUrl || state.manualDownloadUrl),
  },
  actions: {
    async openQuark() {
      if (this.openingQuark) return
      this.openingQuark = true
      this.quarkError = null
      try {
        const site = await distributionUpdate()
        const url = site.sources.find(s => s.kind === 'cloud_drive' && s.delivery.url?.startsWith('https://pan.quark.cn/'))?.delivery.url
        if (!url) throw new Error('当前平台尚未提供夸克下载，请选择其他来源。')
        await openUpdateDownload(url)
      } catch (error) {
        this.quarkError = error instanceof Error ? error.message : String(error)
      } finally {
        this.openingQuark = false
      }
    },
    setSource(source: 'github' | 'mirror' | 'quark', customMirror?: string) {
      if (this.checking || this.installing) return
      this.source = source
      this.customMirror = (customMirror ?? this.customMirror).trim()
      this.update?.close?.().catch(() => {})
      this.update = null
      this.androidDownloadUrl = null
      this.manualDownloadUrl = null
      this.downloadMirror = undefined
      this.availableVersion = null
      this.checked = false
      this.error = null
      this.warning = null
      try { localStorage.setItem('bdl.update-source', JSON.stringify({ source, customMirror: this.customMirror })) } catch { /* Storage may be unavailable. */ }
    },
    async initialize(supported = true, android = false) {
      try {
        const saved = JSON.parse(localStorage.getItem('bdl.update-source') ?? '{}')
        if (['github', 'mirror', 'quark'].includes(saved.source)) this.source = saved.source
        if (typeof saved.customMirror === 'string') this.customMirror = saved.customMirror
      } catch { /* Ignore invalid preferences. */ }
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
      this.warning = null
      this.checked = false
      this.update?.close?.().catch(() => {})
      this.update = null
      this.androidDownloadUrl = null
      this.manualDownloadUrl = null
      this.downloadMirror = undefined
      this.availableVersion = null
      this.distribution = null
      try {
        let siteError: unknown
        try { this.distribution = await distributionUpdate() } catch (error) { siteError = error }
        const site = this.distribution
        const github = site?.sources.find(s => s.kind === 'github')?.delivery.url
        const quark = site?.sources.find(s => s.kind === 'cloud_drive' && s.delivery.url?.startsWith('https://pan.quark.cn/'))
        let template = this.customMirror
        if (!template && github) {
          template = site?.sources.map(s => mirrorTemplate(s, github)).find(Boolean) ?? ''
        }
        if (this.source === 'quark') {
          if (!site) throw siteError ?? new Error('软件站暂时无法连接。')
          if (!quark) throw new Error('当前平台尚未提供夸克下载，请选择其他来源。')
          if (newer(site.release.version, this.currentVersion)) {
            this.manualDownloadUrl = quark.delivery.url!
            this.availableVersion = site.release.version
            this.notes = site.releaseNotes
          }
          this.checked = true
          return
        }
        if (this.source === 'mirror' && !template) throw new Error('软件站暂无可用加速源，请填写自定义加速地址。')
        if (this.source === 'mirror') mirrorUrl(template, 'https://github.com/Yuelioi/bdl/releases/latest/download/latest.json')
        if (this.source === 'mirror' && github) this.downloadMirror = { template, officialUrl: github }
        if (this.android) {
          if (this.source === 'mirror' && site && !github) throw new Error('当前平台尚未提供可加速的安装包，请选择其他来源。')
          if (site && github && newer(site.release.version, this.currentVersion)) {
            this.androidDownloadUrl = this.source === 'mirror' ? mirrorUrl(template, github) : github
            this.availableVersion = site.release.version
            this.notes = site.releaseNotes
          } else if (this.source === 'github') {
            const availableUpdate = await checkAndroidUpdate(this.currentVersion)
            this.androidDownloadUrl = availableUpdate?.downloadUrl ?? null
            this.availableVersion = availableUpdate?.version ?? null
            this.notes = availableUpdate?.notes ?? null
          } else if (!site) {
            throw siteError ?? new Error('软件站暂时无法连接。')
          }
          this.checked = true
          return
        }
        const proxy = useSettingsStore().saved.proxy_url ?? undefined
        try {
          const availableUpdate = this.source === 'mirror'
            ? await checkMirror(template, proxy)
            : await check({ timeout: 20_000, ...(proxy ? { proxy } : {}) })
          this.update = availableUpdate ? markRaw(availableUpdate) : null
          this.availableVersion = this.update?.version ?? null
          this.notes = this.update?.body ?? null
          if (!this.update && site && github && newer(site.release.version, this.currentVersion)) {
            this.manualDownloadUrl = this.source === 'mirror' ? mirrorUrl(template, github) : github
            this.availableVersion = site.release.version
            this.notes = site.releaseNotes
            this.warning = '软件站已有新版本，自动安装尚未就绪，可先下载后手动安装。'
          }
          this.checked = true
        } catch (error) {
          if (!site || !github || !newer(site.release.version, this.currentVersion)) throw error
          this.manualDownloadUrl = this.source === 'mirror' ? mirrorUrl(template, github) : github
          this.availableVersion = site.release.version
          this.notes = site.releaseNotes
          this.warning = '自动安装服务暂不可用，可在浏览器下载此版本后手动安装。'
        }
      } catch (error) {
        if (!silent) this.error = error instanceof Error ? error.message : String(error)
      } finally {
        this.checking = false
      }
    },
    async install() {
      if (!this.supported) return
      if (this.android || this.manualDownloadUrl) {
        if (!(this.manualDownloadUrl || this.androidDownloadUrl) || this.installing) return
        this.installing = true
        this.error = null
        try {
          await openUpdateDownload((this.manualDownloadUrl || this.androidDownloadUrl)!, this.downloadMirror)
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
