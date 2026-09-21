import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { DownloadDirectoryHealth, EnvironmentHealthSnapshot } from '../api/dto'
import { environmentCreateDownloadDirectory, environmentHealth, settingsGet, settingsUpdate } from '../api/tauri'
import {
  cloneMediaPreferences,
  embeddingContainerError,
  namingVariables,
  useSettingsStore,
  validateNamingTemplate,
} from './settings'

vi.mock('../api/tauri', () => ({
  diagnosticsExport: vi.fn(),
  environmentCreateDownloadDirectory: vi.fn(),
  environmentHealth: vi.fn(),
  maintenanceCleanupCache: vi.fn(),
  maintenanceCleanupTemp: vi.fn(),
  settingsGet: vi.fn(),
  settingsUpdate: vi.fn(),
}))

const mockedEnvironmentHealth = vi.mocked(environmentHealth)
const mockedCreateDirectory = vi.mocked(environmentCreateDownloadDirectory)
const mockedSettingsGet = vi.mocked(settingsGet)
const mockedSettingsUpdate = vi.mocked(settingsUpdate)

const healthSnapshot = (path: string, status: 'ready' | 'missing'): EnvironmentHealthSnapshot => ({
  ready: status === 'ready',
  download_directory: {
    status,
    path,
    message: status === 'ready' ? '保存目录可写。' : '保存目录尚未创建。',
  },
  ffmpeg: {
    status: 'ready',
    source: 'system',
    path: 'C:\\ffmpeg.exe',
    version: 'ffmpeg version test',
    message: 'FFmpeg 可用。',
  },
})

const deferred = <T>() => {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise
  })
  return { promise, resolve }
}

describe('settings environment health', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    setActivePinia(createPinia())
  })

  it('creates the currently requested directory instead of a cached path', async () => {
    const store = useSettingsStore()
    store.environmentHealth = healthSnapshot('C:\\old', 'missing')
    mockedEnvironmentHealth.mockResolvedValueOnce(healthSnapshot('C:\\current', 'missing'))
    mockedCreateDirectory.mockResolvedValueOnce({
      ...healthSnapshot('C:\\current', 'ready').download_directory,
    })

    await store.createDownloadDirectory({ downloadDir: 'C:\\current' })

    expect(mockedCreateDirectory).toHaveBeenCalledWith('C:\\current')
  })

  it('initializes the application environment only once per session', async () => {
    const store = useSettingsStore()
    mockedSettingsGet.mockResolvedValueOnce(JSON.parse(JSON.stringify(store.saved)))
    mockedEnvironmentHealth.mockResolvedValueOnce(healthSnapshot('C:\\downloads', 'ready'))

    await store.initializeEnvironment()
    await store.initializeEnvironment()

    expect(mockedSettingsGet).toHaveBeenCalledTimes(1)
    expect(mockedEnvironmentHealth).toHaveBeenCalledTimes(1)
    expect(store.environmentReady).toBe(true)
  })

  it('checks the saved runtime configuration instead of unsaved draft paths', async () => {
    const store = useSettingsStore()
    store.saved.download_dir = 'C:\\saved'
    store.saved.ffmpeg_path = 'C:\\saved\\ffmpeg.exe'
    store.draft.download_dir = 'C:\\draft'
    store.draft.ffmpeg_path = 'C:\\draft\\ffmpeg.exe'
    mockedEnvironmentHealth.mockResolvedValueOnce(healthSnapshot('C:\\saved', 'ready'))

    await store.checkEnvironment()

    expect(mockedEnvironmentHealth).toHaveBeenCalledWith({
      download_dir: 'C:\\saved',
      ffmpeg_path: 'C:\\saved\\ffmpeg.exe',
    })
  })

  it('does not let an older directory repair overwrite a newer health check', async () => {
    const store = useSettingsStore()
    const repair = deferred<DownloadDirectoryHealth>()
    mockedEnvironmentHealth
      .mockResolvedValueOnce(healthSnapshot('C:\\first', 'missing'))
      .mockResolvedValueOnce(healthSnapshot('C:\\second', 'ready'))
    mockedCreateDirectory.mockReturnValueOnce(repair.promise)

    const repairPromise = store.createDownloadDirectory({ downloadDir: 'C:\\first' })
    await vi.waitFor(() => expect(mockedCreateDirectory).toHaveBeenCalledWith('C:\\first'))
    await store.checkEnvironment({ downloadDir: 'C:\\second' })
    repair.resolve(healthSnapshot('C:\\first', 'ready').download_directory)
    await repairPromise

    expect(store.environmentHealth?.download_directory.path).toBe('C:\\second')
  })
})

describe('settings media compatibility', () => {
  it('rejects embedding when the output container is MP4', () => {
    const error = embeddingContainerError({
      output_extension: 'mp4',
      embed_cover: true,
      embed_subtitles: false,
    })

    expect(error).toContain('仅支持 MKV')
  })

  it('accepts embedding when the output container is MKV', () => {
    const error = embeddingContainerError({
      output_extension: 'mkv',
      embed_cover: true,
      embed_subtitles: true,
    })

    expect(error).toBeNull()
  })
})

describe('settings defaults', () => {
  it('fills every missing field, including nested objects, using the current defaults', () => {
    setActivePinia(createPinia())
    const store = useSettingsStore()
    const defaults = JSON.parse(JSON.stringify(store.saved))
    const check = (object: Record<string, unknown>, path: string[] = []) => {
      for (const [key, value] of Object.entries(object)) {
        const partial = JSON.parse(JSON.stringify(defaults))
        const parent = path.reduce((node, segment) => node[segment], partial)
        delete parent[key]
        store.apply(partial)
        expect(store.saved, `missing ${[...path, key].join('.')}`).toEqual(defaults)
        if (value && typeof value === 'object' && !Array.isArray(value)) check(value as Record<string, unknown>, [...path, key])
      }
    }
    check(defaults)
  })

  it('keeps explicit choices when loading a partial configuration without a version', () => {
    setActivePinia(createPinia())
    const store = useSettingsStore()
    const partial = {
      naming_template: '{title}/{title} - P{part_index} - {part_title}.{ext}',
      quality: '80', codec: 'hevc', segment_count: 1, retry_count: 0,
      auto_refresh_expired_urls: false, archive_assets: { cover: false },
      parse_rules: { interval_seconds: 2 }, media_preferences: { video: [], fallback: 'error' },
    }
    store.apply(partial as unknown as Parameters<typeof store.apply>[0])
    expect(store.saved).toMatchObject(partial)
    expect(store.saved.parse_rules.rest_seconds).toBe(3)
    expect(store.saved.archive_assets.subtitles).toBe(true)
    expect(store.saved.media_preferences.audio).toEqual([])
  })

  it('defaults name collisions to skipping an existing final output', () => {
    setActivePinia(createPinia())
    const store = useSettingsStore()

    expect(store.saved.duplicate_naming_strategy).toBe('skip_existing')
    expect(store.draft.duplicate_naming_strategy).toBe('skip_existing')
  })

  it('restores the editable draft without overwriting the saved settings', () => {
    setActivePinia(createPinia())
    const store = useSettingsStore()
    store.saved.concurrent_tasks = 5
    store.draft.concurrent_tasks = 3

    store.restoreDefaults()

    expect(store.draft.concurrent_tasks).toBe(1)
    expect(store.saved.concurrent_tasks).toBe(5)
  })
})

describe('naming template persistence', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    setActivePinia(createPinia())
  })

  it('persists date and publish-date variables through the main save action', async () => {
    const store = useSettingsStore()
    const template = '{publish_date} - {date} - {title}.{ext}'
    const updated = { ...store.saved, naming_template: template }
    store.draft.naming_template = template
    mockedSettingsUpdate.mockResolvedValueOnce(updated)
    mockedEnvironmentHealth.mockResolvedValueOnce(healthSnapshot('downloads', 'ready'))

    await store.save()

    expect(mockedSettingsUpdate).toHaveBeenCalledWith(expect.objectContaining({ naming_template: template }))
    expect(store.saved.naming_template).toBe(template)
    expect(store.draft.naming_template).toBe(template)
  })

  it('keeps every advertised magic variable valid', () => {
    for (const variable of namingVariables) {
      expect(validateNamingTemplate(`{${variable.name}}.{ext}`)).toBeNull()
    }
  })
})


describe('media preferences settings', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('preserves HDR quality and independently clones ordered preferences', () => {
    const store = useSettingsStore()
    store.setVideoQuality('sdr')
    expect(store.draft.quality).toBe('sdr')
    store.setVideoQuality('125')
    expect(store.draft.quality).toBe('125')
    store.setVideoQuality('126')
    expect(store.draft.quality).toBe('126')
    const preferences = { video: [{ quality: '125', codec: 'hevc' as const }], audio: ['30251', '30280'], fallback: 'error' as const }
    store.apply({ ...store.saved, media_preferences: preferences })
    store.draft.media_preferences.video[0]!.quality = '126'
    store.draft.media_preferences.audio.reverse()
    expect(store.saved.media_preferences).toEqual(preferences)
    expect(store.changed).toBe(true)
  })

  it('loads settings written before preferences were introduced', () => {
    const store = useSettingsStore()
    const oldSettings = JSON.parse(JSON.stringify(store.saved))
    delete oldSettings.media_preferences
    store.apply(oldSettings)
    expect(store.draft.media_preferences).toEqual({ video: [], audio: [], fallback: 'best' })
  })
})


describe('naming presets', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('keeps personal presets in the draft until saved and updates by name', () => {
    const store = useSettingsStore()
    store.draft.naming_template = '{title}.{ext}'
    expect(store.saveNamingPreset('收藏')).toBe(true)
    const id = store.draft.naming_presets[0].id
    expect(store.saved.naming_presets).toEqual([])
    store.draft.naming_template = '{title}/{part_title}.{ext}'
    store.saveNamingPreset('收藏')
    expect(store.draft.naming_presets).toEqual([{ id, name: '收藏', template: '{title}/{part_title}.{ext}' }])
    store.apply(store.draft)
    store.draft.naming_presets[0].name = '修改'
    expect(store.saved.naming_presets[0].name).toBe('收藏')
    store.resetDraft()
    expect(store.draft.naming_presets[0].name).toBe('收藏')
    store.removeNamingPreset(id)
    expect(store.saved.naming_presets).toHaveLength(1)
  })
})


it('expands legacy unrestricted video rules into the default concrete quality order', () => {
  const legacy = { video: [{ quality: 'best', codec: 'hevc' as const }], audio: [], fallback: 'best' as const }
  const preferences = cloneMediaPreferences(legacy)
  expect(preferences.video.map((rule) => rule.quality)).toEqual(['127', '126', '125', '120', '116', '112', '80', '74', '64', '32', '16'])
  expect(preferences.video.every((rule) => rule.codec === 'hevc')).toBe(true)
  expect(legacy.video[0].quality).toBe('best')
})
