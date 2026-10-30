import { invoke, isTauri } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { readClipboardText, writeClipboardText } from './clipboard'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => true),
}))

describe('readClipboardText', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset()
  })

  it('reads Android clipboard through the native command instead of the WebView clipboard API', async () => {
    vi.mocked(invoke).mockResolvedValue('https://www.bilibili.com/video/BV1test')

    await expect(readClipboardText(true)).resolves.toBe('https://www.bilibili.com/video/BV1test')
    expect(invoke).toHaveBeenCalledWith('mobile_read_clipboard_text')
    const diagnostic = '详细原因🙂'.repeat(1000) + ' final-cause-marker'
    await writeClipboardText(diagnostic, true)
    expect(invoke).toHaveBeenLastCalledWith('mobile_write_clipboard_text', { text: diagnostic })
  })
})

it('reads the desktop native clipboard without browser permissions', async () => {
  vi.mocked(isTauri).mockReturnValue(true)
  vi.mocked(invoke).mockResolvedValue('BV1xx411c7mD')
  await expect(readClipboardText(false)).resolves.toBe('BV1xx411c7mD')
  expect(invoke).toHaveBeenCalledWith('desktop_read_clipboard_text')
  await writeClipboardText('diagnostic final cause', false)
  expect(invoke).toHaveBeenLastCalledWith('desktop_write_clipboard_text', { text: 'diagnostic final cause' })
})
