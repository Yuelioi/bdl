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
  '||': 'pause',
  '>': 'play',
  r: 'refresh',
  x: 'x',
  '-': 'trash',
  f: 'file',
  d: 'folder',
  '?': 'help',
  v: 'chevron-down',
}

const iconPaths: Record<string, string[]> = {
  pause: ['M8 5v14', 'M16 5v14'],
  play: ['M8 5v14l11-7-11-7z'],
  refresh: ['M20 6v5h-5', 'M19 13a7 7 0 1 1-2.05-4.95L20 11'],
  x: ['M6 6l12 12', 'M18 6L6 18'],
  trash: ['M4 7h16', 'M10 11v6', 'M14 11v6', 'M6 7l1 14h10l1-14', 'M9 7V4h6v3'],
  file: ['M6 3h8l4 4v14H6z', 'M14 3v5h5'],
  folder: ['M3 6h7l2 2h9v11H3z'],
  copy: ['M8 8h11v11H8z', 'M5 16H4V4h12v1'],
  help: ['M12 18h.01', 'M9.25 9a3 3 0 1 1 4.32 2.69c-.95.57-1.57 1.13-1.57 2.31', 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z'],
  'chevron-down': ['M7 10l5 5 5-5'],
  more: ['M5 12h.01', 'M12 12h.01', 'M19 12h.01'],
}

const normalizedIcon = computed(() => aliases[props.icon] ?? props.icon)
const paths = computed(() => iconPaths[normalizedIcon.value] ?? iconPaths.help)
</script>

<template>
  <button
    class="ui-icon-button"
    :class="`variant-${props.variant}`"
    :type="props.type"
    :disabled="props.disabled"
    :aria-label="props.label"
    :title="props.label"
  >
    <svg aria-hidden="true" class="button-icon" viewBox="0 0 24 24">
      <path v-for="path in paths" :key="path" :d="path" />
    </svg>
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
  transition:
    background-color 120ms ease,
    border-color 120ms ease,
    color 120ms ease;
}

.button-icon {
  width: 16px;
  height: 16px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
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
  border-color: var(--color-danger);
  color: #ffffff;
}
</style>
