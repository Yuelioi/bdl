<script setup lang="ts">
import { useId } from 'vue'

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

const fieldId = useId()
const messageId = `${fieldId}-message`
</script>

<template>
  <label class="ui-field" :class="{ 'has-message': error || helper }" :for="fieldId">
    <span>{{ label }}</span>
    <input
      :id="fieldId"
      v-model="model"
      :type
      :placeholder
      :disabled
      :min
      :aria-invalid="error ? true : undefined"
      :aria-describedby="error || helper ? messageId : undefined"
    />
    <small
      v-if="error || helper"
      :id="messageId"
      :class="{ error: Boolean(error) }"
      :aria-live="error ? 'polite' : undefined"
    >
      {{ error || helper }}
    </small>
  </label>
</template>

<style scoped>
.ui-field {
  display: grid;
  grid-template-rows: 18px var(--height-input);
  gap: var(--space-6);
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 600;
}

.ui-field > span {
  display: flex;
  align-items: center;
  line-height: 18px;
}

.ui-field.has-message {
  grid-template-rows: 18px var(--height-input) auto;
}

.ui-field small {
  color: var(--color-text-muted);
  font-size: var(--font-11);
  font-weight: 500;
}

.ui-field small.error {
  color: var(--color-danger);
}

input {
  width: 100%;
  height: var(--height-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-text);
  padding: 0 var(--space-12);
  font-size: var(--font-14);
}

input:focus {
  border-color: var(--color-accent);
  outline: 2px solid rgb(8 127 91 / 16%);
  outline-offset: 0;
}

input:disabled {
  background: var(--color-panel);
}
</style>
