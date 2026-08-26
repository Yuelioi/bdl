import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { DownloadDirectoryHealth, EnvironmentHealthSnapshot } from '../api/dto'
import { environmentCreateDownloadDirectory, environmentHealth, settingsGet } from '../api/tauri'
import { embeddingContainerError, useSettingsStore } from './settings'

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
