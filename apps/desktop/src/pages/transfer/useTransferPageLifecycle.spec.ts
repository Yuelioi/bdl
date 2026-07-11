import { mount } from '@vue/test-utils';
import { defineComponent, h, KeepAlive, nextTick, ref } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import { useTransferPageLifecycle } from './useTransferPageLifecycle';

describe('transfer page lifecycle', () => {
  it('refreshes the queue and owns its window listeners', async () => {
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

    const active = ref(true);
    const host = defineComponent({
      setup() {
        return () => h(KeepAlive, null, { default: () => (active.value ? h(component) : null) });
      },
    });

    const wrapper = mount(host);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    window.dispatchEvent(new Event('blur'));

    expect(queue.list).toHaveBeenCalledOnce();
    expect(handlers.closeContextMenuOnEscape).toHaveBeenCalledOnce();
    expect(handlers.closeContextMenu).toHaveBeenCalledOnce();

    active.value = false;
    await nextTick();
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    window.dispatchEvent(new Event('blur'));

    expect(handlers.closeContextMenuOnEscape).toHaveBeenCalledOnce();
    expect(handlers.closeContextMenu).toHaveBeenCalledOnce();
    wrapper.unmount();
  });
});
