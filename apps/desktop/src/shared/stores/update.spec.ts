import { useSettingsStore } from './settings'
import { settingsUpdate, openExternalUrl } from '../api/tauri'
vi.mock('../api/tauri', () => ({ settingsUpdate: vi.fn(), openExternalUrl: vi.fn() }))
import { checkAndroidUpdate } from '../api/androidUpdate'
vi.mock('../api/androidUpdate', () => ({ checkAndroidUpdate: vi.fn() }))
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const { check, relaunch } = vi.hoisted(() => ({ check: vi.fn(), relaunch: vi.fn() }))

vi.mock('@tauri-apps/api/app', () => ({ getVersion: vi.fn().mockResolvedValue('1.2.3') }))
vi.mock('@tauri-apps/plugin-process', () => ({ relaunch }))
vi.mock('@tauri-apps/plugin-updater', () => ({ check }))

import { useUpdateStore } from './update'

describe('update preferences', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    const settings = useSettingsStore()
    settings.loaded = true
    vi.mocked(settingsUpdate).mockImplementation(async (value) => value)
    check.mockReset().mockResolvedValue(null)
    relaunch.mockReset()
    vi.mocked(checkAndroidUpdate).mockReset().mockResolvedValue(null)
    vi.mocked(openExternalUrl).mockReset().mockResolvedValue(undefined)
  })

  it('does not check automatically by default', async () => {
    const update = useUpdateStore()
    await update.initialize()
    expect(update.autoCheck).toBe(false)
    expect(check).not.toHaveBeenCalled()
    expect(update.currentVersion).toBe('1.2.3')
  })

  it('keeps version reporting but disables updater commands on unsupported platforms', async () => {
    useSettingsStore().saved.auto_check_updates = true
    const update = useUpdateStore()
    await update.initialize(false)

    expect(update.supported).toBe(false)
    expect(update.currentVersion).toBe('1.2.3')
    expect(update.autoCheck).toBe(false)
    expect(check).not.toHaveBeenCalled()
  })

  it('persists and immediately enables automatic checks', async () => {
    const update = useUpdateStore()
    await update.setAutoCheck(true)
    await vi.waitFor(() => expect(check).toHaveBeenCalledOnce())
    expect(useSettingsStore().saved.auto_check_updates).toBe(true)
  })

  it('checks Android releases and opens the APK without calling desktop installation', async () => {
    const url = 'https://github.com/Yuelioi/bdl/releases/download/v1.2.4/BDL-v1.2.4-android-arm64.apk'
    vi.mocked(checkAndroidUpdate).mockResolvedValue({ version: '1.2.4', notes: '修复', downloadUrl: url })
    const update = useUpdateStore()
    await update.initialize(false, true)
    expect(update.supported).toBe(true)
    expect(checkAndroidUpdate).not.toHaveBeenCalled()
    await update.checkForUpdate()
    expect(checkAndroidUpdate).toHaveBeenCalledWith('1.2.3')
    expect(update.hasUpdate).toBe(true)
    await update.install()
    expect(openExternalUrl).toHaveBeenCalledWith(url)
    expect(check).not.toHaveBeenCalled()
    expect(relaunch).not.toHaveBeenCalled()
    vi.mocked(checkAndroidUpdate).mockResolvedValue(null)
    await update.checkForUpdate()
    expect(update.hasUpdate).toBe(false)
  })

  it('supports silent startup checks on Android when explicitly enabled', async () => {
    useSettingsStore().saved.auto_check_updates = true
    const update = useUpdateStore()
    await update.initialize(false, true)
    await vi.waitFor(() => expect(checkAndroidUpdate).toHaveBeenCalledOnce())
    expect(check).not.toHaveBeenCalled()
  })

  it('keeps silent startup failures out of the interface', async () => {
    check.mockRejectedValue(new Error('offline'))
    useSettingsStore().saved.auto_check_updates = true
    const update = useUpdateStore()
    await update.initialize()
    await vi.waitFor(() => expect(update.checking).toBe(false))
    expect(update.error).toBeNull()
  })

  it('keeps updater class instances callable after storing them', async () => {
    class PrivateUpdate {
      #installed = false

      version = '1.2.4'
      body = 'hotfix'

      async downloadAndInstall() {
        if (!this.#installed) this.#installed = true
      }
    }

    check.mockResolvedValue(new PrivateUpdate())
    const update = useUpdateStore()
    await update.checkForUpdate()
    await update.install()

    expect(update.error).toBeNull()
    expect(relaunch).toHaveBeenCalledOnce()
  })
})
