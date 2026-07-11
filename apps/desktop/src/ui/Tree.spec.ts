import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'

import UiTree from './Tree.vue'

const nodes = [
  {
    id: 'collection',
    label: '合集',
    children: [
      { id: 'part-1', label: '第一集', partIds: ['part-1'] },
      { id: 'part-2', label: '第二集', partIds: ['part-2'] },
    ],
  },
  { id: 'single', label: '单集', partIds: ['single'] },
]

afterEach(() => {
  document.body.innerHTML = ''
})

describe('Tree keyboard navigation', () => {
  it('keeps one tab stop and moves focus with vertical navigation keys', async () => {
    const wrapper = mount(UiTree, { attachTo: document.body, props: { nodes } })
    const rows = wrapper.findAll('[role="treeitem"]')

    expect(rows.map((row) => row.attributes('tabindex'))).toEqual(['0', '-1', '-1', '-1'])

    await rows[0].trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(rows[1].element)
    expect(rows[1].attributes('tabindex')).toBe('0')

    await rows[1].trigger('keydown', { key: 'End' })
    expect(document.activeElement).toBe(rows[3].element)

    await rows[3].trigger('keydown', { key: 'Home' })
    expect(document.activeElement).toBe(rows[0].element)
  })

  it('moves between an expanded parent and its children with horizontal arrows', async () => {
    const wrapper = mount(UiTree, { attachTo: document.body, props: { nodes } })
    const rows = wrapper.findAll('[role="treeitem"]')

    await rows[0].trigger('keydown', { key: 'ArrowRight' })
    expect(document.activeElement).toBe(rows[1].element)

    await rows[1].trigger('keydown', { key: 'ArrowLeft' })
    expect(document.activeElement).toBe(rows[0].element)
  })

  it('toggles the focused node with Space', async () => {
    const wrapper = mount(UiTree, { props: { nodes } })
    const firstRow = wrapper.find('[role="treeitem"]')

    await firstRow.trigger('keydown', { key: ' ' })

    expect(wrapper.emitted('toggle')).toEqual([['collection']])
  })

  it('moves focus across a virtualized large tree', async () => {
    const largeNodes = Array.from({ length: 250 }, (_, index) => ({
      id: `part-${index}`,
      label: `第 ${index + 1} 集`,
      partIds: [`part-${index}`],
    }))
    const wrapper = mount(UiTree, { attachTo: document.body, props: { nodes: largeNodes } })

    expect(wrapper.findAll('[role="treeitem"]').length).toBeLessThan(largeNodes.length)
    await wrapper.find('[role="treeitem"]').trigger('keydown', { key: 'End' })

    const lastRow = wrapper.find('[data-tree-index="249"]')
    expect(lastRow.exists()).toBe(true)
    expect(document.activeElement).toBe(lastRow.element)
  })
})
