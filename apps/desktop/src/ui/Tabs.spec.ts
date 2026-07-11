import { mount } from '@vue/test-utils'
import { defineComponent } from 'vue'
import { describe, expect, it } from 'vitest'

import UiTabs from './Tabs.vue'

const TabsStub = defineComponent({
  name: 'UTabs',
  props: { modelValue: [String, Number], items: Array },
  emits: ['update:modelValue'],
  template: `
    <div role="tablist">
      <button
        v-for="item in items"
        :key="item.value"
        role="tab"
        :aria-selected="item.value === modelValue"
        @click="$emit('update:modelValue', item.value)"
      >{{ item.label }}</button>
    </div>
  `,
})

describe('Tabs', () => {
  it('forwards active state, counts, and changes through the shared wrapper', async () => {
    const wrapper = mount(UiTabs, {
      props: {
        modelValue: 'active',
        tabs: [
          { label: '活动', value: 'active', count: 2 },
          { label: '完成', value: 'completed', count: 8 },
        ],
        'onUpdate:modelValue': (value: string) => wrapper.setProps({ modelValue: value }),
      },
      global: { components: { UTabs: TabsStub } },
    })

    expect(wrapper.get('[aria-selected="true"]').text()).toBe('活动')
    expect(wrapper.getComponent(TabsStub).props('items')).toEqual([
      { label: '活动', value: 'active', badge: 2 },
      { label: '完成', value: 'completed', badge: 8 },
    ])

    await wrapper.findAll('[role="tab"]')[1].trigger('click')
    expect(wrapper.emitted('update:modelValue')).toEqual([['completed']])
  })
})
