<script setup lang="ts">
import UiFormField from './FormField.vue';

export interface SelectOption {
  label: string;
  value: string;
}

const model = defineModel<string>({ default: '' });
const {
  label,
  options,
  disabled = false,
  error,
  helper,
} = defineProps<{
  label: string;
  options: readonly SelectOption[];
  disabled?: boolean;
  error?: string;
  helper?: string;
}>();
</script>

<template>
  <UiFormField v-slot="{ fieldId, describedBy, invalid }" :label :error :helper>
    <USelect
      :id="fieldId"
      v-model="model"
      :items="options"
      value-key="value"
      label-key="label"
      color="neutral"
      variant="outline"
      size="md"
      :disabled
      trailing-icon="i-tabler-chevron-down"
      selected-icon="i-tabler-check"
      :portal="true"
      :aria-invalid="invalid || undefined"
      :aria-describedby="describedBy"
      :content="{ side: 'bottom', align: 'start', sideOffset: 4, collisionPadding: 12, position: 'popper' }"
      :ui="{
        base: 'h-9 min-h-9 py-0',
        trailingIcon: 'size-4',
        content: 'bdl-select-content z-50 !max-h-44',
        item: 'bdl-select-item font-medium',
      }"
      class="ui-select"
    />
  </UiFormField>
</template>

<style scoped>
:deep(.ui-select) {
  width: 100%;
  height: var(--height-input);
  min-height: var(--height-input);
  font-weight: 650;
  background: var(--color-control);
}
</style>
