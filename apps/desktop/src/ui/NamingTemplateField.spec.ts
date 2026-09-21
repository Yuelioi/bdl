import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import NamingTemplateField from './NamingTemplateField.vue'

describe('NamingTemplateField', () => {
  it('inserts a selected magic variable at the current cursor', async () => {
    const wrapper = mount(NamingTemplateField, {
      props: { modelValue: '{title}.mp4' },
      global: {
        stubs: {
          UPopover: { template: '<div><slot /><slot name="content" /></div>' },
          UButton: true,
        },
      },
    })
    const input = wrapper.get('input').element as HTMLInputElement
    input.focus()
    input.setSelectionRange(7, 7)

    await wrapper.get('[data-variable="publish_date"]').trigger('click')

    expect(wrapper.emitted('update:modelValue')?.at(-1)).toEqual(['{title}{publish_date}.mp4'])
    expect(input.selectionStart).toBe(21)
  })
})
