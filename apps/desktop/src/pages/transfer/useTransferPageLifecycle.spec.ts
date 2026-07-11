/* eslint-disable vue/one-component-per-file -- KeepAlive lifecycle requires a host and cached fixture. */
import { mount } from '@vue/test-utils';
import { defineComponent, h, KeepAlive, nextTick, ref } from 'vue';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useTransferPageLifecycle } from './useTransferPageLifecycle';

describe('transfer page lifecycle', () => {
  afterEach(() => vi.useRealTimers());

  it('refreshes the queue and owns its window listeners', async () => {
    vi.useFakeTimers();
    const queue = {
      list: vi.fn().mockResolvedValue(undefined),
      reconcile: vi.fn().mockResolvedValue(undefined),
      hasInFlightTasks: true,
    };
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
    await vi.advanceTimersByTimeAsync(2_000);
    expect(queue.reconcile).toHaveBeenCalledOnce();

    active.value = false;
    await nextTick();
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    window.dispatchEvent(new Event('blur'));

    expect(handlers.closeContextMenuOnEscape).toHaveBeenCalledOnce();
    expect(handlers.closeContextMenu).toHaveBeenCalledOnce();
    await vi.advanceTimersByTimeAsync(2_000);
    expect(queue.reconcile).toHaveBeenCalledOnce();
    wrapper.unmount();
  });
});
