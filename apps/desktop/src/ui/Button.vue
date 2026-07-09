<script setup lang="ts">
const { variant = 'primary', type = 'button', disabled = false } = defineProps<{
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
}>()
</script>

<template>
  <button class="ui-button" :class="`variant-${variant}`" :type :disabled>
    <slot />
  </button>
</template>

<style scoped>
.ui-button {
  min-width: 76px;
  height: var(--height-button);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-8);
  padding: 0 var(--space-12);
  border: 1px solid transparent;
  border-radius: var(--radius-6);
  font-weight: 650;
  font-size: var(--font-13);
  line-height: 1;
  white-space: nowrap;
  transition:
    background-color 120ms ease,
    border-color 120ms ease,
    color 120ms ease,
    box-shadow 120ms ease;
}

.ui-button:focus-visible {
  outline: 2px solid rgb(8 127 91 / 30%);
  outline-offset: 2px;
}

.ui-button:disabled {
  opacity: 0.56;
}

.variant-primary {
  background: var(--color-accent);
  color: #ffffff;
}

.variant-primary:hover:not(:disabled) {
  background: var(--color-accent-strong);
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
