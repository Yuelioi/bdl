import { open } from '@tauri-apps/plugin-dialog'
import { defineStore } from 'pinia'

import type { SettingsSnapshot } from '../api/dto'
import { settingsGet, settingsUpdate } from '../api/tauri'
import { useUiStore } from './ui'

const defaultSettings = (): SettingsSnapshot => ({
  download_dir: null,
  quality: 'best',
  archive_mode: 'fast',
  output_extension: 'mp4',
  concurrent_tasks: 1,
  retry_count: 3,
  auto_refresh_expired_urls: true,
})

const archiveModes = new Set<SettingsSnapshot['archive_mode']>(['fast', 'complete_archive'])
const outputExtensions = new Set<SettingsSnapshot['output_extension']>(['mp4', 'mkv'])
const concurrentTaskCounts = new Set([1, 2, 3, 5])
const retryCounts = new Set([0, 1, 3, 5])

interface SettingsState {
  saved: SettingsSnapshot
  draft: SettingsSnapshot
  loaded: boolean
  loading: boolean
  saving: boolean
  error: string | null
}

export const useSettingsStore = defineStore('settings', {
  state: (): SettingsState => ({
    saved: defaultSettings(),
    draft: defaultSettings(),
    loaded: false,
    loading: false,
    saving: false,
    error: null,
  }),
  getters: {
    changed(state): boolean {
      return JSON.stringify(state.saved) !== JSON.stringify(state.draft)
    },
  },
  actions: {
    async ensureLoaded() {
      if (this.loaded || this.loading) {
        return
      }

      await this.load()
    },
    async load() {
      this.loading = true
      this.error = null
      try {
        this.apply(await settingsGet())
      } catch (error) {
        this.error = errorMessage(error)
      } finally {
        this.loading = false
      }
    },
    async save() {
      const ui = useUiStore()
      this.saving = true
      this.error = null
      try {
        this.apply(await settingsUpdate(normalizeSettings(this.draft)))
        ui.pushToast('设置已保存', 'success')
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
      } finally {
        this.saving = false
      }
    },
    async chooseDownloadDir() {
      const ui = useUiStore()
      try {
        const selected = await open({
          directory: true,
          multiple: false,
          title: '选择保存目录',
          defaultPath: this.draft.download_dir ?? undefined,
        })

        if (typeof selected === 'string') {
          this.setDownloadDir(selected)
        }
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
      }
    },
    resetDraft() {
      this.draft = { ...this.saved }
    },
    setDownloadDir(value: string) {
      const trimmed = value.trim()
      this.draft.download_dir = trimmed ? trimmed : null
    },
    setArchiveMode(value: string) {
      this.draft.archive_mode = archiveModes.has(value as SettingsSnapshot['archive_mode'])
        ? (value as SettingsSnapshot['archive_mode'])
        : 'fast'
    },
    setOutputExtension(value: string) {
      this.draft.output_extension = outputExtensions.has(value as SettingsSnapshot['output_extension'])
        ? (value as SettingsSnapshot['output_extension'])
        : 'mp4'
    },
    setConcurrentTasks(value: string) {
      const count = Number(value)
      this.draft.concurrent_tasks = concurrentTaskCounts.has(count) ? count : 1
    },
    setRetryCount(value: string) {
      const count = Number(value)
      this.draft.retry_count = retryCounts.has(count) ? count : 3
    },
    setAutoRefreshExpiredUrls(value: boolean) {
      this.draft.auto_refresh_expired_urls = value
    },
    apply(settings: SettingsSnapshot) {
      const normalized = normalizeSettings(settings)
      this.saved = normalized
      this.draft = { ...normalized }
      this.loaded = true
    },
  },
})

const normalizeSettings = (settings: SettingsSnapshot): SettingsSnapshot => ({
  download_dir: settings.download_dir?.trim() || null,
  quality: settings.quality || 'best',
  archive_mode: archiveModes.has(settings.archive_mode) ? settings.archive_mode : 'fast',
  output_extension: outputExtensions.has(settings.output_extension) ? settings.output_extension : 'mp4',
  concurrent_tasks: concurrentTaskCounts.has(settings.concurrent_tasks) ? settings.concurrent_tasks : 1,
  retry_count: retryCounts.has(settings.retry_count) ? settings.retry_count : 3,
  auto_refresh_expired_urls: settings.auto_refresh_expired_urls !== false,
})

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
