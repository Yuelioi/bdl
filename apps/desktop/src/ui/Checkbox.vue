<script setup lang="ts">
const model = defineModel<boolean>({ default: false })
defineProps<{
  label: string
  disabled?: boolean
}>()
</script>

<template>
  <label class="ui-checkbox">
    <input v-model="model" type="checkbox" :disabled />
    <span class="check-mark" aria-hidden="true" />
    <span>{{ label }}</span>
  </label>
</template>

<style scoped>
.ui-checkbox {
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-8);
  color: var(--color-text);
  font-size: var(--font-13);
  font-weight: 600;
}

input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.check-mark {
  width: 16px;
  height: 16px;
  display: inline-grid;
  place-items: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-4);
  background: var(--color-surface);
}

input:checked + .check-mark {
  border-color: var(--color-accent);
  background: var(--color-accent);
}

input:checked + .check-mark::after {
  content: "";
  width: 8px;
  height: 5px;
  border-left: 2px solid #ffffff;
  border-bottom: 2px solid #ffffff;
  transform: rotate(-45deg) translateY(-1px);
}

input:focus-visible + .check-mark {
  outline: 2px solid rgb(8 127 91 / 30%);
  outline-offset: 2px;
}

input:disabled + .check-mark,
input:disabled ~ span {
  opacity: 0.56;
}
</style>
