<script setup lang="ts">
import UiFormField from './FormField.vue'

const model = defineModel<string>({ default: '' })
const { rows = 6, disabled = false, error, helper } = defineProps<{
  label: string
  placeholder?: string
  rows?: number
  disabled?: boolean
  error?: string
  helper?: string
}>()
</script>

<template>
  <UiFormField v-slot="{ fieldId, describedBy, invalid }" :label :error :helper>
    <textarea
      :id="fieldId"
      v-model="model"
      :rows
      :placeholder
      :disabled
      :aria-invalid="invalid || undefined"
      :aria-describedby="describedBy"
    />
  </UiFormField>
</template>

<style scoped>
textarea {
  width: 100%;
  min-height: 112px;
  resize: vertical;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-inset);
  color: var(--color-text);
  padding: var(--space-8) var(--space-12);
  font-size: var(--font-13);
  line-height: 1.5;
}

textarea:focus {
  border-color: var(--color-accent);
  outline: 2px solid var(--color-focus-outline);
  outline-offset: 0;
}

textarea:disabled {
  background: var(--color-panel);
}
</style>
