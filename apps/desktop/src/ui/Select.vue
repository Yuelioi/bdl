<script setup lang="ts">
import { useId } from 'vue'

export interface SelectOption {
  label: string
  value: string
}

const model = defineModel<string>({ default: '' })
defineProps<{
  label: string
  options: SelectOption[]
  disabled?: boolean
}>()

const fieldId = useId()
</script>

<template>
  <label class="ui-field" :for="fieldId">
    <span>{{ label }}</span>
    <select :id="fieldId" v-model="model" :disabled>
      <option v-for="option in options" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </select>
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

select {
  width: 100%;
  height: var(--height-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-text);
  padding: 0 var(--space-12);
  font-size: var(--font-14);
}

select:focus {
  border-color: var(--color-accent);
  outline: 2px solid rgb(8 127 91 / 16%);
  outline-offset: 0;
}
</style>
