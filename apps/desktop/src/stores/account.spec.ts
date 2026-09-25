import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { accountImportCookie, accountLoginQrPoll, accountVerify } from '../api/tauri'
import { useAccountStore } from './account'

vi.mock('../api/tauri', () => ({
  accountImportCookie: vi.fn(),
  accountLoginQrPoll: vi.fn(),
  accountVerify: vi.fn(),
}))

const imported = { logged_in: true, mid: '42', name: null, avatar_url: null, vip_label: null }
const verified = { ...imported, name: '测试用户', avatar_url: 'https://i0.hdslb.com/bfs/face/test.jpg' }

describe('account profile after login', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(accountImportCookie).mockResolvedValue(imported)
    vi.mocked(accountVerify).mockResolvedValue(verified)
  })

  it('loads nickname and avatar immediately after cookie login', async () => {
    const account = useAccountStore()
    expect(await account.importCookie('DedeUserID=42; SESSDATA=test')).toBe(true)
    expect(account.profile).toEqual(verified)
  })

  it('loads nickname and avatar immediately after QR confirmation', async () => {
    const account = useAccountStore()
    account.qrSession = { qrcode_key: 'test', qr_url: '', qr_image_svg: '', expires_in_seconds: 180 }
    vi.mocked(accountLoginQrPoll).mockResolvedValue({ status: 'confirmed', message: '', account: imported })
    await account.pollQrLogin()
    expect(account.profile).toEqual(verified)
    expect(account.qrSession).toBeNull()
  })

  it('keeps the saved login if fetching profile information fails', async () => {
    vi.mocked(accountVerify).mockRejectedValue(new Error('offline'))
    const account = useAccountStore()
    expect(await account.importCookie('DedeUserID=42; SESSDATA=test')).toBe(true)
    expect(account.profile).toEqual(imported)
  })
})
