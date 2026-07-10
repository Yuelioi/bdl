import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, expect, it } from 'vitest'
import { defineComponent, h, type PropType } from 'vue'

import { useThemeStore } from '../stores/theme'
import AppearanceMenu from './AppearanceMenu.vue'

describe('AppearanceMenu', () => {
  it('exposes the current preference as a checked menu item', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    useThemeStore().setPreference('dark')
    const wrapper = mount(AppearanceMenu, {
      global: {
        plugins: [pinia],
        stubs: {
          UDropdownMenu: defineComponent({
            props: {
              items: {
                type: Array as PropType<Array<Array<{ label: string; type: string; checked: boolean }>>>,
                required: true,
              },
            },
            setup(props, { slots }) {
              return () => h('div', [
                slots.default?.(),
                ...props.items.flat().map((item) => h('div', {
                  role: item.type === 'checkbox' ? 'menuitemcheckbox' : 'menuitem',
                  'aria-checked': String(item.checked),
                }, item.label)),
              ])
            },
          }),
        },
      },
    })

    expect(wrapper.get('[role="menuitemcheckbox"][aria-checked="true"]').text()).toBe('深色')
  })
})
