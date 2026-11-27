<script setup lang="ts">
import type { NoticeTone } from '../stores/feedback'
import UiButton from './Button.vue'
import ErrorDetails from './ErrorDetails.vue'

const props = withDefaults(
  defineProps<{
    tone?: NoticeTone
    actionLabel?: string
    detail?: string
  }>(),
  {
    tone: 'info',
  },
)

const emit = defineEmits<{
  action: []
  openSettings: []
}>()
</script>

<template>
  <div class="inline-notice feedback-tone" :class="`tone-${props.tone}`" role="status">
    <span class="notice-dot" aria-hidden="true"></span>
    <p><slot /></p>
    <UiButton v-if="props.actionLabel" size="compact" variant="secondary" @click="emit('action')">
      {{ props.actionLabel }}
    </UiButton>
    <ErrorDetails v-if="props.detail" class="notice-details" :detail="props.detail" @open-settings="emit('openSettings')" />
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
  border: 0.0625rem solid var(--feedback-border);
  border-radius: var(--radius-6);
  background: var(--feedback-background);
  color: var(--color-text);
  font-size: var(--font-12);
  line-height: 1.45;
}

.notice-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 62.4375rem;
  background: var(--feedback-accent);
}

.notice-details {
  grid-column: 2 / -1;
}

.inline-notice p {
  min-width: 0;
  margin: 0;
  overflow-wrap: anywhere;
}

</style>
