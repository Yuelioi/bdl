import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import UiEmptyState from './EmptyState.vue'

describe('EmptyState', () => {
  it('renders accessible error content and its action', () => {
    const wrapper = mount(UiEmptyState, {
      props: {
        title: '读取失败',
        description: '暂时无法读取内容。',
        icon: 'i-tabler-cloud-off',
        tone: 'danger',
        role: 'alert',
      },
      slots: {
        action: '<button>重新加载</button>',
      },
    })

    expect(wrapper.attributes('role')).toBe('alert')
    expect(wrapper.classes()).toContain('tone-danger')
    expect(wrapper.get('h3').text()).toBe('读取失败')
    expect(wrapper.get('button').text()).toBe('重新加载')
  })
})
