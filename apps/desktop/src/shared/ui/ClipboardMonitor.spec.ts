import { mount, flushPromises } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, expect, it, vi } from 'vitest';
import ClipboardMonitor from './ClipboardMonitor.vue';
import { useSettingsStore } from '../stores/settings';
import { useClipboardStore } from '../stores/clipboard';
import { useUiStore } from '../stores/ui';
import { readClipboardText } from '../utils/clipboard';
vi.mock('../utils/clipboard', () => ({ readClipboardText: vi.fn() }));
beforeEach(() => {
  setActivePinia(createPinia());
  vi.spyOn(document, 'hasFocus').mockReturnValue(true);
  vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible');
});
const mountMonitor = () => mount(ClipboardMonitor, { global: { stubs: {
  UiDialog: { props: ['modelValue'], template: '<div v-if="modelValue"><slot/><slot name="footer"/></div>' },
  UiButton: { template: '<button><slot/></button>' },
} } });
it('reads only when enabled, stays quiet on failure, ignores non-links and adds only after confirmation', async () => {
  const wrapper = mountMonitor();
  expect(readClipboardText).not.toHaveBeenCalled();
  const toast = vi.spyOn(useUiStore(), 'pushToast');
  vi.mocked(readClipboardText).mockRejectedValue(new Error('clipboard unavailable'));
  useSettingsStore().saved.clipboard_monitor_enabled = true;
  await flushPromises();
  expect(toast).not.toHaveBeenCalled();
  vi.mocked(readClipboardText).mockResolvedValue('无关文字');
  window.dispatchEvent(new Event('focus'));
  await flushPromises();
  expect(wrapper.findAll('button')).toHaveLength(0);
  vi.mocked(readClipboardText).mockResolvedValue('分享 https://www.bilibili.com/video/BV1xx411c7mD');
  window.dispatchEvent(new Event('focus'));
  await flushPromises();
  expect(useClipboardStore().additions).toEqual([]);
  await wrapper.findAll('button')[1].trigger('click');
  expect(useClipboardStore().additions).toEqual(['https://www.bilibili.com/video/BV1xx411c7mD']);
  window.dispatchEvent(new Event('focus'));
  await flushPromises();
  expect(wrapper.findAll('button')).toHaveLength(0);
  wrapper.unmount();
});
it('discards pending reads after disabling and never reads in background', async () => {
  let resolve!: (value: string) => void;
  vi.mocked(readClipboardText).mockReturnValue(new Promise((done) => { resolve = done; }));
  useSettingsStore().saved.clipboard_monitor_enabled = true;
  const wrapper = mountMonitor();
  useSettingsStore().saved.clipboard_monitor_enabled = false;
  await flushPromises();
  resolve('BV1xx411c7mD');
  await flushPromises();
  expect(wrapper.findAll('button')).toHaveLength(0);
  vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('hidden');
  vi.mocked(readClipboardText).mockClear();
  useSettingsStore().saved.clipboard_monitor_enabled = true;
  window.dispatchEvent(new Event('focus'));
  await flushPromises();
  expect(readClipboardText).not.toHaveBeenCalled();
  wrapper.unmount();
});
