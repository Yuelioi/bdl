import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import UiCheckbox from './Checkbox.vue'

describe('UiCheckbox', () => {
  it('keeps the accessible label while visually compacting table selection', () => {
    const wrapper = mount(UiCheckbox, {
      props: { label: '选择任务', compact: true },
      global: { stubs: { UCheckbox: { template: '<label><input type="checkbox"><span>{{ label }}</span></label>', props: ['label'] } } },
    })

    expect(wrapper.classes()).toContain('compact')
    expect(wrapper.text()).toContain('选择任务')
  })
})
