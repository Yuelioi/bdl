import { shallowMount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import WorkflowSteps from './WorkflowSteps.vue'

describe('WorkflowSteps', () => {
  it('moves to an available workflow step', async () => {
    const wrapper = shallowMount(WorkflowSteps, {
      props: {
        modelValue: 'source',
        steps: [
          { value: 'source', label: '解析来源' },
          { value: 'content', label: '选择内容' },
        ],
        'onUpdate:modelValue': (value: string) => wrapper.setProps({ modelValue: value }),
      },
      global: { stubs: { UIcon: true } },
    })

    await wrapper.findAll('button')[1].trigger('click')

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual(['content'])
  })
})
