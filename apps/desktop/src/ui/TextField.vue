<script setup lang="ts">
import UiFormField from './FormField.vue'

const model = defineModel<string>({ default: '' })
const { type = 'text', disabled = false, error, helper, min } = defineProps<{
  label: string
  placeholder?: string
  type?: 'text' | 'password' | 'url' | 'datetime-local'
  disabled?: boolean
  error?: string
  helper?: string
  min?: string
}>()

</script>

<template>
  <UiFormField v-slot="{ fieldId, describedBy, invalid }" :label :error :helper>
    <input
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

<style scoped>
input {
  width: 100%;
  height: var(--height-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-inset);
  color: var(--color-text);
  padding: 0 var(--space-12);
  font-size: var(--font-14);
}

input:focus {
  border-color: var(--color-accent);
  outline: 0;
  box-shadow: inset 0 0 0 1px var(--color-focus-outline);
}

input:disabled {
  background: var(--color-panel);
}
</style>
