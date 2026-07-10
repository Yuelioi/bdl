import { shallowMount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import type { TransferTaskView } from '../stores/transferView'
import TransferTaskTable from './TransferTaskTable.vue'

const completedView = (): TransferTaskView => ({
  id: 'task-1',
  isCompleted: true,
  displayTitle: '示例视频',
  subtitle: '',
  statusLabel: '已完成',
  statusBadge: 'done',
  progressValue: 100,
  progressLabel: '100%',
  speedLabel: '--',
  etaLabel: '--',
  sizeLabel: '20 MB / 20 MB',
  issueLabel: '-',
  shortLocation: 'downloads',
  fullLocation: 'E:\\downloads',
  outputPath: 'E:\\downloads\\video.mp4',
  sourceId: 'source-1',
  primaryAction: 'open_file',
  primaryActionLabel: '打开文件',
  primaryActionIcon: 'file',
  secondaryActions: [],
})

describe('TransferTaskTable', () => {
  it('shows result columns instead of transfer metrics for completed tasks', () => {
    const wrapper = shallowMount(TransferTaskTable, {
      props: {
        views: [completedView()],
        selectedTaskId: null,
        selectedTaskIds: [],
        mode: 'completed',
      },
      global: { stubs: { UIcon: true } },
    })

    expect(wrapper.text()).toContain('输出位置')
    expect(wrapper.text()).not.toContain('剩余')
  })
})
