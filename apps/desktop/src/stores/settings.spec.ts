import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { DownloadDirectoryHealth, EnvironmentHealthSnapshot } from '../api/dto'
import { environmentCreateDownloadDirectory, environmentHealth } from '../api/tauri'
import { useSettingsStore } from './settings'

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
