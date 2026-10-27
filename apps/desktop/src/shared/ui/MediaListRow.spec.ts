import { mount } from '@vue/test-utils';
import { expect, it } from 'vitest';
import MediaListRow from './MediaListRow.vue';
it('toggles card space once while isolating playback, title, actions and disabled rows', async () => {
  const wrapper = mount(MediaListRow, {
    props: { title: '视频', selected: true, playable: true },
    slots: { actions: '<button>更多</button>' },
    global: {
      stubs: {
        UButton: { template: '<button><slot/></button>' },
        UiCheckbox: true,
        UIcon: true,
        UTooltip: { template: '<div><slot/></div>' },
      },
    },
  });
  await wrapper.get('article').trigger('click');
  expect(wrapper.emitted('toggle')).toHaveLength(1);
  await wrapper.get('.media-title').trigger('click');
  expect(wrapper.emitted('toggle')).toHaveLength(2);
  await wrapper.get('.media-play').trigger('click');
  await wrapper.get('.media-actions button').trigger('click');
  expect(wrapper.emitted('play')).toHaveLength(1);
  expect(wrapper.emitted('toggle')).toHaveLength(2);
  await wrapper.setProps({ disabled: true });
  await wrapper.get('article').trigger('click');
  expect(wrapper.emitted('toggle')).toHaveLength(2);
});
