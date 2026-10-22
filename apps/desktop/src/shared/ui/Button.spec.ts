import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import UiButton from './Button.vue'

describe('Button', () => {
  it('renders its action label', () => {
    const wrapper = mount(UiButton, {
      props: {
        variant: 'danger',
        size: 'compact',
      },
      slots: {
        default: '移除',
      },
    })

    expect(wrapper.text()).toBe('移除')
  })
})
