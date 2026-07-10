import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import UiTextField from './TextField.vue'

describe('TextField validation', () => {
  it('associates an inline error with the native input', () => {
    const wrapper = mount(UiTextField, {
      props: {
        label: '开始时间',
        type: 'datetime-local',
        error: '开始时间必须晚于当前时间',
      },
    })
    const input = wrapper.get('input')
    const messageId = input.attributes('aria-describedby')

    expect(input.attributes('aria-invalid')).toBe('true')
    expect(wrapper.get(`#${messageId}`).text()).toBe('开始时间必须晚于当前时间')
    expect(wrapper.get(`#${messageId}`).attributes('aria-live')).toBe('polite')
  })
})
