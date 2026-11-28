import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useUpdateStore } from '../stores/update'
import { distributionUpdate, openUpdateDownload } from './updateSources'

vi.mock('./updateSources', async original => ({
  ...await original<object>(), distributionUpdate: vi.fn(), openUpdateDownload: vi.fn(),
}))

const version = '0.8.10'
const fileName = `BDL-v${version}-android-arm64.apk`
const url = `https://github.com/Yuelioi/bdl/releases/download/v${version}/${fileName}`
const release = (name = fileName, official = url) => ({
  release: { version, fileName: name },
  releaseNotes: '修复手机端',
  sources: [{ id: 'github', kind: 'github', label: 'GitHub', delivery: {mode: 'file', url: official, authenticationRequired: false} }],
})

describe('Android release checks', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    const update = useUpdateStore()
    update.android = true
    update.currentVersion = '0.8.9'
    vi.mocked(distributionUpdate).mockReset().mockResolvedValue(release())
    vi.mocked(openUpdateDownload).mockReset()
  })

  it('offers the site-selected APK through GitHub and both mirror formats', async () => {
    const update = useUpdateStore()
    const config = release()
    const mirrors = [`https://gh-proxy.com/${url}`, `https://ghfast.top/${url.slice(8)}`]
    for (const mirror of mirrors) {
      config.sources.push({ id: mirror, label: mirror, kind: 'external_link', delivery: {mode: 'file', url: mirror, authenticationRequired: false} })
    }
    vi.mocked(distributionUpdate).mockResolvedValue(config)
    await update.loadSources()
    const routes = update.sourceOptions.filter(o => o.value === 'github' || o.value.startsWith('official:'))
    expect(routes).toHaveLength(3)
    for (const [i, route] of routes.entries()) {
      update.setSource(route.value)
      await update.checkForUpdate()
      expect(update.availableVersion).toBe(version)
      expect(update.notes).toBe(config.releaseNotes)
      await update.install()
      expect(vi.mocked(openUpdateDownload).mock.lastCall?.[0]).toBe([url, ...mirrors][i])
    }
  })

  it.each(['0.8.10', '0.9.0'])('does not offer a downgrade from %s', async current => {
    const update = useUpdateStore()
    update.currentVersion = current
    await update.checkForUpdate()
    expect(update.checked).toBe(true)
    expect(update.hasUpdate).toBe(false)
  })

  it.each([
    ['BDL.exe', url.replace(fileName, 'BDL.exe')], [fileName, 'https://example.com/app.apk'],
  ])('rejects an incorrect package or repository', async (name, official) => {
    vi.mocked(distributionUpdate).mockResolvedValue(release(name, official))
    const update = useUpdateStore()
    await update.checkForUpdate()
    expect(update.error).toBeTruthy()
    expect(update.hasUpdate).toBe(false)
  })

  it('reports a site failure without announcing that the installed version is latest', async () => {
    vi.mocked(distributionUpdate).mockRejectedValue(new Error('HTTP 503'))
    const update = useUpdateStore()
    await update.checkForUpdate()
    expect(update.error).toContain('HTTP 503')
    expect(update.checked).toBe(false)
  })
})
