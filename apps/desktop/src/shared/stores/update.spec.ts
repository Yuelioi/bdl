import { distributionUpdate, checkUpdate, openUpdateDownload } from '../api/updateSources'
vi.mock('../api/updateSources', async (original) => ({ ...await original<object>(), distributionUpdate: vi.fn(), checkUpdate: vi.fn(), openUpdateDownload: vi.fn() }))
import { useSettingsStore } from './settings'
import { settingsUpdate } from '../api/tauri'
vi.mock('../api/tauri', () => ({ settingsUpdate: vi.fn() }))
import { checkAndroidUpdate } from '../api/androidUpdate'
vi.mock('../api/androidUpdate', () => ({ checkAndroidUpdate: vi.fn() }))
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const { relaunch } = vi.hoisted(() => ({ relaunch: vi.fn() }))
const check = vi.mocked(checkUpdate)

vi.mock('@tauri-apps/api/app', () => ({ getVersion: vi.fn().mockResolvedValue('1.2.3') }))
vi.mock('@tauri-apps/plugin-process', () => ({ relaunch }))

import { useUpdateStore } from './update'

describe('update preferences', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.mocked(distributionUpdate).mockReset().mockRejectedValue(new Error("site offline"))
    vi.mocked(checkUpdate).mockReset()
    setActivePinia(createPinia())
    const settings = useSettingsStore()
    settings.loaded = true
    vi.mocked(settingsUpdate).mockImplementation(async (value) => value)
    check.mockReset().mockResolvedValue(null)
    relaunch.mockReset()
    vi.mocked(checkAndroidUpdate).mockReset().mockResolvedValue(null)
    vi.mocked(openUpdateDownload).mockReset().mockResolvedValue(undefined)
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
    expect(openUpdateDownload).toHaveBeenCalledWith(url, undefined)
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

    check.mockResolvedValue(new PrivateUpdate() as unknown as Awaited<ReturnType<typeof checkUpdate>>)
    const update = useUpdateStore()
    await update.checkForUpdate()
    await update.install()

    expect(update.error).toBeNull()
    expect(relaunch).toHaveBeenCalledOnce()
  })
  const site = {
    release: { version: '1.2.4', fileName: 'BDL.exe' }, releaseNotes: '修复',
    sources: [
      { id: 'github', kind: 'github', label: 'GitHub', delivery: { mode: 'file', authenticationRequired: false, url: 'https://github.com/Yuelioi/bdl/releases/download/v1.2.4/BDL.exe' } },
      { id: 'mirror', kind: 'external_link', label: '加速', delivery: { mode: 'file', authenticationRequired: false, url: 'https://mirror.test/https://github.com/Yuelioi/bdl/releases/download/v1.2.4/BDL.exe' } },
      { id: 'quark', kind: 'cloud_drive', label: '夸克', delivery: { mode: 'browser', authenticationRequired: false, url: 'https://pan.quark.cn/s/test' } },
      { id: 'fast', kind: 'external_link', label: 'GHFast', delivery: { mode: 'file', authenticationRequired: false, url: 'https://fast.test/github.com/Yuelioi/bdl/releases/download/v1.2.4/BDL.exe' } },
    ],
  }
  it('uses the native updater for official and custom mirrors with saved network settings', async () => {
    vi.mocked(distributionUpdate).mockResolvedValue(site)
    vi.mocked(checkUpdate).mockResolvedValue(null)
    useSettingsStore().saved.proxy_url = 'http://127.0.0.1:7890'
    const update = useUpdateStore()
    await update.loadSources()
    const mirrors = update.sourceOptions.filter(option => option.value.startsWith('official:'))
    expect(mirrors.map(option => option.value)).toEqual(['official:https://mirror.test/{url}', 'official:https://fast.test/{path}'])
    update.setSource(mirrors[0]!.value)
    await update.checkForUpdate()
    expect(checkUpdate).toHaveBeenCalledWith('https://mirror.test/{url}')
    await update.install()
    expect(openUpdateDownload).toHaveBeenCalledWith(
      site.sources[1]!.delivery.url,
      { template: 'https://mirror.test/{url}', officialUrl: site.sources[0]!.delivery.url },
    )
    update.setSource(mirrors[1]!.value)
    await update.checkForUpdate()
    await update.install()
    expect(openUpdateDownload).toHaveBeenLastCalledWith(site.sources[3]!.delivery.url, {
      template: 'https://fast.test/{path}', officialUrl: site.sources[0]!.delivery.url,
    })
    vi.mocked(distributionUpdate).mockResolvedValue({ ...site, sources: site.sources.filter(source => source.id !== 'fast') })
    await update.checkForUpdate()
    expect(update.hasUpdate).toBe(false)
    expect(update.error).toContain('不再提供所选加速源')
    expect(update.source).toBe(mirrors[1]!.value)
    update.setSource('mirror')
    await update.checkForUpdate()
    expect(update.error).toContain('请填写自定义加速地址')
    expect(update.hasUpdate).toBe(false)
    update.setSource('mirror', 'https://custom.test/{url}')
    await update.checkForUpdate()
    expect(checkUpdate).toHaveBeenLastCalledWith('https://custom.test/{url}')
    const restored = useUpdateStore(createPinia())
    await restored.initialize()
    expect(restored.source).toBe('mirror')
    expect(restored.customMirror).toBe('https://custom.test/{url}')
  })
  it('opens Quark manually and invalidates previous installation on source change', async () => {
    vi.mocked(distributionUpdate).mockResolvedValue(site)
    const update = useUpdateStore()
    update.setSource('quark')
    await update.checkForUpdate()
    await update.install()
    expect(openUpdateDownload).toHaveBeenCalledWith('https://pan.quark.cn/s/test', undefined)
    expect(check).not.toHaveBeenCalled()
    expect(relaunch).not.toHaveBeenCalled()
    update.setSource('github')
    expect(update.hasUpdate).toBe(false)
    expect(update.checked).toBe(false)
  })
  it('offers explicit manual download if the signed service fails', async () => {
    vi.mocked(distributionUpdate).mockResolvedValue(site)
    check.mockRejectedValue(new Error('offline'))
    const update = useUpdateStore()
    await update.checkForUpdate()
    expect(update.warning).toContain('手动安装')
    expect(update.manualDownloadUrl).toContain('github.com')
    expect(update.update).toBeNull()
  })
  it('opens Quark without checking or requiring a newer version, on desktop and Android', async () => {
    vi.mocked(distributionUpdate).mockResolvedValue(site)
    const update = useUpdateStore()
    update.currentVersion = '99.0.0'
    for (const android of [false, true]) {
      update.android = android
      await update.openQuark()
      expect(update.quarkError).toBeNull()
      expect(update.hasUpdate).toBe(false)
      expect(update.checked).toBe(false)
    }
    expect(openUpdateDownload).toHaveBeenCalledTimes(2)
    expect(openUpdateDownload).toHaveBeenCalledWith('https://pan.quark.cn/s/test')
    expect(check).not.toHaveBeenCalled()
  })
  it('reports unavailable Quark sources and allows retry after failure', async () => {
    const update = useUpdateStore()
    await update.openQuark()
    expect(update.quarkError).toBe('site offline')
    expect(update.openingQuark).toBe(false)
    vi.mocked(distributionUpdate).mockResolvedValue({ ...site, sources: [] })
    await update.openQuark()
    expect(update.quarkError).toContain('尚未提供夸克下载')
    expect(openUpdateDownload).not.toHaveBeenCalled()
    vi.mocked(distributionUpdate).mockResolvedValue(site)
    await update.openQuark()
    expect(update.quarkError).toBeNull()
    expect(openUpdateDownload).toHaveBeenCalledOnce()
  })
  it('does not report latest after a silent failure or retain stale update', async () => {
    const update = useUpdateStore()
    update.androidDownloadUrl = 'https://old.test/apk'
    check.mockRejectedValue(new Error('offline'))
    await update.checkForUpdate(true)
    expect(update.checked).toBe(false)
    expect(update.hasUpdate).toBe(false)
  })

})
