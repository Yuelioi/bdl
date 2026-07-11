import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { FEEDBACK_AUTO_DISMISS_MS } from './feedback'
import { useUiStore } from './ui'

describe('global feedback policy', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  afterEach(() => vi.useRealTimers())

  it('dismisses ordinary messages after the shared five-second delay', async () => {
    const ui = useUiStore()
    ui.pushToast('保存成功', 'success')

    await vi.advanceTimersByTimeAsync(FEEDBACK_AUTO_DISMISS_MS - 1)
    expect(ui.toasts).toHaveLength(1)
    await vi.advanceTimersByTimeAsync(1)
    expect(ui.toasts).toHaveLength(0)
  })

  it('keeps error messages until the user closes them', async () => {
    const ui = useUiStore()
    ui.pushToast('下载失败', 'danger')

    await vi.advanceTimersByTimeAsync(FEEDBACK_AUTO_DISMISS_MS * 2)
    expect(ui.toasts).toHaveLength(1)
  })
})
