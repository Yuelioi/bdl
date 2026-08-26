import type { AccountLibraryFolderKind, DownloadTask, NormalizedItem } from '../api/dto'

export const bilibiliUserUrl = (mid: string | number | null | undefined): string | null => {
  const value = String(mid ?? '').trim()
  return /^\d+$/.test(value) ? `https://space.bilibili.com/${value}` : null
}

export const bilibiliFavoriteCategoryUrl = (
  mid: string | number | null | undefined,
  kind: AccountLibraryFolderKind,
): string | null => {
  const profileUrl = bilibiliUserUrl(mid)
  if (!profileUrl) return null
  return `${profileUrl}/favlist?ftype=${kind === 'created_favorite' ? 'create' : 'collect'}`
}

export const bilibiliVideoUrl = (item: NormalizedItem): string | null => {
  const part = item.parts.find((candidate) => candidate.bvid || candidate.aid)
  if (part?.bvid) return `https://www.bilibili.com/video/${part.bvid}`
  if (part?.aid) return `https://www.bilibili.com/video/av${part.aid}`
  return null
}

export const bilibiliTaskUrl = (task: DownloadTask): string | null => {
  const input = task.refresh_intent?.input
  const page = task.refresh_intent?.page_number
  const pageQuery = page && page > 1 ? `?p=${page}` : ''
  if (input?.kind === 'video_bvid') return `https://www.bilibili.com/video/${input.bvid}${pageQuery}`
  if (input?.kind === 'video_aid') return `https://www.bilibili.com/video/av${input.aid}${pageQuery}`
  if (input?.kind === 'bangumi_episode') return `https://www.bilibili.com/bangumi/play/ep${input.ep_id}`
  if (input?.kind === 'cheese_episode') return `https://www.bilibili.com/cheese/play/ep${input.ep_id}`

  const uploader = task.source_id.match(/^uploader:(\d+):videos$/)
  if (uploader) return `https://space.bilibili.com/${uploader[1]}/video`
  return null
}
