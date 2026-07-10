import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import UiIconButton from './IconButton.vue'

describe('IconButton', () => {
  it('keeps its accessible label in compact mode', () => {
    const wrapper = mount(UiIconButton, {
      props: {
        icon: 'refresh',
        label: '刷新内容',
        size: 'compact',
      },
    })

    expect(wrapper.attributes('aria-label')).toBe('刷新内容')
    expect(wrapper.classes()).toContain('size-compact')
  })
})
