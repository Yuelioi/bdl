import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { accountLibraryList, parseCreateSource, queueResume } from './tauri'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

describe('user-facing BPI failures', () => {
  beforeEach(() => vi.resetAllMocks())
  const raw = 'bpi error: failed to decode response (Data) at line 1 column 15164'
  it.each([
    ['parse', () => parseCreateSource({ input: 'BV1xx411c7mD' }), '解析失败'],
    ['library', () => accountLibraryList({ kind: 'created_favorite', page: 1, page_size: 20 }), '收藏夹加载失败'],
    ['download', () => queueResume('task:1'), '下载操作失败'],
  ] as const)('hides backend details for %s', async (_name, operation, message) => {
    vi.mocked(invoke).mockRejectedValue({ code: 'core_error', message: raw })
    await expect(operation()).rejects.toThrow(message)
    await expect(operation()).rejects.not.toThrow('column')
  })
  it.each([raw, new Error(raw)])('handles legacy unstructured failures', async (error) => {
    vi.mocked(invoke).mockRejectedValue(error)
    await expect(parseCreateSource({ input: 'fixture' })).rejects.toThrow('解析失败')
  })
  it.each(['bilibili_response_decode_failed', 'bilibili_api_error'])('handles the backend error code %s', async (code) => {
    vi.mocked(invoke).mockRejectedValue({ code, message: '后端通用提示' })
    await expect(parseCreateSource({ input: 'fixture' })).rejects.toThrow('解析失败')
    await expect(accountLibraryList({ kind: 'created_favorite', page: 1, page_size: 20 })).rejects.toThrow('收藏夹加载失败')
  })
  it('preserves actionable login errors', async () => {
    vi.mocked(invoke).mockRejectedValue({ code: 'login_required', message: '登录状态可能已失效，请重新登录后重试。' })
    await expect(parseCreateSource({ input: 'fixture' })).rejects.toMatchObject({ code: 'login_required', message: '登录状态可能已失效，请重新登录后重试。' })
  })
})
