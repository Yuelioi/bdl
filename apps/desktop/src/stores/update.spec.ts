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
    localStorage.clear()
    check.mockReset().mockResolvedValue(null)
    relaunch.mockReset()
  })

  it('does not check automatically by default', async () => {
    const update = useUpdateStore()
    await update.initialize()
    expect(update.autoCheck).toBe(false)
    expect(check).not.toHaveBeenCalled()
    expect(update.currentVersion).toBe('1.2.3')
  })

  it('keeps version reporting but disables updater commands on unsupported platforms', async () => {
    localStorage.setItem('bdl.update.auto-check', 'true')
    const update = useUpdateStore()
    await update.initialize(false)

    expect(update.supported).toBe(false)
    expect(update.currentVersion).toBe('1.2.3')
    expect(update.autoCheck).toBe(false)
    expect(check).not.toHaveBeenCalled()
  })

  it('persists and immediately enables automatic checks', async () => {
    const update = useUpdateStore()
    update.setAutoCheck(true)
    await vi.waitFor(() => expect(check).toHaveBeenCalledOnce())
    expect(localStorage.getItem('bdl.update.auto-check')).toBe('true')
  })

  it('keeps silent startup failures out of the interface', async () => {
    check.mockRejectedValue(new Error('offline'))
    localStorage.setItem('bdl.update.auto-check', 'true')
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
