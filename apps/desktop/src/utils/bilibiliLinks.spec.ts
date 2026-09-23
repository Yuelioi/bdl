import { describe, expect, it } from 'vitest'

import type { DownloadTask, NormalizedItem } from '../api/dto'
import {
  bilibiliFavoriteCategoryUrl,
  bilibiliTaskUrl,
  bilibiliUserUrl,
  bilibiliVideoUrl,
  extractBilibiliInputs,
  isSingleVideoInput,
} from './bilibiliLinks'

describe('Bilibili page links', () => {
  it('accepts only video identities and short links in single-video mode', () => {
    for (const input of ['BV1xx411c7mD', 'av170001', 'https://www.bilibili.com/video/BV1xx411c7mD/?p=2', 'https://b23.tv/abc']) {
      expect(isSingleVideoInput(input)).toBe(true)
    }
    for (const input of ['https://space.bilibili.com/42/favlist?fid=7', 'https://space.bilibili.com/42/lists/7?type=season', 'https://www.bilibili.com/bangumi/play/ss1', 'BV1xx411c7mD BV1xx411c7mE', 'https://example.com/video/BV1xx411c7mD']) {
      expect(isSingleVideoInput(input)).toBe(false)
    }
  })
  it('extracts links and ids from pasted share text while ignoring titles', () => {
    expect(
      extractBilibiliInputs(
        '【视频标题】 https://www.bilibili.com/video/BV1xx411c7mD/?spm_id_from=333.1007 复制打开\n' +
          '另一个标题\nhttps://b23.tv/abc123。\nBV1xx411c7mD\nav170001',
      ),
    ).toEqual([
      'https://www.bilibili.com/video/BV1xx411c7mD/?spm_id_from=333.1007',
      'https://b23.tv/abc123',
      'BV1xx411c7mD',
      'av170001',
    ])
  })

  it('does not extract nested Bilibili-looking values from unrelated URLs', () => {
    expect(
      extractBilibiliInputs(
        '标题 https://example.com/watch?next=https://www.bilibili.com/video/BV1xx411c7mD&aid=av170001',
      ),
    ).toEqual([])
  })

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
