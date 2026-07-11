import { mount } from '@vue/test-utils'
import { defineComponent } from 'vue'
import { describe, expect, it } from 'vitest'

import UiDialog from './Dialog.vue'

const ModalStub = defineComponent({
  name: 'UModal',
  props: { open: Boolean, title: String },
  emits: ['update:open'],
  template: `
    <div v-if="open" role="dialog" aria-modal="true" :aria-label="title">
      <slot name="body" />
      <slot name="footer" />
      <button type="button" @click="$emit('update:open', false)">关闭</button>
    </div>
  `,
})

describe('Dialog', () => {
  it('exposes modal semantics and propagates close state', async () => {
    const wrapper = mount(UiDialog, {
      props: { modelValue: true, title: '任务详情' },
      slots: { default: '诊断内容', footer: '<button>重试</button>' },
      global: { components: { UModal: ModalStub } },
    })

    const dialog = wrapper.get('[role="dialog"]')
    expect(dialog.attributes('aria-modal')).toBe('true')
    expect(dialog.attributes('aria-label')).toBe('任务详情')
    expect(dialog.text()).toContain('诊断内容')

    await wrapper.get('button[type="button"]').trigger('click')
    expect(wrapper.emitted('update:modelValue')).toEqual([[false]])
  })
})
