<script setup lang="ts">
import { computed } from 'vue'

const { variant = 'primary', size = 'default', type = 'button', disabled = false } = defineProps<{
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  size?: 'default' | 'compact'
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
}>()

const color = computed(() => (variant === 'danger' ? 'error' : variant === 'primary' ? 'primary' : 'neutral'))
const uiVariant = computed(() => {
  if (variant === 'primary' || variant === 'danger') {
    return 'solid'
  }

  return variant === 'secondary' ? 'outline' : 'ghost'
})
const toneClass = computed(() => `variant-${variant}`)
const sizeClass = computed(() => `size-${size}`)
</script>

<template>
  <UButton :class="['ui-button', toneClass, sizeClass]" :color :variant="uiVariant" size="sm" :type :disabled>
    <slot />
  </UButton>
</template>

<style scoped>
.ui-button {
  min-width: 76px;
  min-height: var(--height-button);
  justify-content: center;
  font-size: var(--font-13);
  font-weight: 650;
  white-space: nowrap;
}

.ui-button.variant-primary {
  background: var(--color-accent);
  color: var(--color-on-accent);
}

.ui-button.variant-primary:hover:not(:disabled) {
  background: var(--color-accent-strong);
}

.ui-button.variant-secondary {
  color: var(--color-text);
  background: var(--color-surface);
}

.ui-button.variant-ghost {
  color: var(--color-muted);
}

.ui-button.variant-secondary:hover:not(:disabled),
.ui-button.variant-ghost:hover:not(:disabled) {
  background: var(--color-panel);
}

.ui-button.variant-danger {
  background: var(--color-danger);
  color: var(--color-on-accent);
}

.ui-button.size-compact {
  min-width: 0;
  min-height: 28px;
  height: 28px;
  padding-inline: var(--space-8);
  font-size: var(--font-12);
}
</style>
