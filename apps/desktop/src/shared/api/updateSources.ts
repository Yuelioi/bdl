import { invoke } from '@tauri-apps/api/core'
import { Update } from '@tauri-apps/plugin-updater'

export interface DownloadSource {
  id: string
  kind: string
  label: string
  delivery: { mode: string; url?: string; authenticationRequired: boolean }
}
export interface DistributionUpdate {
  release: { version: string; fileName: string }
  releaseNotes: string
  sources: DownloadSource[]
}

export function newer(version: string, current: string): boolean {
  const parse = (v: string) => {
    if (!/^\d+\.\d+\.\d+$/.test(v)) throw new Error('更新版本号无效。')
    return v.split('.').map(Number)
  }
  const a = parse(version), b = parse(current)
  const i = a.findIndex((part, index) => part !== b[index])
  return i >= 0 && a[i] > b[i]
}

export async function distributionUpdate(): Promise<DistributionUpdate> {
  const result = await invoke<DistributionUpdate>('update_distribution')
  newer(result.release.version, '0.0.0')
  if (!Array.isArray(result.sources)) throw new Error('下载来源响应无效。')
  result.sources = result.sources.filter(s => {
    try { return !s.delivery.authenticationRequired && new URL(s.delivery.url ?? '').protocol === 'https:' }
    catch { return false }
  })
  return result
}

export function mirrorTemplate(source: DownloadSource, official: string): string | null {
  const url = source.delivery.url ?? ''
  if (source.kind !== 'external_link' || source.delivery.mode !== 'file') return null
  if (url.endsWith(official)) return url.slice(0, -official.length) + '{url}'
  const path = official.replace(/^https:\/\//, '')
  if (url.endsWith(path)) return url.slice(0, -path.length) + '{path}'
  return null
}

export function mirrorUrl(template: string, official: string): string {
  const value = template.includes('{url}') ? template.replace('{url}', official)
    : template.includes('{path}') ? template.replace('{path}', official.replace(/^https:\/\//, ''))
      : `${template.replace(/\/+$/, '')}/${official}`
  const url = new URL(value)
  if (url.protocol !== 'https:' || url.username || url.password || url.hash) throw new Error('请填写 HTTPS 加速地址。')
  return url.href
}

export async function checkUpdate(template?: string): Promise<Update | null> {
  const metadata = await invoke<ConstructorParameters<typeof Update>[0] | null>('update_check', { template })
  return metadata ? new Update(metadata) : null
}

export async function openUpdateDownload(url: string, mirror?: { template: string; officialUrl: string }): Promise<void> {
  await invoke('update_open_download', { url, mirror })
}
