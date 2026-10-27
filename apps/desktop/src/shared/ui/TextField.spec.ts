import { defineComponent, h, ref } from 'vue';
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';

import UiTextField from './TextField.vue';

const InputStub = defineComponent({
  props: { modelValue: String },
  emits: ['update:modelValue'],
  setup(props, { emit, expose }) {
    const inputRef = ref<HTMLInputElement | null>(null);
    expose({ inputRef });
    return () =>
      h('input', {
        ref: inputRef,
        value: props.modelValue,
        onInput: (event: Event) => emit('update:modelValue', (event.target as HTMLInputElement).value),
      });
  },
});

describe('TextField validation', () => {
  it('associates an inline error with the native input', () => {
    const wrapper = mount(UiTextField, {
      props: {
        label: '开始时间',
        type: 'datetime-local',
        error: '开始时间必须晚于当前时间',
      },
      global: {
        stubs: {
          UInput: InputStub,
          UFormField: { props: ['error'], template: '<div><slot/><slot v-if="error" name="error"/></div>' },
        },
      },
    });
    const input = wrapper.get('input');
    const messageId = input.attributes('aria-describedby');

    expect(input.attributes('aria-invalid')).toBe('true');
    expect(wrapper.get(`#${messageId}`).text()).toBe('开始时间必须晚于当前时间');
    expect(wrapper.get(`#${messageId}`).attributes('aria-live')).toBe('polite');
  });
});
