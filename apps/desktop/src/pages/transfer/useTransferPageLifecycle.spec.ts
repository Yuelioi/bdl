import { mount } from '@vue/test-utils';
import { defineComponent } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import { useTransferPageLifecycle } from './useTransferPageLifecycle';

describe('transfer page lifecycle', () => {
  it('refreshes the queue and owns its window listeners', () => {
    const queue = { list: vi.fn().mockResolvedValue(undefined) };
    const handlers = {
      closeContextMenu: vi.fn(),
      closeContextMenuOnEscape: vi.fn(),
    };
    const component = defineComponent({
      setup() {
        useTransferPageLifecycle(queue, handlers);
        return () => null;
      },
    });

    const wrapper = mount(component);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    window.dispatchEvent(new Event('blur'));

    expect(queue.list).toHaveBeenCalledOnce();
    expect(handlers.closeContextMenuOnEscape).toHaveBeenCalledOnce();
    expect(handlers.closeContextMenu).toHaveBeenCalledOnce();

    wrapper.unmount();
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    window.dispatchEvent(new Event('blur'));

    expect(handlers.closeContextMenuOnEscape).toHaveBeenCalledOnce();
    expect(handlers.closeContextMenu).toHaveBeenCalledOnce();
  });
});
