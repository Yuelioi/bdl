import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import type { TransferTaskView } from '../stores/transferView'
import TransferTaskTable from './TransferTaskTable.vue'

const view = (index: number): TransferTaskView => ({
  id: `task:${index}`,
  isCompleted: false,
  displayTitle: `任务 ${index}`,
  subtitle: '',
  statusLabel: '等待中',
  statusBadge: 'queued',
  progressValue: 0,
  progressLabel: '0%',
  speedLabel: '--',
  etaLabel: '--',
  sizeLabel: '--',
  issueLabel: '-',
  shortLocation: 'downloads',
  fullLocation: 'downloads',
  outputPath: 'downloads',
  sourceId: `video:${index}`,
  primaryAction: 'cancel',
  primaryActionLabel: '取消',
  primaryActionIcon: 'x',
  secondaryActions: [],
})

describe('TransferTaskTable virtualization', () => {
  it('keeps selection scope complete while rendering a bounded large-list window', () => {
    const views = Array.from({ length: 250 }, (_, index) => view(index))
    const wrapper = mount(TransferTaskTable, {
      props: { views, selectedTaskId: null, selectedTaskIds: [] },
      global: {
        stubs: {
          UiCheckbox: true,
          UiIconButton: true,
          UiProgressBar: true,
          UiStatusBadge: true,
          TaskActionMenu: true,
          UIcon: true,
        },
      },
    })

    expect(wrapper.attributes('aria-rowcount')).toBe('251')
    expect(wrapper.findAll('.task-table-row').length).toBeLessThan(views.length)
    expect(wrapper.find('.virtual-task-list').attributes('style')).toContain('15500px')
  })
})
