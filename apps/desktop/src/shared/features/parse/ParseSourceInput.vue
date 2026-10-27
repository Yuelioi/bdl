<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import UiButton from '../../ui/Button.vue';
import { extractBilibiliInputs } from '../../utils/bilibiliLinks';

const mode = defineModel<'single' | 'batch'>('mode', { required: true });
const input = defineModel<string>('input', { required: true });
const singleInput = defineModel<string>('singleInput', { required: true });
defineProps<{ busy: boolean; hasResults: boolean }>();
const emit = defineEmits<{ submit: []; paste: []; singlePaste: [event: ClipboardEvent] }>();
const modeOptions = [
  { label: '视频及所属合集', value: 'batch' },
  { label: '仅当前视频', value: 'single' },
] as const;
const multiline = ref(input.value.includes('\n'));
const value = computed({
  get: () => (mode.value === 'single' ? singleInput.value : input.value),
  set: (text: string) => {
    if (mode.value === 'single') singleInput.value = text;
    else input.value = text;
  },
});
watch(input, (text) => {
  if (text.includes('\n')) multiline.value = true;
});
const paste = (event: ClipboardEvent) => {
  if (mode.value === 'single') {
    event.preventDefault();
    emit('singlePaste', event);
    return;
  }
  const text = event.clipboardData?.getData('text/plain') ?? '';
  if (!text.includes('\n')) return;
  event.preventDefault();
  const links = extractBilibiliInputs(text);
  input.value = links.length ? links.join('\n') : text;
  multiline.value = input.value.includes('\n');
};
</script>

<template>
  <form class="parse-source-input" @submit.prevent="emit('submit')" @keydown.ctrl.enter.prevent="emit('submit')">
    <div class="source-input-row">
      <UTextarea
        v-if="mode === 'batch' && multiline"
        v-model="value"
        class="source-input-control"
        aria-label="Bilibili 链接或 BV / AV"
        placeholder="链接 / BV / AV，每行一个来源"
        :rows="3"
        :disabled="busy"
        color="neutral"
        variant="outline"
        size="sm"
        :ui="{ base: 'max-h-40 resize-y text-(length:--font-13)' }"
        @paste="paste"
      />
      <UInput
        v-else
        v-model="value"
        class="source-input-control"
        type="text"
        :aria-label="mode === 'single' ? '视频链接或 BV / AV' : 'Bilibili 链接或 BV / AV'"
        placeholder="粘贴 Bilibili 链接 / BV / AV"
        :disabled="busy"
        color="neutral"
        variant="outline"
        size="sm"
        :ui="{ base: 'min-h-[2.125rem] text-(length:--font-13)' }"
        @paste="paste"
      />
      <UiButton class="source-submit" type="submit" :disabled="busy" :aria-label="busy ? '正在解析' : '开始解析'">
        {{ busy ? '解析中…' : '解析' }}
      </UiButton>
    </div>
    <div class="source-input-tools">
      <div class="source-range">
        <USelect
          v-model="mode"
          :items="modeOptions"
          value-key="value"
          label-key="label"
          :disabled="busy"
          aria-label="解析范围"
          color="neutral"
          variant="outline"
          size="xs"
          :portal="true"
          :content="{ position: 'popper', align: 'start', sideOffset: 4, collisionPadding: 12 }"
          :ui="{ base: 'min-h-7 text-(length:--font-12)', content: 'z-50', item: 'text-(length:--font-12)' }"
        />
      </div>
      <div class="source-input-actions">
        <UButton v-if="!hasResults" variant="link" size="xs" :disabled="busy" @click="emit('paste')">粘贴链接</UButton>
        <UButton
          v-if="mode === 'batch'"
          variant="link"
          size="xs"
          :disabled="busy"
          :aria-expanded="multiline"
          @click="multiline = !multiline"
        >
          {{ multiline ? '收起输入' : '多行输入' }}
        </UButton>
      </div>
    </div>
  </form>
</template>

<style scoped>
.parse-source-input {
  display: grid;
  flex-shrink: 0;
  gap: 0.375rem;
  min-width: 0;
}

.source-input-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.source-input-control {
  flex: 1;
  min-width: 0;
}

.source-submit {
  min-width: 4.5rem;
  min-height: var(--height-input);
  border-radius: var(--radius-6);
  padding-inline: 1rem;
  font-weight: 500;
}

.source-input-tools,
.source-range,
.source-input-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.source-input-tools {
  justify-content: flex-start;
  flex-wrap: wrap;
}

.source-range {
  color: var(--color-muted);
  font-size: var(--font-12);
}
</style>
