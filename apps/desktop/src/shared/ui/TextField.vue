<script setup lang="ts">
import UiFormField from './FormField.vue';

const model = defineModel<string>({ default: '' });
const {
  type = 'text',
  disabled = false,
  error,
  helper,
  min,
} = defineProps<{
  label: string;
  placeholder?: string;
  type?: 'text' | 'password' | 'url' | 'datetime-local';
  disabled?: boolean;
  error?: string;
  helper?: string;
  min?: string;
}>();
</script>

<template>
  <UiFormField v-slot="{ fieldId, describedBy, invalid }" :label :error :helper>
    <UInput
      class="w-full"
      color="neutral"
      variant="outline"
      size="sm"
      :ui="{ base: 'h-(--height-input) text-(length:--font-13)' }"
      :id="fieldId"
      v-model="model"
      :type
      :placeholder
      :disabled
      :min
      :aria-invalid="invalid || undefined"
      :aria-describedby="describedBy"
    />
  </UiFormField>
</template>
