import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { accountImportCookie, accountLoginQrPoll, accountLoginQrStart, accountVerify, accountLogout, accountLoginQrCancel, accountGet } from '../api/tauri'
import { useAccountStore } from './account'

vi.mock('../api/tauri', () => ({
  accountImportCookie: vi.fn(),
  accountLoginQrPoll: vi.fn(),
  accountLoginQrStart: vi.fn(),
  accountVerify: vi.fn(),
  accountLogout: vi.fn(),
  accountLoginQrCancel: vi.fn(),
  accountGet: vi.fn(),
}))

const imported = { logged_in: true, mid: '42', name: null, avatar_url: null, vip_label: null }
const verified = { ...imported, name: '测试用户', avatar_url: 'https://i0.hdslb.com/bfs/face/test.jpg' }

describe('account profile after login', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(accountImportCookie).mockResolvedValue(imported)
    vi.mocked(accountVerify).mockResolvedValue(verified)
    vi.mocked(accountLoginQrCancel).mockResolvedValue(undefined)
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

  it('ignores a profile verification that finishes after logout', async () => {
    let finish!: (value: typeof verified) => void
    vi.mocked(accountVerify).mockImplementation(() => new Promise(resolve => { finish = resolve }))
    const account = useAccountStore()
    account.profile = verified
    const verifying = account.verify()
    const loggedOut = { ...imported, logged_in: false, mid: null }
    vi.mocked(accountLogout).mockResolvedValue(loggedOut)
    await account.logout()
    finish(verified)
    await verifying
    expect(account.profile).toEqual(loggedOut)
  })

  it('discards QR confirmation after closing the session and cancels its exact request', async () => {
    let finish!: (value: { status: 'confirmed'; message: string; account: typeof imported }) => void
    vi.mocked(accountLoginQrPoll).mockImplementation(() => new Promise(resolve => { finish = resolve }))
    const account = useAccountStore()
    account.qrRequestToken = 'session-1'
    account.qrSession = { qrcode_key: 'old', qr_url: '', qr_image_svg: '', expires_in_seconds: 180 }
    const polling = account.pollQrLogin()
    account.resetQrLogin()
    finish({ status: 'confirmed', message: '', account: imported })
    await polling
    expect(accountLoginQrCancel).toHaveBeenCalledWith('session-1')
    expect(account.profile.logged_in).toBe(false)
    expect(accountVerify).not.toHaveBeenCalled()
  })

  it('ignores an older account snapshot after a newer read completes', async () => {
    let finish!: (value: typeof imported) => void
    vi.mocked(accountGet).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    const account = useAccountStore()
    const older = account.load()
    vi.mocked(accountGet).mockResolvedValue(verified)
    await account.load()
    finish(imported)
    await older
    expect(account.profile).toEqual(verified)
  })

  it('can poll a new QR while a previous request is still returning', async () => {
    let finish!: (value: { status: 'confirmed'; message: string; account: typeof imported }) => void
    vi.mocked(accountLoginQrPoll).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
      .mockResolvedValue({ status: 'waiting', message: '', account: null })
    const account = useAccountStore()
    account.qrSession = { qrcode_key: 'old', qr_url: '', qr_image_svg: '', expires_in_seconds: 180 }
    const old = account.pollQrLogin()
    vi.mocked(accountLoginQrStart).mockResolvedValue({ ...account.qrSession, qrcode_key: 'new' })
    await account.startQrLogin()
    expect(accountLoginQrPoll).toHaveBeenCalledWith({ qrcode_key: 'new' })
    finish({ status: 'confirmed', message: '', account: imported })
    await old
    expect(account.profile.logged_in).toBe(false)
    expect(account.qrSession?.qrcode_key).toBe('new')
    account.resetQrLogin()
  })
})
