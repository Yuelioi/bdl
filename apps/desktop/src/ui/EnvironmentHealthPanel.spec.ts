import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import type { EnvironmentHealthSnapshot } from '../api/dto'
import EnvironmentHealthPanel from './EnvironmentHealthPanel.vue'

const global = {
  stubs: {
    UButton: {
      props: ['disabled'],
      template: '<button type="button" :disabled="disabled"><slot /></button>',
    },
    UIcon: true,
  },
}

const unhealthyEnvironment: EnvironmentHealthSnapshot = {
  ready: false,
  download_directory: {
    status: 'missing',
    path: 'C:\\downloads',
    message: '保存目录尚未创建。',
  },
  ffmpeg: {
    status: 'invalid',
    source: 'configured',
    path: 'C:\\tools\\ffmpeg.exe',
    version: null,
    message: 'FFmpeg 配置无效。',
  },
}

describe('EnvironmentHealthPanel', () => {
  it('announces checking state and marks the region busy', () => {
    const wrapper = mount(EnvironmentHealthPanel, {
      props: { health: null, checking: true },
      global,
    })

    expect(wrapper.get('section').attributes('aria-busy')).toBe('true')
    expect(wrapper.get('[role="status"]').text()).toContain('检查中')
    expect(wrapper.get('[role="status"]').attributes('aria-live')).toBe('polite')
  })

  it('offers repairs for the reported directory and configured FFmpeg', async () => {
    const wrapper = mount(EnvironmentHealthPanel, {
      props: { health: unhealthyEnvironment },
      global,
    })
    const buttons = wrapper.findAll('button')

    await buttons.find((button) => button.text() === '创建目录')?.trigger('click')
    await buttons.find((button) => button.text() === '重新选择')?.trigger('click')
    await buttons.find((button) => button.text() === '选择 FFmpeg')?.trigger('click')
    await buttons.find((button) => button.text() === '使用系统版本')?.trigger('click')

    expect(wrapper.emitted()).toMatchObject({
      createDirectory: [[]],
      chooseDirectory: [[]],
      chooseFfmpeg: [[]],
      useSystemFfmpeg: [[]],
    })
  })
})
