<script setup lang="ts">
import type { NoticeTone } from '../stores/feedback'
import UiButton from './Button.vue'

const props = withDefaults(
  defineProps<{
    tone?: NoticeTone
    actionLabel?: string
  }>(),
  {
    tone: 'info',
  },
)

const emit = defineEmits<{
  action: []
}>()
</script>

<template>
  <div class="inline-notice" :class="`tone-${props.tone}`" role="status">
    <span class="notice-dot" aria-hidden="true"></span>
    <p><slot /></p>
    <UiButton v-if="props.actionLabel" size="compact" variant="secondary" @click="emit('action')">
      {{ props.actionLabel }}
    </UiButton>
  </div>
</template>

<style scoped>
.inline-notice {
  min-width: 0;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-8);
  padding: var(--space-8) var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-panel);
  color: var(--color-text);
  font-size: var(--font-12);
  line-height: 1.45;
}

.notice-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: var(--color-accent);
}

.inline-notice p {
  min-width: 0;
  margin: 0;
  overflow-wrap: anywhere;
}

.tone-success {
  border-color: color-mix(in oklab, var(--color-success) 34%, var(--color-border));
  background: var(--color-success-soft);
}

.tone-success .notice-dot {
  background: var(--color-success);
}

.tone-warning {
  border-color: color-mix(in oklab, var(--color-warning) 38%, var(--color-border));
  background: var(--color-warning-soft);
}

.tone-warning .notice-dot {
  background: var(--color-warning);
}

.tone-danger {
  border-color: color-mix(in oklab, var(--color-danger) 34%, var(--color-border));
  background: var(--color-danger-soft);
}

.tone-danger .notice-dot {
  background: var(--color-danger);
}
</style>
