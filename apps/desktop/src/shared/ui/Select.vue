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
  hideLabel = false,
} = defineProps<{
  label: string;
  options: readonly SelectOption[];
  disabled?: boolean;
  error?: string;
  helper?: string;
  hideLabel?: boolean;
}>();
</script>

<template>
  <UiFormField :label :error :helper :hide-label="hideLabel">
    <template v-if="$slots.hint" #hint><slot name="hint" /></template>
    <template #default="{ fieldId, describedBy, invalid }">
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
          base: 'h-(--height-input) min-h-(--height-input) py-0 font-medium text-(length:--font-13)',
          trailingIcon: 'size-4',
          itemTrailingIcon: 'size-4 text-(--color-accent)',
          content: 'bdl-select-content z-50 !max-h-44',
          item: 'bdl-select-item font-medium',
        }"
        class="ui-select w-full"
      />
    </template>
  </UiFormField>
</template>
