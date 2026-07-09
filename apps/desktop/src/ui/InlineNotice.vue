<script setup lang="ts">
import type { NoticeTone } from '../stores/feedback'

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
    <button v-if="props.actionLabel" type="button" @click="emit('action')">
      {{ props.actionLabel }}
    </button>
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

.inline-notice button {
  height: 26px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-accent-strong);
  padding: 0 var(--space-8);
  font-size: var(--font-12);
  font-weight: 700;
  white-space: nowrap;
}

.inline-notice button:hover {
  background: #eef8f3;
}

.tone-success {
  border-color: rgb(43 138 62 / 24%);
  background: #f5fbf6;
}

.tone-success .notice-dot {
  background: var(--color-success);
}

.tone-warning {
  border-color: rgb(230 119 0 / 28%);
  background: #fff8ef;
}

.tone-warning .notice-dot {
  background: var(--color-warning);
}

.tone-danger {
  border-color: rgb(201 42 42 / 24%);
  background: #fffafa;
}

.tone-danger .notice-dot {
  background: var(--color-danger);
}
</style>
