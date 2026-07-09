<script setup lang="ts">
import { useId } from 'vue'

const model = defineModel<string>({ default: '' })
const { type = 'text', disabled = false } = defineProps<{
  label: string
  placeholder?: string
  type?: 'text' | 'password' | 'url'
  disabled?: boolean
}>()

const fieldId = useId()
</script>

<template>
  <label class="ui-field" :for="fieldId">
    <span>{{ label }}</span>
    <input :id="fieldId" v-model="model" :type :placeholder :disabled />
  </label>
</template>

<style scoped>
.ui-field {
  display: grid;
  gap: var(--space-8);
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 600;
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
