import { afterEach, describe, expect, it, vi } from 'vitest'
import { diagnosticsEnvironment } from '../api/tauri'
import { captureDiagnosticEnvironment } from './diagnosticEnvironment'

vi.mock('../api/tauri', () => ({ diagnosticsEnvironment: vi.fn() }))
afterEach(() => { vi.resetAllMocks(); vi.unstubAllGlobals() })

describe('copied diagnostic environment', () => {
  it('uses native Android version and only includes safe download configuration', async () => {
    vi.stubGlobal('navigator', { userAgent: 'Mozilla/5.0 (Linux; Android 10) Chrome/130' })
    vi.mocked(diagnosticsEnvironment).mockResolvedValue({
      app_version: '0.8.4', os: 'android', arch: 'aarch64', device_info_available: true,
      device: { os_version: '16', android_sdk: 36, webview_version: '130.0' },
      segment_count: 4, concurrent_tasks: 2, retry_count: 5, proxy_mode: 'custom',
    })
    const text = await captureDiagnosticEnvironment()
    expect(text).toContain('系统: android 16 · aarch64')
    expect(text).toContain('Android SDK: 36')
    expect(text).toContain('WebView: 130.0')
    expect(text).toContain('分段 4 · 并发 2 · 重试 5 · 下载代理: 手动设置代理')
    expect(text).toContain('BDL: 0.8.4')
  })

  it('still allows copying logs when native environment collection fails', async () => {
    vi.mocked(diagnosticsEnvironment).mockRejectedValue(new Error('fixture failure with private details'))
    const text = await captureDiagnosticEnvironment()
    expect(text).toContain('未能获取原生信息')
    expect(text).not.toContain('private details')
  })
})
