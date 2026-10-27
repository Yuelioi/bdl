<script setup lang="ts">
import { computed, ref } from 'vue';
import type { NamingPreset } from '../api/dto';
import { builtinNamingPresets, previewTemplate } from '../stores/settings';
import UiSelect from './Select.vue';

const model = defineModel<string>({ required: true });
const props = withDefaults(
  defineProps<{
    presets: NamingPreset[];
    extension?: string;
    disabled?: boolean;
  }>(),
  { extension: 'mp4', disabled: false },
);
const selectedId = ref('');
const choices = computed(() => [
  ...builtinNamingPresets().map((preset) => ({ label: preset.name, value: preset.id, template: preset.template })),
  ...props.presets.map((preset) => ({ label: preset.name, value: preset.id, template: preset.template })),
]);
const matchesTemplate = (template: string) => template === model.value || `${template}.{ext}` === model.value;
const selected = computed({
  get: () =>
    choices.value.find((item) => item.value === selectedId.value && matchesTemplate(item.template))?.value ??
    choices.value.find((item) => matchesTemplate(item.template))?.value ??
    'current',
  set: (value: string) => {
    selectedId.value = value;
    const preset = choices.value.find((item) => item.value === value);
    if (preset) model.value = preset.template;
  },
});
const options = computed(() => [
  ...choices.value,
  ...(selected.value === 'current' ? [{ label: '当前命名规则', value: 'current' }] : []),
]);
defineExpose({
  selectedPreset: computed(() => choices.value.find((item) => item.value === selected.value)),
  selectPreset: (id: string) => {
    selected.value = id;
  },
});
</script>

<template>
  <div class="grid min-w-0 gap-3">
    <div class="naming-preset-row" :class="{ 'has-actions': $slots.actions }">
      <UiSelect v-model="selected" label="命名预设" :options="options" :disabled="disabled" />
      <slot name="actions" />
    </div>
    <p class="m-0 text-xs leading-5 wrap-anywhere text-(--color-muted)">
      文件名预览：<code>{{ previewTemplate(model, extension) }}</code>
    </p>
  </div>
</template>

<style scoped>
.naming-preset-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: end;
  gap: var(--space-8);
}

.naming-preset-row.has-actions {
  grid-template-columns: minmax(0, 1fr) auto;
}
</style>
