import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';

import type { TransferTaskView } from '../../../shared/stores/transferView';
import TransferTaskTable from './TransferTaskTable.vue';
import UiCheckbox from '../../../shared/ui/Checkbox.vue';

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
});

describe('TransferTaskTable virtualization', () => {
  it('keeps selection scope complete while rendering a bounded large-list window', async () => {
    const views = Array.from({ length: 250 }, (_, index) => view(index));
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
          UButton: { template: '<button><slot /></button>' },
        },
      },
    });

    expect(wrapper.find('.task-table-row').attributes('aria-setsize')).toBe('250');
    expect(wrapper.findAll('.task-table-row').length).toBeLessThan(views.length);
    const scroll = wrapper.find('.transfer-scroll');
    Object.defineProperty(scroll.element, 'scrollTop', { value: Number.MAX_SAFE_INTEGER, configurable: true });
    Object.defineProperty(scroll.element, 'clientHeight', { value: 600, configurable: true });
    await scroll.trigger('scroll');
    expect(wrapper.findAll('.task-table-row').at(-1)?.attributes('aria-posinset')).toBe('250');
    wrapper.find('.transfer-selection-bar').findComponent(UiCheckbox).vm.$emit('update:modelValue', true);
    expect(wrapper.emitted('toggleVisibleSelection')?.[0]?.[0]).toEqual(views.map((view) => view.id));
  });
});
