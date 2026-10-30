import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, expect, it, vi } from 'vitest';
import ParseActivityStatus from './ParseActivityStatus.vue';
import { parseProgress } from '../api/tauri';
import { createPinia, setActivePinia } from 'pinia';
import { useParseStore } from '../stores/parse';

vi.mock('../api/tauri', () => ({ parseProgress: vi.fn() }));
afterEach(() => {
  vi.useRealTimers();
  vi.resetAllMocks();
});

it('expires the countdown after a suspended window even if status polling stalls', async () => {
  setActivePinia(createPinia());
  useParseStore().operationsBySource.test = ['request-test'];
  vi.useFakeTimers();
  const startClock = vi.spyOn(globalThis, 'setInterval');
  const stopClock = vi.spyOn(globalThis, 'clearInterval');
  vi.mocked(parseProgress)
    .mockResolvedValueOnce({ active: true, waiting_seconds: 3, queued: false })
    .mockImplementation(() => new Promise(() => {}));
  const wrapper = mount(ParseActivityStatus, { props: { sourceId: 'test' } });
  await flushPromises();
  expect(wrapper.text()).toContain('休息 3 秒');
  await vi.advanceTimersByTimeAsync(500);
  vi.setSystemTime(Date.now() + 30000);
  window.dispatchEvent(new Event('focus'));
  await wrapper.vm.$nextTick();
  expect(wrapper.text()).toBe('正在解析…');
  wrapper.unmount();
  await vi.advanceTimersByTimeAsync(1000);
  expect(stopClock).toHaveBeenCalledWith(startClock.mock.results[0]?.value);
  expect(vi.mocked(parseProgress)).toHaveBeenCalledTimes(2);
  expect(vi.mocked(parseProgress)).toHaveBeenCalledWith('request-test');
});
