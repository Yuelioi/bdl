<script setup lang="ts">
const { variant = 'secondary', type = 'button', disabled = false } = defineProps<{
  icon: string
  label: string
  variant?: 'secondary' | 'ghost' | 'danger'
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
}>()
</script>

<template>
  <button class="ui-icon-button" :class="`variant-${variant}`" :type :disabled :aria-label="label" :title="label">
    <span aria-hidden="true">{{ icon }}</span>
  </button>
</template>

<style scoped>
.ui-icon-button {
  width: var(--height-button);
  height: var(--height-button);
  display: inline-grid;
  place-items: center;
  border: 1px solid transparent;
  border-radius: var(--radius-6);
  font-weight: 700;
  line-height: 1;
  transition:
    background-color 120ms ease,
    border-color 120ms ease,
    color 120ms ease;
}

.ui-icon-button:focus-visible {
  outline: 2px solid rgb(8 127 91 / 30%);
  outline-offset: 2px;
}

.ui-icon-button:disabled {
  opacity: 0.56;
}

.variant-secondary {
  border-color: var(--color-border);
  background: var(--color-surface);
  color: var(--color-text);
}

.variant-secondary:hover:not(:disabled),
.variant-ghost:hover:not(:disabled) {
  background: var(--color-panel);
}

.variant-ghost {
  background: transparent;
  color: var(--color-muted);
}

.variant-danger {
  background: var(--color-danger);
  color: #ffffff;
}
</style>
