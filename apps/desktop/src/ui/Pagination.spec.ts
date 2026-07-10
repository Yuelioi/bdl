import { mount } from '@vue/test-utils'
import { defineComponent } from 'vue'
import { describe, expect, it } from 'vitest'

import UiPagination from './Pagination.vue'

const PaginationStub = defineComponent({
  name: 'UPagination',
  props: {
    page: Number,
    total: Number,
    itemsPerPage: Number,
    disabled: Boolean,
  },
  emits: ['update:page'],
  template: '<button type="button" @click="$emit(\'update:page\', 3)">第 3 页</button>',
})

describe('Pagination', () => {
  it('forwards pagination state and page changes through one shared component', async () => {
    const wrapper = mount(UiPagination, {
      props: {
        page: 2,
        total: 200,
        itemsPerPage: 20,
        label: '视频分页',
      },
      global: {
        components: { UPagination: PaginationStub },
      },
    })

    expect(wrapper.get('nav').attributes('aria-label')).toBe('视频分页')
    expect(wrapper.getComponent(PaginationStub).props()).toMatchObject({
      page: 2,
      total: 200,
      itemsPerPage: 20,
    })

    await wrapper.get('button').trigger('click')

    expect(wrapper.emitted('update:page')).toEqual([[3]])
  })
})
