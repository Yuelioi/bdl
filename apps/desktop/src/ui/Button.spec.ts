import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import UiButton from './Button.vue'

describe('Button', () => {
  it('combines compact density with semantic danger styling', () => {
    const wrapper = mount(UiButton, {
      props: {
        variant: 'danger',
        size: 'compact',
      },
      slots: {
        default: '移除',
      },
    })

    expect(wrapper.classes()).toContain('size-compact')
    expect(wrapper.classes()).toContain('variant-danger')
    expect(wrapper.text()).toBe('移除')
  })
})
