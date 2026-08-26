import { describe, expect, it } from 'vitest'

import type { DownloadTask, NormalizedItem } from '../api/dto'
import { bilibiliFavoriteCategoryUrl, bilibiliTaskUrl, bilibiliUserUrl, bilibiliVideoUrl } from './bilibiliLinks'

describe('Bilibili page links', () => {
  it('builds safe user profile links only from numeric MID values', () => {
    expect(bilibiliUserUrl('4279370')).toBe('https://space.bilibili.com/4279370')
    expect(bilibiliUserUrl('javascript:alert(1)')).toBeNull()
  })

  it('builds one account-level link for each favorite category', () => {
    expect(bilibiliFavoriteCategoryUrl('4279370', 'created_favorite')).toBe(
      'https://space.bilibili.com/4279370/favlist?ftype=create',
    )
    expect(bilibiliFavoriteCategoryUrl('4279370', 'collected_favorite')).toBe(
      'https://space.bilibili.com/4279370/favlist?ftype=collect',
    )
    expect(bilibiliFavoriteCategoryUrl('javascript:alert(1)', 'created_favorite')).toBeNull()
  })

  it('prefers a BV page for normalized video items', () => {
    const item = { parts: [{ bvid: 'BV1xx411c7mD', aid: 170001 }] } as NormalizedItem
    expect(bilibiliVideoUrl(item)).toBe('https://www.bilibili.com/video/BV1xx411c7mD')
  })

  it('uses refresh identity for completed task pages', () => {
    const task = { refresh_intent: { input: { kind: 'video_bvid', bvid: 'BV1xx411c7mD' }, cid: 1 } } as DownloadTask
    expect(bilibiliTaskUrl(task)).toBe('https://www.bilibili.com/video/BV1xx411c7mD')
  })

  it('preserves the selected page for multi-part video tasks', () => {
    const task = {
      refresh_intent: { input: { kind: 'video_bvid', bvid: 'BV1xx411c7mD' }, cid: 2, page_number: 3 },
    } as DownloadTask
    expect(bilibiliTaskUrl(task)).toBe('https://www.bilibili.com/video/BV1xx411c7mD?p=3')
  })
})
