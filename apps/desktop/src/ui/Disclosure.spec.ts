import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import UiDisclosure from './Disclosure.vue'

describe('UiDisclosure', () => {
  it('renders its summary and hidden content through a native details element', () => {
    const wrapper = mount(UiDisclosure, {
      props: { title: '更多选项', description: '编码与附加内容', variant: 'panel' },
      slots: { default: '<p>高级设置</p>' },
      global: { stubs: { UIcon: true } },
    })

    expect(wrapper.get('details').classes()).toContain('variant-panel')
    expect(wrapper.get('summary').text()).toContain('更多选项')
    expect(wrapper.text()).toContain('高级设置')
  })
})
