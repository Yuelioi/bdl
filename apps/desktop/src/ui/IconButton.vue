<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    icon: string
    label: string
    variant?: 'secondary' | 'ghost' | 'danger'
    type?: 'button' | 'submit' | 'reset'
    disabled?: boolean
  }>(),
  {
    variant: 'secondary',
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
</script>

<template>
  <UButton
    class="ui-icon-button"
    square
    size="xs"
    :type="props.type"
    :icon="iconName"
    :color
    :variant="uiVariant"
    :disabled="props.disabled"
    :aria-label="props.label"
    :title="props.label"
  />
</template>

<style scoped>
.ui-icon-button {
  width: var(--height-button);
  height: var(--height-button);
}
</style>
