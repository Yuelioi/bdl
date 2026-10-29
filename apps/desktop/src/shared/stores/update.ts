import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import type { Update } from '@tauri-apps/plugin-updater'
import { defineStore } from 'pinia'
import { markRaw } from 'vue'

import { useSettingsStore } from './settings'
import { checkAndroidUpdate } from '../api/androidUpdate'
import { distributionUpdate, checkUpdate, openUpdateDownload, mirrorTemplate, mirrorUrl, newer, type DistributionUpdate } from '../api/updateSources'

type UpdateSource = 'github' | 'mirror' | 'quark' | `official:${string}`

export const useUpdateStore = defineStore('update', {
  state: () => ({
    supported: true,
    source: 'github' as UpdateSource,
    customMirror: '',
    distribution: null as DistributionUpdate | null,
    loadingSources: false,
    sourcesError: null as string | null,
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
    sourceOptions(state): { label: string; value: UpdateSource; disabled?: boolean }[] {
      const options: { label: string; value: UpdateSource; disabled?: boolean }[] = [{ label: 'GitHub 下载', value: 'github' }]
      const github = state.distribution?.sources.find(s => s.kind === 'github')?.delivery.url
      if (github) {
        for (const source of state.distribution!.sources) {
          const template = mirrorTemplate(source, github)
          if (template) options.push({ label: source.label, value: `official:${template}` })
        }
      }
      if (state.source.startsWith('official:') && !options.some(option => option.value === state.source)) {
        options.push({ label: '所选加速源（当前未提供）', value: state.source, disabled: true })
      }
      if (state.source === 'quark' || state.distribution?.sources.some(s => s.kind === 'cloud_drive' && s.delivery.url?.startsWith('https://pan.quark.cn/'))) {
        options.push({ label: '夸克手动下载', value: 'quark' })
      }
      options.push({ label: '自定义加速', value: 'mirror' })
      return options
    },
  },
  actions: {
    async loadSources() {
      if (this.loadingSources || this.checking) return
      this.loadingSources = true
      this.sourcesError = null
      try {
        this.distribution = await distributionUpdate()
      } catch (error) {
        this.sourcesError = error instanceof Error ? error.message : String(error)
      } finally {
        this.loadingSources = false
      }
    },
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
    setSource(source: UpdateSource, customMirror?: string) {
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
        if (typeof saved.source === 'string' && (['github', 'mirror', 'quark'].includes(saved.source) || saved.source.startsWith('official:'))) this.source = saved.source as UpdateSource
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
      try {
        let siteError: unknown
        let site: DistributionUpdate | null = null
        try {
          site = await distributionUpdate()
          this.distribution = site
          this.sourcesError = null
        } catch (error) { siteError = error }
        const github = site?.sources.find(s => s.kind === 'github')?.delivery.url
        const quark = site?.sources.find(s => s.kind === 'cloud_drive' && s.delivery.url?.startsWith('https://pan.quark.cn/'))
        let template = ''
        if (this.source.startsWith('official:')) {
          if (!site) throw siteError ?? new Error('软件站暂时无法连接。')
          const selected = this.source.slice('official:'.length)
          if (!github || !site.sources.some(s => mirrorTemplate(s, github) === selected)) {
            throw new Error('软件站已不再提供所选加速源，请重新选择更新来源。')
          }
          template = selected
        } else if (this.source === 'mirror') {
          template = this.customMirror
          if (!template) throw new Error('请填写自定义加速地址，或选择软件站提供的更新来源。')
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
        if (template) mirrorUrl(template, 'https://github.com/Yuelioi/bdl/releases/latest/download/latest.json')
        if (template && github) this.downloadMirror = { template, officialUrl: github }
        if (this.android) {
          if (template && site && !github) throw new Error('当前平台尚未提供可加速的安装包，请选择其他来源。')
          if (site && github && newer(site.release.version, this.currentVersion)) {
            this.androidDownloadUrl = template ? mirrorUrl(template, github) : github
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
        try {
          const availableUpdate = await checkUpdate(template ?? undefined)
          this.update = availableUpdate ? markRaw(availableUpdate) : null
          this.availableVersion = this.update?.version ?? null
          this.notes = this.update?.body ?? null
          if (!this.update && site && github && newer(site.release.version, this.currentVersion)) {
            this.manualDownloadUrl = template ? mirrorUrl(template, github) : github
            this.availableVersion = site.release.version
            this.notes = site.releaseNotes
            this.warning = '软件站已有新版本，自动安装尚未就绪，可先下载后手动安装。'
          }
          this.checked = true
        } catch (error) {
          if (!site || !github || !newer(site.release.version, this.currentVersion)) throw error
          this.manualDownloadUrl = template ? mirrorUrl(template, github) : github
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
