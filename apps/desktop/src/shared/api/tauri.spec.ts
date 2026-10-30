import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { accountLibraryList, parseCreateSource, queueResume, diagnosticsExport } from './tauri'
import { clearRecentFailures, recentFailureReport } from '../utils/recentFailures'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

describe('user-facing BPI failures', () => {
  it('keeps bounded redacted failures at the command boundary without request secrets', async () => {
    clearRecentFailures()
    vi.mocked(invoke).mockRejectedValue({ code: 'bilibili_api_error', message: '失败', detail: 'HTTP 429; SESSDATA=private-cookie' })
    for (let index = 0; index < 21; index++) {
      await parseCreateSource({ input: `private-input-${index}` }, 'request-fixture').catch(() => undefined)
    }
    expect(recentFailureReport.value.match(/操作: parse_create_source/g)).toHaveLength(20)
    expect(recentFailureReport.value).toContain('HTTP 429')
    expect(recentFailureReport.value).not.toContain('private-cookie')
    expect(recentFailureReport.value).not.toContain('private-input')
    clearRecentFailures()
    expect(recentFailureReport.value).toBe('')
  })
  beforeEach(() => vi.resetAllMocks())
  const raw = 'bpi error: failed to decode response (Data) at line 1 column 15164'
  it.each([
    ['parse', () => parseCreateSource({ input: 'BV1xx411c7mD' }, 'request-fixture'), '解析失败'],
    ['library', () => accountLibraryList({ kind: 'created_favorite', page: 1, page_size: 20 }), '收藏夹加载失败'],
    ['download', () => queueResume('task:1'), '下载操作失败'],
  ] as const)('hides backend details for %s', async (_name, operation, message) => {
    vi.mocked(invoke).mockRejectedValue({ code: 'core_error', message: raw })
    await expect(operation()).rejects.toThrow(message)
    await expect(operation()).rejects.not.toThrow('column')
  })
  it.each([raw, new Error(raw)])('handles legacy unstructured failures', async (error) => {
    vi.mocked(invoke).mockRejectedValue(error)
    await expect(parseCreateSource({ input: 'fixture' }, 'request-fixture')).rejects.toThrow('解析失败')
  })
  it.each(['bilibili_response_decode_failed', 'bilibili_api_error'])('handles the backend error code %s', async (code) => {
    vi.mocked(invoke).mockRejectedValue({ code, message: '后端通用提示' })
    await expect(parseCreateSource({ input: 'fixture' }, 'request-fixture')).rejects.toThrow('解析失败')
    await expect(accountLibraryList({ kind: 'created_favorite', page: 1, page_size: 20 })).rejects.toThrow('收藏夹加载失败')
  })
  it('preserves structured causes even when a legacy error omits code', async () => {
    vi.mocked(invoke).mockRejectedValue({ message: '连接中断', detail: 'socket reset by peer' })
    const error = await parseCreateSource({ input: 'fixture' }, 'request-fixture').catch(error => error)
    expect(error.message).toBe('连接中断')
    expect(error.detail).toContain('socket reset by peer')
  })
  it('includes failures before task creation in exported diagnostics', async () => {
    clearRecentFailures()
    vi.mocked(invoke).mockRejectedValueOnce({ code: 'bilibili_api_error', message: '操作失败', detail: 'HTTP 412; token=hidden-export-token' })
    await parseCreateSource({ input: 'fixture' }, 'request-fixture').catch(() => undefined)
    vi.mocked(invoke).mockResolvedValueOnce({ path: 'diagnostics.json', saved: true })
    await diagnosticsExport()
    const stored = vi.mocked(invoke).mock.calls.find(([command]) => command === 'diagnostics_record_operation_error')?.[1]
    expect(JSON.stringify(stored)).toContain('HTTP 412')
    expect(JSON.stringify(stored)).not.toContain('hidden-export-token')
    expect(invoke).toHaveBeenLastCalledWith('diagnostics_export', { taskId: null })
    clearRecentFailures()
  })
  it('preserves actionable login errors', async () => {
    vi.mocked(invoke).mockRejectedValue({ code: 'login_required', message: '登录状态可能已失效，请重新登录后重试。' })
    await expect(parseCreateSource({ input: 'fixture' }, 'request-fixture')).rejects.toMatchObject({ code: 'login_required', message: '登录状态可能已失效，请重新登录后重试。' })
  })
  it('keeps diagnostic causes separate from the user-facing message', async () => {
    vi.mocked(invoke).mockRejectedValue({ code: 'bilibili_api_error', message: '操作失败', detail: 'API错误 [-400]: fixture cause' })
    const error = await parseCreateSource({ input: 'fixture' }, 'request-fixture').catch(error => error)
    expect(error.detail).toContain('API错误 [-400]: fixture cause')
    await expect(parseCreateSource({ input: 'fixture' }, 'request-fixture')).rejects.not.toThrow('fixture cause')
  })
  it.each([
    ['course_preview_only', '当前账号只能试看此分集，请登录已购买该课程的账号后重试。'],
    ['drm_protected', '该分集使用 DRM 加密，BDL 暂不支持下载。请前往哔哩哔哩观看。'],
  ])('preserves actionable course error %s', async (code, message) => {
    vi.mocked(invoke).mockRejectedValue({ code, message })
    await expect(parseCreateSource({ input: 'https://www.bilibili.com/cheese/play/ep644827' }, 'request-fixture'))
      .rejects.toMatchObject({ code, message })
  })
})
