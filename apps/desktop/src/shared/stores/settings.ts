import { open } from '@tauri-apps/plugin-dialog'
import { defineStore } from 'pinia'

import type { DocumentTreeDirectory, EnvironmentHealthSnapshot, SettingsSnapshot, MediaPreferences, NamingPreset, DownloadPreset } from '../api/dto'
import {
  diagnosticsExport,
  environmentCreateDownloadDirectory,
  environmentHealth,
  maintenanceCleanupCache,
  maintenanceCleanupTemp,
  mobilePickExportDirectory,
  settingsGet,
  settingsUpdate,
} from '../api/tauri'
import { audioQualityOptions, videoQualityOptions } from '../features/settings/settingsCatalog'
import { useUiStore } from './ui'
import type { InlineNotice, NoticeTone } from './feedback'
import { NOTICE_CLEAR_DELAY } from './feedback'
import { fillMissingDefaults } from '../utils/settingsDefaults'
import { builtinDownloadPresets, cloneDownloadPresets, migrateDownloadPresets, downloadPresetsError } from '../utils/downloadWorkflow'

export const defaultNamingTemplate = '{title}/P{part_index} - {part_title}'
export const embeddingContainerError = (
  settings: Pick<SettingsSnapshot, 'output_extension' | 'embed_cover' | 'embed_subtitles'>,
): string | null =>
  settings.output_extension !== 'mkv' && (settings.embed_cover || settings.embed_subtitles)
    ? '嵌入封面和字幕仅支持 MKV 封装，请改用 MKV 或关闭嵌入选项。'
    : null

export const selectedArchiveAssets = (
  settings: Pick<SettingsSnapshot, 'archive_mode' | 'archive_assets'>,
): SettingsSnapshot['archive_assets'] => {
  if (settings.archive_mode === 'custom') return { ...settings.archive_assets }
  const selected = settings.archive_mode === 'complete_archive'
  return { cover: selected, subtitles: selected, danmaku: selected, nfo: selected }
}

export const namingTemplatePresets = [
  { label: '分P视频', value: defaultNamingTemplate },
  { label: '单文件', value: '{title}' },
  { label: '合集/列表', value: '{collection_title}/{index} - {title}' },
  { label: '番剧/课程', value: '{series_title}/{episode_index} - {episode_title}' },
]

export const builtinNamingPresets = (): NamingPreset[] => [
  ...namingTemplatePresets.map((preset, index) => ({ id: `builtin-${index}`, name: preset.label, template: preset.value })),
  { id: 'direct', name: '直接保存到下载目录', template: '{title} - P{part_index} - {part_title}' },
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
  { name: 'episode_index', desc: '集序号' },
  { name: 'episode_title', desc: '集标题' },
  { name: 'collection_title', desc: '合集名' },
  { name: 'index', desc: '列表序号' },
  { name: 'quality', desc: '清晰度编号（如 80）' },
  { name: 'quality_label', desc: '清晰度名称（如 1080P）' },
  { name: 'date:%Y.%m.%d', desc: '下载日期（点分）' },
  { name: 'publish_date:%Y.%m.%d', desc: '发布日期（点分）' },
  { name: 'date:%Y%m%d', desc: '下载日期（紧凑）' },
  { name: 'publish_date:%Y%m%d', desc: '发布日期（紧凑）' },
  { name: 'codec', desc: '编码' },
  { name: 'date', desc: '下载日期（任务创建日）' },
  { name: 'publish_date', desc: '发布时间（B站发布日期）' },
]

const defaultSettings = (): SettingsSnapshot => ({
  settings_schema_version: 1,
  clipboard_monitor_enabled: false,
  usage_notice_acknowledged: false,
  auto_check_updates: false,
  theme_preference: 'system',
  parse_rules: { pages_per_round: 3, interval_seconds: 1, rest_seconds: 3 },
  download_dir: null,
  document_tree_output: null,
  naming_template: defaultNamingTemplate,
  naming_presets: [],
  download_presets: builtinDownloadPresets(),
  selected_download_preset: 'video',
  quality: 'best',
  archive_mode: 'fast',
  archive_assets: {
    cover: true,
    subtitles: true,
    danmaku: true,
    nfo: true,
  },
  output_extension: 'mp4',
  media_mode: 'audio_video',
  audio_output_format: 'm4s',
  subtitle_format: 'srt',
  danmaku_format: 'xml',
  duplicate_naming_strategy: 'skip_existing',
  audio_quality: 'best',
  codec: 'auto',
  media_preferences: { video: [], audio: [], video_codecs: [], audio_codecs: [] },
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

const videoQualities = new Set(videoQualityOptions.map((option) => option.value))
const audioQualities = new Set(audioQualityOptions.map((option) => option.value))
const archiveModes = new Set<SettingsSnapshot['archive_mode']>(['fast', 'complete_archive', 'custom'])
const outputExtensions = new Set<SettingsSnapshot['output_extension']>(['mp4', 'mkv'])
const duplicateNamingStrategies = new Set<SettingsSnapshot['duplicate_naming_strategy']>([
  'skip_existing',
  'overwrite_existing',
  'append_suffix',
])
const codecPreferences = new Set<SettingsSnapshot['codec']>(['auto', 'avc', 'hevc', 'av1'])
const missingQualityPolicies = new Set<SettingsSnapshot['missing_quality_policy']>(['lower', 'skip', 'ask'])
const logLevels = new Set<SettingsSnapshot['log_level']>(['debug', 'info', 'warning', 'error'])
const concurrentTaskCounts = new Set([1, 2, 3, 5])
const retryCounts = new Set([0, 1, 3, 5])
const segmentCounts = new Set([1, 2, 4, 8])
// `season_index` was advertised before the source model had a reliable season ordinal.
// Extensions are automatic now. Accept both legacy variables without offering them in the menu.
const namingVariableNames = new Set([...namingVariables.map((variable) => variable.name), 'season_index', 'ext'])

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
  environmentInitialized: boolean
  environmentCheckId: number
}

interface EnvironmentCheckOverrides {
  downloadDir?: string | null
  ffmpegPath?: string | null
}

type AppPreferences = Pick<SettingsSnapshot, 'clipboard_monitor_enabled' | 'usage_notice_acknowledged' | 'auto_check_updates' | 'theme_preference' | 'selected_download_preset' | 'download_presets' | 'naming_template'>
type DownloadPreferences = Pick<SettingsSnapshot, 'media_mode' | 'audio_output_format' | 'subtitle_format' | 'danmaku_format' | 'archive_mode' | 'archive_assets' | 'embed_cover' | 'embed_subtitles' | 'output_extension' | 'retain_raw_streams'>
const settingsWrites = new WeakMap<object, Promise<unknown>>()
const enqueueSettingsWrite = <T>(store: object, write: () => Promise<T>): Promise<T> => {
  const pending = (settingsWrites.get(store) ?? Promise.resolve()).then(write)
  settingsWrites.set(store, pending.catch(() => undefined))
  return pending
}
const settingsLoads = new WeakMap<object, Promise<void>>()

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
    environmentInitialized: false,
    environmentCheckId: 0,
  }),
  getters: {
    downloadPresetError(state): string | null {
      return downloadPresetsError(state.draft.download_presets, state.draft.selected_download_preset)
    },
    changed(state): boolean {
      return JSON.stringify(state.saved) !== JSON.stringify(state.draft)
    },
    namingPreview(state): string {
      return previewTemplate(state.draft.naming_template, state.draft.output_extension)
    },
    namingTemplateError(state): string | null {
      return validateNamingTemplate(state.draft.naming_template)
    },
    environmentReady(state): boolean {
      return state.environmentInitialized && state.environmentHealth?.ready === true
    },
  },
  actions: {
    async ensureLoaded() {
      if (this.loaded) return
      const pending = settingsLoads.get(this)
      if (pending) return pending
      const loading = this.load()
      settingsLoads.set(this, loading)
      try {
        await loading
      } finally {
        settingsLoads.delete(this)
      }
    },
    async initializeEnvironment(): Promise<EnvironmentHealthSnapshot | null> {
      if (this.environmentInitialized) {
        return this.environmentHealth
      }

      await this.ensureLoaded()
      if (!this.loaded) {
        this.environmentInitialized = true
        return null
      }

      return this.checkEnvironment()
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
    saveAppPreferences(patch: Partial<AppPreferences | DownloadPreferences>): Promise<boolean> {
      return enqueueSettingsWrite(this, async () => {
        await this.ensureLoaded()
        if (!this.loaded) {
          useUiStore().pushToast('设置未能读取，本次偏好无法保存。', 'warning')
          return false
        }
        try {
          const updated = normalizeSettings(await settingsUpdate({ ...cloneSettings(this.saved), ...patch }))
          this.saved = cloneSettings(updated)
          Object.assign(this.draft, patch)
          return true
        } catch (error) {
          useUiStore().pushToast(`设置保存失败：${errorMessage(error)}`, 'warning')
          return false
        }
      })
    },
    save() {
      return enqueueSettingsWrite(this, () => this.saveDraft())
    },
    persistDownloadPreset(preset: DownloadPreset, creating: boolean, applyOnSave: boolean): Promise<boolean> {
      return enqueueSettingsWrite(this, async () => {
        await this.ensureLoaded()
        if (!this.loaded) return false
        if (!creating && !this.saved.download_presets.some((item) => item.id === preset.id)) return false
        const items = creating ? [...this.saved.download_presets, preset] : this.saved.download_presets.map((item) => item.id === preset.id ? preset : item)
        const selected = applyOnSave ? preset.id : this.saved.selected_download_preset
        if (downloadPresetsError(items, selected)) return false
        try {
          const updated = normalizeSettings(await settingsUpdate({ ...cloneSettings(this.saved), download_presets: items, selected_download_preset: selected }))
          this.saved = cloneSettings(updated)
          this.draft.download_presets = cloneDownloadPresets(creating ? [...this.draft.download_presets, preset] : this.draft.download_presets.map((item) => item.id === preset.id ? preset : item))
          if (applyOnSave) this.draft.selected_download_preset = preset.id
          return true
        } catch (error) {
          useUiStore().pushToast(`预设保存失败：${errorMessage(error)}`, 'warning')
          return false
        }
      })
    },
    deleteDownloadPreset(id: string): Promise<boolean> {
      return enqueueSettingsWrite(this, async () => {
        await this.ensureLoaded()
        if (!this.loaded || this.saved.download_presets.length <= 1 || !this.saved.download_presets.some((item) => item.id === id)) return false
        const items = this.saved.download_presets.filter((item) => item.id !== id)
        const selected = this.saved.selected_download_preset === id ? items[0].id : this.saved.selected_download_preset
        try {
          const updated = normalizeSettings(await settingsUpdate({ ...cloneSettings(this.saved), download_presets: items, selected_download_preset: selected }))
          this.saved = cloneSettings(updated)
          this.draft.download_presets = this.draft.download_presets.filter((item) => item.id !== id)
          if (this.draft.selected_download_preset === id) this.draft.selected_download_preset = selected
          return true
        } catch (error) {
          useUiStore().pushToast(`预设删除失败：${errorMessage(error)}`, 'warning')
          return false
        }
      })
    },
    persistNamingPreset(preset: NamingPreset, creating: boolean): Promise<boolean> {
      return enqueueSettingsWrite(this, async () => {
        await this.ensureLoaded()
        if (!this.loaded) return false
        if (!preset.name.trim() || [...preset.name].length > 40 || validateNamingTemplate(preset.template)) return false
        if (creating && (this.saved.naming_presets.length >= 32 || this.draft.naming_presets.length >= 32)) return false
        if ([...this.saved.naming_presets, ...this.draft.naming_presets].some((item) => item.id !== preset.id && item.name.trim() === preset.name)) return false
        if (!creating && !this.saved.naming_presets.some((item) => item.id === preset.id)) return false
        const previous = this.saved.naming_presets.find((item) => item.id === preset.id)
        const items = creating ? [...this.saved.naming_presets, { ...preset }] : this.saved.naming_presets.map((item) => item.id === preset.id ? { ...preset } : item)
        const namingTemplate = !creating && previous?.template === this.saved.naming_template ? preset.template : this.saved.naming_template
        try {
          const updated = normalizeSettings(await settingsUpdate({ ...cloneSettings(this.saved), naming_presets: items, naming_template: namingTemplate }))
          this.saved = cloneSettings(updated)
          this.draft.naming_presets = creating ? [...this.draft.naming_presets, { ...preset }] : this.draft.naming_presets.map((item) => item.id === preset.id ? { ...preset } : item)
          if (!creating && previous?.template === this.draft.naming_template) this.draft.naming_template = preset.template
          return true
        } catch (error) {
          useUiStore().pushToast(`预设保存失败：${errorMessage(error)}`, 'warning')
          return false
        }
      })
    },
    deleteNamingPreset(id: string): Promise<boolean> {
      return enqueueSettingsWrite(this, async () => {
        await this.ensureLoaded()
        if (!this.loaded || !this.saved.naming_presets.some((item) => item.id === id)) return false
        try {
          const updated = normalizeSettings(await settingsUpdate({ ...cloneSettings(this.saved), naming_presets: this.saved.naming_presets.filter((item) => item.id !== id) }))
          this.saved = cloneSettings(updated)
          this.draft.naming_presets = this.draft.naming_presets.filter((item) => item.id !== id)
          return true
        } catch (error) {
          useUiStore().pushToast(`预设删除失败：${errorMessage(error)}`, 'warning')
          return false
        }
      })
    },
    async saveDraft() {
      const ui = useUiStore()
      const namingError = validateNamingTemplate(this.draft.naming_template)
      const compatibilityError = embeddingContainerError(this.draft) ?? this.downloadPresetError
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
        await this.checkEnvironment()
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
      } finally {
        this.saving = false
      }
    },
    async chooseDownloadDir(): Promise<boolean> {
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
          return true
        }
        return false
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
        return false
      }
    },
    async chooseDocumentTreeOutput(): Promise<boolean> {
      const ui = useUiStore()
      try {
        this.draft.document_tree_output = await mobilePickExportDirectory()
        return true
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
        return false
      }
    },
    saveDefaultDocumentTreeOutput(directory: DocumentTreeDirectory): Promise<boolean> {
      return enqueueSettingsWrite(this, () => this.persistDefaultDocumentTreeOutput(directory))
    },
    async persistDefaultDocumentTreeOutput(directory: DocumentTreeDirectory): Promise<boolean> {
      const ui = useUiStore()
      const previousSaved = this.saved.document_tree_output
      const draftFollowedSaved = JSON.stringify(this.draft.document_tree_output) === JSON.stringify(previousSaved)
      try {
        const updated = normalizeSettings(await settingsUpdate({
          ...cloneSettings(this.saved),
          document_tree_output: { ...directory },
        }))
        this.saved = cloneSettings(updated)
        if (draftFollowedSaved) {
          this.draft.document_tree_output = updated.document_tree_output
            ? { ...updated.document_tree_output }
            : null
        }
        this.setNotice('已设为默认导出目录', 'success')
        return true
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
        return false
      }
    },
    clearDocumentTreeOutput() {
      this.draft.document_tree_output = null
    },
    async chooseFfmpegPath(): Promise<boolean> {
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
          return true
        }
        return false
      } catch (error) {
        this.error = errorMessage(error)
        ui.pushToast(this.error, 'danger')
        return false
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
          download_dir: overrides.downloadDir !== undefined ? overrides.downloadDir : this.saved.download_dir,
          ffmpeg_path: overrides.ffmpegPath !== undefined ? overrides.ffmpegPath : this.saved.ffmpeg_path,
        })
        if (checkId !== this.environmentCheckId) {
          return null
        }
        this.environmentHealth = health
        this.environmentInitialized = true
        return health
      } catch (error) {
        if (checkId === this.environmentCheckId) {
          this.environmentInitialized = true
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
    async cleanupTemp(token: string) {
      const result = await maintenanceCleanupTemp(token)
      this.setNotice(`已清理临时文件 ${result.removed_files} 个`, 'success')
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
    },
    restoreDefaults() {
      this.draft = normalizeSettings(defaultSettings())
    },
    setDownloadDir(value: string) {
      const trimmed = value.trim()
      this.draft.download_dir = trimmed ? trimmed : null
    },
    saveNamingPreset(name: string): boolean {
      const trimmed = name.trim()
      const error = validateNamingTemplate(this.draft.naming_template)
      if (!trimmed || [...trimmed].length > 40 || error) {
        this.setNotice(error ?? '请输入 1–40 个字符的预设名称。', 'warning')
        return false
      }
      const existing = this.draft.naming_presets.find((preset) => preset.name === trimmed)
      if (!existing && this.draft.naming_presets.length >= 32) {
        this.setNotice('命名预设最多保存 32 个。', 'warning')
        return false
      }
      const preset: NamingPreset = { id: existing?.id ?? crypto.randomUUID(), name: trimmed, template: this.draft.naming_template }
      this.draft.naming_presets = existing
        ? this.draft.naming_presets.map((item) => item.id === existing.id ? preset : item)
        : [...this.draft.naming_presets, preset]
      return true
    },
    removeNamingPreset(id: string) {
      this.draft.naming_presets = this.draft.naming_presets.filter((preset) => preset.id !== id)
    },
    setNamingTemplate(value: string) {
      this.draft.naming_template = value
    },
    setMediaPreferences(value: MediaPreferences) {
      this.draft.media_preferences = cloneMediaPreferences(value)
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
        ...selectedArchiveAssets(this.draft),
        [kind]: value,
      }
      this.draft.archive_mode = 'custom'
    },
    setOutputExtension(value: string) {
      this.draft.output_extension = outputExtensions.has(value as SettingsSnapshot['output_extension'])
        ? (value as SettingsSnapshot['output_extension'])
        : 'mp4'
      if (this.draft.output_extension !== 'mkv') {
        this.draft.embed_cover = false
        this.draft.embed_subtitles = false
      }
    },
    setDuplicateNamingStrategy(value: string) {
      this.draft.duplicate_naming_strategy = duplicateNamingStrategies.has(
        value as SettingsSnapshot['duplicate_naming_strategy'],
      )
        ? (value as SettingsSnapshot['duplicate_naming_strategy'])
        : 'skip_existing'
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
    },
    clearFfmpegPath() {
      this.draft.ffmpeg_path = null
    },
    setRetainRawStreams(value: boolean) {
      this.draft.retain_raw_streams = value
    },
    setEmbedCover(value: boolean) {
      this.draft.embed_cover = value
      if (value) this.draft.output_extension = 'mkv'
    },
    setEmbedSubtitles(value: boolean) {
      this.draft.embed_subtitles = value
      if (value) this.draft.output_extension = 'mkv'
    },
    apply(settings: SettingsSnapshot) {
      const normalized = normalizeSettings(settings)
      this.saved = cloneSettings(normalized)
      this.draft = cloneSettings(normalized)
      this.loaded = true
    },
  },
})

const normalizeSettings = (saved: SettingsSnapshot): SettingsSnapshot => {
  const settings = fillMissingDefaults(defaultSettings(), saved)
  return {
    ...settings,
    ...migrateDownloadPresets(saved.download_presets?.length ? settings : { ...settings, download_presets: [], selected_download_preset: '' }),
    parse_rules: { pages_per_round: settings.parse_rules?.pages_per_round ?? 3, interval_seconds: settings.parse_rules?.interval_seconds ?? 1, rest_seconds: settings.parse_rules?.rest_seconds ?? 3 },
    download_dir: settings.download_dir?.trim() || null,
    document_tree_output: settings.document_tree_output
      ? {
          tree_uri: settings.document_tree_output.tree_uri,
          display_name: settings.document_tree_output.display_name,
        }
      : null,
    naming_template: normalizeNamingTemplate(settings.naming_template),
    naming_presets: (settings.naming_presets ?? []).map((preset) => ({ ...preset })),
    quality: videoQualities.has(settings.quality) ? settings.quality : 'best',
    archive_mode: archiveModes.has(settings.archive_mode) ? settings.archive_mode : 'fast',
    archive_assets: normalizeArchiveAssets(settings.archive_assets),
    output_extension: outputExtensions.has(settings.output_extension) ? settings.output_extension : 'mp4',
    media_mode: ['audio_video', 'video_only', 'audio_only'].includes(settings.media_mode) ? settings.media_mode : 'audio_video',
    audio_output_format: settings.audio_output_format === 'mp3' ? 'mp3' : 'm4s',
    subtitle_format: settings.subtitle_format === 'ass' ? 'ass' : 'srt',
    danmaku_format: settings.danmaku_format === 'html' || settings.danmaku_format === 'srt' || settings.danmaku_format === 'ass' ? settings.danmaku_format : 'xml',
    duplicate_naming_strategy: duplicateNamingStrategies.has(settings.duplicate_naming_strategy)
      ? settings.duplicate_naming_strategy
      : 'skip_existing',
    audio_quality: audioQualities.has(settings.audio_quality) ? settings.audio_quality : 'best',
    codec: codecPreferences.has(settings.codec) ? settings.codec : 'auto',
    media_preferences: cloneMediaPreferences(settings.media_preferences),
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
  }
}

const cloneSettings = (settings: SettingsSnapshot): SettingsSnapshot => ({
  ...settings,
  download_presets: cloneDownloadPresets(settings.download_presets ?? []),
  parse_rules: { ...settings.parse_rules },
  document_tree_output: settings.document_tree_output ? { ...settings.document_tree_output } : null,
  archive_assets: { ...settings.archive_assets },
  naming_presets: (settings.naming_presets ?? []).map((preset) => ({ ...preset })),
  media_preferences: cloneMediaPreferences(settings.media_preferences),
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
  if (!trimmed) {
    return defaultNamingTemplate
  }

  return trimmed
}

export const validateNamingTemplate = (template: string): string | null => {
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
    if (!namingVariableNames.has(name) && !/^(date|publish_date):%Y-%m-%d$/.test(name)) {
      return `命名模板存在未知变量 {${name}}。`
    }

    index = end
  }

  return null
}

export const previewTemplate = (template: string, ext: string): string => {
  const now = new Date()
  const localDate = [
    now.getFullYear(),
    String(now.getMonth() + 1).padStart(2, '0'),
    String(now.getDate()).padStart(2, '0'),
  ].join('-')
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
    quality_label: '1080P',
    codec: 'avc',
    date: localDate,
    publish_date: '2024-01-02',
    ext,
  }

  const rendered = (template || defaultSettings().naming_template).replace(
    /\{([^{}]+)\}/g,
    (_, key: string) => {
      const [name, format] = key.trim().split(':')
      const value = values[name] ?? ''
      if (!format) return value
      if (!['date', 'publish_date'].includes(name)) return ''
      if (format === '%Y.%m.%d') return value.replaceAll('-', '.')
      if (format === '%Y%m%d') return value.replaceAll('-', '')
      return format === '%Y-%m-%d' ? value : ''
    },
  )
  const extension = ext.trim().replace(/^\.+|\.+$/g, '')
  const path = rendered.split(/[/\\]/).map((component) => component
    .trim().replace(/^\.+|\.+$/g, '') || 'untitled').join('/')
  return extension && !path.toLowerCase().endsWith(`.${extension.toLowerCase()}`)
    ? `${path}.${extension}`
    : path
}

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}

export const cloneMediaPreferences = (value?: MediaPreferences): MediaPreferences => ({
  video: (value?.video ?? []).flatMap((rule) => rule.quality === 'best'
    ? videoQualityOptions.filter((option) => /^\d+$/.test(option.value)).map((option) => ({ quality: option.value, codec: rule.codec }))
    : [{ ...rule }]),
  audio: [...(value?.audio ?? [])],
  video_codecs: [...(value?.video_codecs ?? [])],
  audio_codecs: [...(value?.audio_codecs ?? [])],
})
