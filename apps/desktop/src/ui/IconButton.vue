<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    icon: string
    label: string
    variant?: 'secondary' | 'ghost' | 'danger'
    size?: 'default' | 'compact'
    type?: 'button' | 'submit' | 'reset'
    disabled?: boolean
  }>(),
  {
    variant: 'secondary',
    size: 'default',
    type: 'button',
    disabled: false,
  },
)

const aliases: Record<string, string> = {
  '||': 'player-pause',
  '>': 'player-play',
  pause: 'player-pause',
  play: 'player-play',
  r: 'refresh',
  x: 'x',
  '-': 'trash',
  trash: 'trash',
  f: 'file',
  file: 'file',
  d: 'folder',
  folder: 'folder',
  '?': 'help-circle',
  v: 'chevron-down',
  copy: 'copy',
  info: 'info-circle',
  more: 'dots',
  refresh: 'refresh',
}

const iconName = computed(() => `i-tabler-${aliases[props.icon] ?? props.icon}`)
const color = computed(() => (props.variant === 'danger' ? 'error' : 'neutral'))
const uiVariant = computed(() => (props.variant === 'secondary' ? 'outline' : props.variant))
const sizeClass = computed(() => `size-${props.size}`)
</script>

<template>
  <UButton
    :class="['ui-icon-button', sizeClass, `variant-${props.variant}`]"
    square
    size="xs"
    :type="props.type"
    :icon="iconName"
    :color
    :variant="uiVariant"
    :disabled="props.disabled"
    :aria-label="props.label"
    :title="props.label"
    :ui="{ base: 'p-0 justify-center', leadingIcon: 'size-4' }"
  />
</template>

<style scoped>
.ui-icon-button {
  width: var(--height-button);
  height: var(--height-button);
  min-width: var(--height-button);
  min-height: var(--height-button);
  display: inline-grid;
  place-items: center;
  padding: 0;
  line-height: 1;
}

.ui-icon-button.variant-secondary {
  background: var(--ui-bg);
}

.ui-icon-button.variant-secondary:hover:not(:disabled) {
  background: var(--color-hover-surface);
}

.ui-icon-button :deep(svg) {
  width: 16px;
  height: 16px;
  margin: 0;
}

.ui-icon-button.size-compact {
  width: 28px;
  height: 28px;
  min-width: 28px;
  min-height: 28px;
}

.ui-icon-button.size-compact :deep(svg) {
  width: 15px;
  height: 15px;
}
</style>
