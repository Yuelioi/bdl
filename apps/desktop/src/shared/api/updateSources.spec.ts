import { describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { mirrorTemplate, mirrorUrl, newer, openUpdateDownload } from './updateSources'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))

describe('update transport', () => {
  const official = 'https://github.com/Yuelioi/bdl/releases/download/v1.2.3/BDL.exe'
  it('recognizes both site mirror formats without hardcoding their hosts', () => {
    for (const [url, expected] of [
      [`https://first.test/${official}`, 'https://first.test/{url}'],
      [`https://second.test/${official.slice(8)}`, 'https://second.test/{path}'],
    ]) {
      const source = { id: '1', kind: 'external_link', label: 'mirror', delivery: { mode: 'file', url, authenticationRequired: false } }
      expect(mirrorTemplate(source, official)).toBe(expected)
      expect(mirrorUrl(expected, official)).toBe(url)
    }
  })
  it('rejects insecure mirrors and never treats an older site version as an update', () => {
    expect(() => mirrorUrl('http://mirror.test/', official)).toThrow()
    expect(() => mirrorUrl('https://user:secret@mirror.test/', official)).toThrow()
    expect(newer('0.8.4', '0.8.5')).toBe(false)
    expect(newer('0.8.5', '0.8.5')).toBe(false)
    expect(newer('0.10.0', '0.9.9')).toBe(true)
    expect(() => newer('broken', '0.8.5')).toThrow()
  })
  it('carries the selected official asset and mirror template to the dedicated native opener', async () => {
    const source = { id: '1', kind: 'external_link', label: 'mirror', delivery: { mode: 'file', url: `https://custom.test/${official}`, authenticationRequired: false } }
    const template = mirrorTemplate(source, official)!
    const url = mirrorUrl(template, official)
    const mirror = { template, officialUrl: official }
    await openUpdateDownload(url, mirror)
    expect(invoke).toHaveBeenLastCalledWith('update_open_download', { url, mirror })
    await openUpdateDownload('https://pan.quark.cn/s/test')
    expect(invoke).toHaveBeenLastCalledWith('update_open_download', { url: 'https://pan.quark.cn/s/test', mirror: undefined })
  })
})
