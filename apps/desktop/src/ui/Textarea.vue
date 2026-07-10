<script setup lang="ts">
import { useId } from 'vue'

const model = defineModel<string>({ default: '' })
const { rows = 6, disabled = false } = defineProps<{
  label: string
  placeholder?: string
  rows?: number
  disabled?: boolean
}>()

const fieldId = useId()
</script>

<template>
  <label class="ui-field" :for="fieldId">
    <span>{{ label }}</span>
    <textarea :id="fieldId" v-model="model" :rows :placeholder :disabled />
  </label>
</template>

<style scoped>
.ui-field {
  display: grid;
  gap: var(--space-6);
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 600;
}

.ui-field > span {
  min-height: 18px;
  display: flex;
  align-items: center;
  line-height: 18px;
}

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
