import { open } from '@tauri-apps/plugin-dialog'
import { defineStore } from 'pinia'

import type { SettingsSnapshot } from '../api/dto'
import { settingsGet, settingsUpdate } from '../api/tauri'
import { useUiStore } from './ui'

export const defaultNamingTemplate = '{title}/P{part_index} - {part_title}.{ext}'
const legacyNamingTemplate = '{title}/{title} - P{part_index} - {part_title}.{ext}'

export const namingTemplatePresets = [
  { label: '分P视频', value: defaultNamingTemplate },
  { label: '单文件', value: '{title}.{ext}' },
  { label: '合集/列表', value: '{collection_title}/{index} - {title}.{ext}' },
  { label: '番剧/课程', value: '{series_title}/S{season_index}E{episode_index} - {episode_title}.{ext}' },
]

export const namingVariables = [
  { name: 'title', desc: '视频或条目标题' },
  { name: 'part_title', desc: '分P标题' },
  { name: 'part_index', desc: '分P序号' },
  { name: 'bvid', desc: 'BV号' },
  { name: 'aid', desc: 'AV号' },
  { name: 'cid', desc: 'CID' },
  { name: 'owner_name', desc: 'UP主名称' },
  { name: 'owner_mid', desc: 'UP主MID' },
  { name: 'series_title', desc: '番剧/课程/系列名' },
  { name: 'season_index', desc: '季序号' },
  { name: 'episode_index', desc: '集序号' },
  { name: 'episode_title', desc: '集标题' },
  { name: 'collection_title', desc: '合集名' },
  { name: 'index', desc: '列表序号' },
  { name: 'quality', desc: '清晰度' },
  { name: 'codec', desc: '编码' },
  { name: 'date', desc: '日期' },
  { name: 'ext', desc: '扩展名' },
]

const defaultSettings = (): SettingsSnapshot => ({
  download_dir: null,
  naming_template: defaultNamingTemplate,
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
    namingPreview(state): string {
      return previewTemplate(state.draft.naming_template, state.draft.output_extension)
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
    setNamingTemplate(value: string) {
      this.draft.naming_template = value
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
  naming_template: normalizeNamingTemplate(settings.naming_template),
  quality: settings.quality || 'best',
  archive_mode: archiveModes.has(settings.archive_mode) ? settings.archive_mode : 'fast',
  output_extension: outputExtensions.has(settings.output_extension) ? settings.output_extension : 'mp4',
  concurrent_tasks: concurrentTaskCounts.has(settings.concurrent_tasks) ? settings.concurrent_tasks : 1,
  retry_count: retryCounts.has(settings.retry_count) ? settings.retry_count : 3,
  auto_refresh_expired_urls: settings.auto_refresh_expired_urls !== false,
})

const normalizeNamingTemplate = (template: string | null | undefined): string => {
  const trimmed = template?.trim()
  if (!trimmed || trimmed === legacyNamingTemplate) {
    return defaultNamingTemplate
  }

  return trimmed
}

const previewTemplate = (template: string, ext: string): string => {
  const values: Record<string, string> = {
    title: '示例视频',
    part_title: '开场',
    part_index: '1',
    bvid: 'BV1xx411c7mD',
    aid: '170001',
    cid: '9001',
    owner_name: '示例UP',
    owner_mid: '1001',
    series_title: '示例系列',
    season_index: '1',
    episode_index: '1',
    episode_title: '第一集',
    collection_title: '示例合集',
    index: '1',
    quality: '80',
    codec: 'avc',
    date: new Date().toISOString().slice(0, 10),
    ext,
  }

  return (template || defaultSettings().naming_template).replace(/\{([^{}]+)\}/g, (_, key: string) => values[key.trim()] ?? '')
}

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
