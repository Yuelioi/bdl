<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import UiButton from '../../ui/Button.vue';
import { extractBilibiliInputs } from '../../utils/bilibiliLinks';

const mode = defineModel<'single' | 'batch'>('mode', { required: true });
const input = defineModel<string>('input', { required: true });
const singleInput = defineModel<string>('singleInput', { required: true });
const props = defineProps<{ busy: boolean; paused: boolean; pausing: boolean; mobile?: boolean }>();
const emit = defineEmits<{ submit: []; pause: []; resume: []; paste: []; singlePaste: [event: ClipboardEvent] }>();
const primaryLabel = computed(() => {
  if (props.pausing) return '正在暂停…';
  if (props.paused) return '继续解析';
  return props.busy ? '暂停解析' : '解析';
});
const submit = () => {
  if (props.pausing) return;
  if (props.paused) emit('resume');
  else if (props.busy) emit('pause');
  else emit('submit');
};
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
  <form
    class="parse-source-input"
    :class="{ 'is-mobile': mobile }"
    @submit.prevent="submit"
    @keydown.ctrl.enter.prevent="submit"
  >
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
      :placeholder="mobile ? 'Bilibili 链接或 BV / AV' : '粘贴 Bilibili 链接 / BV / AV'"
      icon="i-tabler-link"
      :disabled="busy"
      color="neutral"
      variant="outline"
      size="sm"
      :ui="{ base: mobile ? 'min-h-12 text-(length:--font-14)' : 'min-h-[2.125rem] text-(length:--font-13)' }"
      @paste="paste"
    />
    <div class="source-input-tools">
      <div v-if="!mobile" class="source-range">
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
          :ui="{
            base: mobile ? 'min-h-11 text-(length:--font-13)' : 'min-h-7 text-(length:--font-12)',
            content: 'z-50',
            item: 'text-(length:--font-12)',
          }"
        />
      </div>
      <div class="source-input-actions">
        <UButton
          variant="ghost"
          color="neutral"
          icon="i-tabler-clipboard"
          :aria-label="mobile ? '粘贴链接' : undefined"
          size="xs"
          :disabled="busy"
          @click="emit('paste')"
        >{{ mobile ? '粘贴' : '粘贴链接' }}</UButton>
        <UButton
          v-if="mode === 'batch'"
          variant="ghost"
          color="neutral"
          icon="i-tabler-align-left"
          size="xs"
          :disabled="busy"
          :aria-expanded="multiline"
          @click="multiline = !multiline"
        >
          {{ multiline ? '收起输入' : '多行输入' }}
        </UButton>
      </div>
    </div>
    <UiButton
      class="source-submit"
      type="submit"
      :disabled="pausing"
      :aria-label="busy ? primaryLabel : '开始解析'"
    >
      {{ primaryLabel }}
      <UIcon v-if="mobile && !busy" name="i-tabler-arrow-right" class="size-4" aria-hidden="true" />
    </UiButton>
  </form>
</template>

<style scoped>
.parse-source-input {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  flex-shrink: 0;
  gap: 0.375rem var(--space-8);
  min-width: 0;
}

.source-input-control {
  grid-column: 1;
  grid-row: 1;
  min-width: 0;
}

.source-submit {
  grid-column: 2;
  grid-row: 1;
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
  grid-column: 1 / -1;
  justify-content: flex-start;
  flex-wrap: wrap;
}

.source-range {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.parse-source-input.is-mobile {
  grid-template-columns: minmax(0, 1fr);
  gap: var(--space-8);
}

.is-mobile .source-submit {
  grid-column: 1;
  grid-row: 3;
  justify-self: end;
  min-height: 3rem;
  min-width: 6rem;
  margin-top: var(--space-8);
}

.is-mobile .source-input-tools {
  grid-row: 2;
}

.is-mobile .source-input-actions {
  gap: var(--space-4);
}

.is-mobile .source-input-actions > button {
  min-height: 2.75rem;
  font-size: var(--font-13);
}

@media (width <= 360px) {
  .is-mobile .source-input-actions > button {
    padding-inline: var(--space-4);
  }
}
</style>
