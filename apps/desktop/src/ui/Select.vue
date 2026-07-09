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
    <span class="select-shell">
      <select :id="fieldId" v-model="model" :disabled>
        <option v-for="option in options" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
      <svg aria-hidden="true" class="select-icon" viewBox="0 0 24 24">
        <path d="M7 10l5 5 5-5" />
      </svg>
    </span>
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

.select-shell {
  position: relative;
  display: block;
  min-width: 0;
}

select {
  appearance: none;
  width: 100%;
  height: var(--height-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-text);
  padding: 0 38px 0 var(--space-12);
  font-size: var(--font-14);
  font-weight: 650;
  line-height: var(--height-input);
  cursor: pointer;
}

select:focus {
  border-color: var(--color-accent);
  outline: 2px solid rgb(8 127 91 / 16%);
  outline-offset: 0;
}

select:disabled {
  background: var(--color-panel);
  cursor: not-allowed;
}

option {
  color: var(--color-text);
  background: var(--color-surface);
}

.select-icon {
  pointer-events: none;
  position: absolute;
  right: var(--space-12);
  top: 50%;
  width: 16px;
  height: 16px;
  translate: 0 -50%;
  fill: none;
  stroke: var(--color-muted);
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}
</style>
