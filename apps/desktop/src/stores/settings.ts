import { open } from '@tauri-apps/plugin-dialog'
import { defineStore } from 'pinia'

import type { EnvironmentHealthSnapshot, SettingsSnapshot } from '../api/dto'
import {
  diagnosticsExport,
  environmentCreateDownloadDirectory,
  environmentHealth,
  maintenanceCleanupCache,
  maintenanceCleanupTemp,
  settingsGet,
  settingsUpdate,
} from '../api/tauri'
import { useUiStore } from './ui'
import type { InlineNotice, NoticeTone } from './feedback'
import { NOTICE_CLEAR_DELAY } from './feedback'

export const defaultNamingTemplate = '{title}/P{part_index} - {part_title}.{ext}'
export const embeddingContainerError = (
  settings: Pick<SettingsSnapshot, 'output_extension' | 'embed_cover' | 'embed_subtitles'>,
): string | null =>
  settings.output_extension !== 'mkv' && (settings.embed_cover || settings.embed_subtitles)
    ? '嵌入封面和字幕仅支持 MKV 封装，请改用 MKV 或关闭嵌入选项。'
    : null
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
  settings_schema_version: 1,
  download_dir: null,
  naming_template: defaultNamingTemplate,
  quality: 'best',
  archive_mode: 'fast',
  archive_assets: {
    cover: true,
    subtitles: true,
    danmaku: true,
    nfo: true,
  },
  output_extension: 'mp4',
  duplicate_naming_strategy: 'append_suffix',
  audio_quality: 'best',
  codec: 'auto',
  missing_quality_policy: 'lower',
  ffmpeg_path: null,
  retain_raw_streams: false,
  embed_cover: false,
  embed_subtitles: false,
  proxy_url: null,
  log_level: 'info',
  data_dir: null,
  concurrent_tasks: 1,
  retry_count: 3,
  segment_count: 4,
  global_speed_limit_bytes_per_second: null,
  startup_auto_recovery: false,
  auto_refresh_expired_urls: true,
})

const videoQualities = new Set(['best', '127', '120', '116', '112', '80', '64', '32', '16'])
const audioQualities = new Set(['best', '30280', '30232', '30216'])
const archiveModes = new Set<SettingsSnapshot['archive_mode']>(['fast', 'complete_archive', 'custom'])
const outputExtensions = new Set<SettingsSnapshot['output_extension']>(['mp4', 'mkv'])
const duplicateNamingStrategies = new Set<SettingsSnapshot['duplicate_naming_strategy']>([
  'append_suffix',
  'overwrite_existing',
])
const codecPreferences = new Set<SettingsSnapshot['codec']>(['auto', 'avc', 'hevc', 'av1'])
const missingQualityPolicies = new Set<SettingsSnapshot['missing_quality_policy']>(['lower', 'skip', 'ask'])
const logLevels = new Set<SettingsSnapshot['log_level']>(['debug', 'info', 'warning', 'error'])
const concurrentTaskCounts = new Set([1, 2, 3, 5])
const retryCounts = new Set([0, 1, 3, 5])
const segmentCounts = new Set([1, 2, 4, 8])
const namingVariableNames = new Set(namingVariables.map((variable) => variable.name))

interface SettingsState {
  saved: SettingsSnapshot
  draft: SettingsSnapshot
  loaded: boolean
  loading: boolean
  saving: boolean
  error: string | null
  notice: InlineNotice | null
  noticeTimer: number | null
  environmentHealth: EnvironmentHealthSnapshot | null
  environmentChecking: boolean
  environmentCheckId: number
}

interface EnvironmentCheckOverrides {
  downloadDir?: string | null
  ffmpegPath?: string | null
}

export const useSettingsStore = defineStore('settings', {
  state: (): SettingsState => ({
    saved: defaultSettings(),
    draft: defaultSettings(),
    loaded: false,
    loading: false,
    saving: false,
    error: null,
    notice: null,
    noticeTimer: null,
    environmentHealth: null,
    environmentChecking: false,
    environmentCheckId: 0,
  }),
  getters: {
    changed(state): boolean {
      return JSON.stringify(state.saved) !== JSON.stringify(state.draft)
    },
    namingPreview(state): string {
      return previewTemplate(state.draft.naming_template, state.draft.output_extension)
    },
    namingTemplateError(state): string | null {
      return validateNamingTemplate(state.draft.naming_template)
    },
  },
  actions: {
    async ensureLoaded() {
      if (this.loaded || this.loading) {
        return
      }

      await this.load()
    },
    setNotice(message: string, tone: NoticeTone = 'info') {
      this.notice = { message, tone }
      if (this.noticeTimer !== null && typeof window !== 'undefined') {
        window.clearTimeout(this.noticeTimer)
        this.noticeTimer = null
      }

      if (tone !== 'danger' && typeof window !== 'undefined') {
        this.noticeTimer = window.setTimeout(() => {
          this.notice = null
          this.noticeTimer = null
        }, NOTICE_CLEAR_DELAY)
      }
    },
    clearNotice() {
      this.notice = null
      if (this.noticeTimer !== null && typeof window !== 'undefined') {
        window.clearTimeout(this.noticeTimer)
        this.noticeTimer = null
      }
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
      const namingError = validateNamingTemplate(this.draft.naming_template)
      const compatibilityError = embeddingContainerError(this.draft)
      if (namingError || compatibilityError) {
        this.error = namingError ?? compatibilityError
        this.setNotice(this.error ?? '请检查设置', 'warning')
        return
      }

      this.saving = true
      this.error = null
      try {
        this.apply(await settingsUpdate(normalizeSettings(this.draft)))
        this.setNotice('设置已保存', 'success')
        void this.checkEnvironment()
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
          void this.checkEnvironment()
        }
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
      }
    },
    async chooseFfmpegPath() {
      const ui = useUiStore()
      try {
        const selected = await open({
          directory: false,
          multiple: false,
          title: '选择 FFmpeg',
          defaultPath: this.draft.ffmpeg_path ?? undefined,
        })

        if (typeof selected === 'string') {
          this.setFfmpegPath(selected)
          void this.checkEnvironment()
        }
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
      }
    },
    async chooseDataDir() {
      const ui = useUiStore()
      try {
        const selected = await open({
          directory: true,
          multiple: false,
          title: '选择数据目录',
          defaultPath: this.draft.data_dir ?? undefined,
        })

        if (typeof selected === 'string') {
          this.setDataDir(selected)
        }
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
      }
    },
    async cleanupCache() {
      const ui = useUiStore()
      try {
        const result = await maintenanceCleanupCache()
        this.setNotice(`已清理缓存 ${result.removed_files} 个文件`, 'success')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      }
    },
    async checkEnvironment(overrides: EnvironmentCheckOverrides = {}): Promise<EnvironmentHealthSnapshot | null> {
      const ui = useUiStore()
      const checkId = ++this.environmentCheckId
      this.environmentChecking = true
      try {
        const health = await environmentHealth({
          download_dir: overrides.downloadDir !== undefined ? overrides.downloadDir : this.draft.download_dir,
          ffmpeg_path: overrides.ffmpegPath !== undefined ? overrides.ffmpegPath : this.draft.ffmpeg_path,
        })
        if (checkId !== this.environmentCheckId) {
          return null
        }
        this.environmentHealth = health
        return health
      } catch (error) {
        if (checkId === this.environmentCheckId) {
          ui.pushToast(errorMessage(error), 'danger')
        }
        return null
      } finally {
        if (checkId === this.environmentCheckId) {
          this.environmentChecking = false
        }
      }
    },
    async createDownloadDirectory(overrides: EnvironmentCheckOverrides = {}): Promise<boolean> {
      const ui = useUiStore()
      const health = await this.checkEnvironment(overrides)
      if (!health) {
        return false
      }
      if (health.download_directory.status !== 'missing') {
        return health.download_directory.status === 'ready'
      }

      const checkId = ++this.environmentCheckId
      this.environmentChecking = true
      try {
        const downloadDirectory = await environmentCreateDownloadDirectory(health.download_directory.path)
        if (checkId !== this.environmentCheckId) {
          return false
        }
        this.environmentHealth = {
          ...health,
          download_directory: downloadDirectory,
          ready: downloadDirectory.status === 'ready' && health.ffmpeg.status === 'ready',
        }
        this.setNotice('保存目录已创建', 'success')
        return downloadDirectory.status === 'ready'
      } catch (error) {
        if (checkId === this.environmentCheckId) {
          ui.pushToast(errorMessage(error), 'danger')
        }
        return false
      } finally {
        if (checkId === this.environmentCheckId) {
          this.environmentChecking = false
        }
      }
    },
    async cleanupTemp() {
      const ui = useUiStore()
      try {
        const result = await maintenanceCleanupTemp()
        this.setNotice(`已清理临时文件 ${result.removed_files} 个`, 'success')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      }
    },
    async exportDiagnostics() {
      const ui = useUiStore()
      try {
        const result = await diagnosticsExport()
        this.setNotice(`诊断已导出：${result.path}`, 'success')
      } catch (error) {
        ui.pushToast(errorMessage(error), 'danger')
      }
    },
    resetDraft() {
      this.draft = cloneSettings(this.saved)
      this.invalidateEnvironmentHealth()
      void this.checkEnvironment()
    },
    invalidateEnvironmentHealth() {
      this.environmentCheckId += 1
      this.environmentHealth = null
      this.environmentChecking = false
    },
    setDownloadDir(value: string) {
      const trimmed = value.trim()
      this.draft.download_dir = trimmed ? trimmed : null
      this.invalidateEnvironmentHealth()
    },
    setNamingTemplate(value: string) {
      this.draft.naming_template = value
    },
    setVideoQuality(value: string) {
      this.draft.quality = videoQualities.has(value) ? value : 'best'
    },
    setAudioQuality(value: string) {
      this.draft.audio_quality = audioQualities.has(value) ? value : 'best'
    },
    setCodec(value: string) {
      this.draft.codec = codecPreferences.has(value as SettingsSnapshot['codec'])
        ? (value as SettingsSnapshot['codec'])
        : 'auto'
    },
    setMissingQualityPolicy(value: string) {
      this.draft.missing_quality_policy = missingQualityPolicies.has(
        value as SettingsSnapshot['missing_quality_policy'],
      )
        ? (value as SettingsSnapshot['missing_quality_policy'])
        : 'lower'
    },
    setArchiveMode(value: string) {
      this.draft.archive_mode = archiveModes.has(value as SettingsSnapshot['archive_mode'])
        ? (value as SettingsSnapshot['archive_mode'])
        : 'fast'
    },
    setArchiveAsset(kind: keyof SettingsSnapshot['archive_assets'], value: boolean) {
      this.draft.archive_assets = {
        ...this.draft.archive_assets,
        [kind]: value,
      }
    },
    setOutputExtension(value: string) {
      this.draft.output_extension = outputExtensions.has(value as SettingsSnapshot['output_extension'])
        ? (value as SettingsSnapshot['output_extension'])
        : 'mp4'
    },
    setDuplicateNamingStrategy(value: string) {
      this.draft.duplicate_naming_strategy = duplicateNamingStrategies.has(
        value as SettingsSnapshot['duplicate_naming_strategy'],
      )
        ? (value as SettingsSnapshot['duplicate_naming_strategy'])
        : 'append_suffix'
    },
    setConcurrentTasks(value: string) {
      const count = Number(value)
      this.draft.concurrent_tasks = concurrentTaskCounts.has(count) ? count : 1
    },
    setRetryCount(value: string) {
      const count = Number(value)
      this.draft.retry_count = retryCounts.has(count) ? count : 3
    },
    setSegmentCount(value: string) {
      const count = Number(value)
      this.draft.segment_count = segmentCounts.has(count) ? count : 4
    },
    setGlobalSpeedLimitBytesPerSecond(value: number | null) {
      this.draft.global_speed_limit_bytes_per_second = value && value > 0 ? value : null
    },
    setStartupAutoRecovery(value: boolean) {
      this.draft.startup_auto_recovery = value
    },
    setAutoRefreshExpiredUrls(value: boolean) {
      this.draft.auto_refresh_expired_urls = value
    },
    setProxyUrl(value: string) {
      const trimmed = value.trim()
      this.draft.proxy_url = trimmed ? trimmed : null
    },
    setLogLevel(value: string) {
      this.draft.log_level = logLevels.has(value as SettingsSnapshot['log_level'])
        ? (value as SettingsSnapshot['log_level'])
        : 'info'
    },
    setDataDir(value: string) {
      const trimmed = value.trim()
      this.draft.data_dir = trimmed ? trimmed : null
    },
    clearDataDir() {
      this.draft.data_dir = null
    },
    setFfmpegPath(value: string) {
      const trimmed = value.trim()
      this.draft.ffmpeg_path = trimmed ? trimmed : null
      this.invalidateEnvironmentHealth()
    },
    clearFfmpegPath() {
      this.draft.ffmpeg_path = null
      this.invalidateEnvironmentHealth()
      void this.checkEnvironment()
    },
    setRetainRawStreams(value: boolean) {
      this.draft.retain_raw_streams = value
    },
    setEmbedCover(value: boolean) {
      this.draft.embed_cover = value
    },
    setEmbedSubtitles(value: boolean) {
      this.draft.embed_subtitles = value
    },
    apply(settings: SettingsSnapshot) {
      const normalized = normalizeSettings(settings)
      this.saved = cloneSettings(normalized)
      this.draft = cloneSettings(normalized)
      this.loaded = true
    },
  },
})

const normalizeSettings = (settings: SettingsSnapshot): SettingsSnapshot => ({
  settings_schema_version: settings.settings_schema_version || 1,
  download_dir: settings.download_dir?.trim() || null,
  naming_template: normalizeNamingTemplate(settings.naming_template),
  quality: videoQualities.has(settings.quality) ? settings.quality : 'best',
  archive_mode: archiveModes.has(settings.archive_mode) ? settings.archive_mode : 'fast',
  archive_assets: normalizeArchiveAssets(settings.archive_assets),
  output_extension: outputExtensions.has(settings.output_extension) ? settings.output_extension : 'mp4',
  duplicate_naming_strategy: duplicateNamingStrategies.has(settings.duplicate_naming_strategy)
    ? settings.duplicate_naming_strategy
    : 'append_suffix',
  audio_quality: audioQualities.has(settings.audio_quality) ? settings.audio_quality : 'best',
  codec: codecPreferences.has(settings.codec) ? settings.codec : 'auto',
  missing_quality_policy: missingQualityPolicies.has(settings.missing_quality_policy)
    ? settings.missing_quality_policy
    : 'lower',
  ffmpeg_path: settings.ffmpeg_path?.trim() || null,
  retain_raw_streams: settings.retain_raw_streams === true,
  embed_cover: settings.embed_cover === true,
  embed_subtitles: settings.embed_subtitles === true,
  proxy_url: settings.proxy_url?.trim() || null,
  log_level: logLevels.has(settings.log_level) ? settings.log_level : 'info',
  data_dir: settings.data_dir?.trim() || null,
  concurrent_tasks: concurrentTaskCounts.has(settings.concurrent_tasks) ? settings.concurrent_tasks : 1,
  retry_count: retryCounts.has(settings.retry_count) ? settings.retry_count : 3,
  segment_count: segmentCounts.has(settings.segment_count) ? settings.segment_count : 4,
  global_speed_limit_bytes_per_second:
    settings.global_speed_limit_bytes_per_second && settings.global_speed_limit_bytes_per_second > 0
      ? settings.global_speed_limit_bytes_per_second
      : null,
  startup_auto_recovery: settings.startup_auto_recovery === true,
  auto_refresh_expired_urls: settings.auto_refresh_expired_urls !== false,
})

const cloneSettings = (settings: SettingsSnapshot): SettingsSnapshot => ({
  ...settings,
  archive_assets: { ...settings.archive_assets },
})

const normalizeArchiveAssets = (
  value: SettingsSnapshot['archive_assets'] | null | undefined,
): SettingsSnapshot['archive_assets'] => ({
  cover: value?.cover !== false,
  subtitles: value?.subtitles !== false,
  danmaku: value?.danmaku !== false,
  nfo: value?.nfo !== false,
})

const normalizeNamingTemplate = (template: string | null | undefined): string => {
  const trimmed = template?.trim()
  if (!trimmed || trimmed === legacyNamingTemplate) {
    return defaultNamingTemplate
  }

  return trimmed
}

const validateNamingTemplate = (template: string): string | null => {
  const value = template.trim()
  if (!value) {
    return '命名模板不能为空。'
  }

  for (let index = 0; index < value.length; index += 1) {
    const char = value[index]
    if (char === '}') {
      return '命名模板存在多余的 }。'
    }
    if (char !== '{') {
      continue
    }

    const end = value.indexOf('}', index + 1)
    if (end === -1) {
      return '命名模板存在未闭合变量。'
    }

    const name = value.slice(index + 1, end).trim()
    if (!name) {
      return '命名模板存在空变量。'
    }
    if (!namingVariableNames.has(name)) {
      return `命名模板存在未知变量 {${name}}。`
    }

    index = end
  }

  return null
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
