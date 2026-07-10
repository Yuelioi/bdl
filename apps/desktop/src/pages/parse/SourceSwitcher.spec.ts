import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import type { NormalizedSourceTree } from '../../api/dto'
import SourceSwitcher from './SourceSwitcher.vue'

const source: NormalizedSourceTree = {
  source: { id: 'source-1', kind: 'video', input: 'BV1', title: '唯一来源', loaded_count: 0, total_count: 0, has_more: false },
  groups: [],
}

describe('SourceSwitcher', () => {
  it('renders a quiet status instead of a disabled dropdown with no sources', () => {
    const wrapper = mount(SourceSwitcher, {
      props: { sources: [], activeSourceId: null, selectionBySource: {} },
      global: { stubs: { UIcon: true, UDropdownMenu: true } },
    })
    expect(wrapper.text()).toContain('等待解析')
    expect(wrapper.find('button').exists()).toBe(false)
  })

  it('renders one source as static context rather than a dropdown', () => {
    const wrapper = mount(SourceSwitcher, {
      props: { sources: [source], activeSourceId: 'source-1', selectionBySource: {} },
      global: { stubs: { UIcon: true, UDropdownMenu: true } },
    })
    expect(wrapper.text()).toContain('已解析 1 个来源')
    expect(wrapper.find('button').exists()).toBe(false)
  })
})
