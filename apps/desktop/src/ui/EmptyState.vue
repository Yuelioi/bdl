<script setup lang="ts">
import { useSlots } from 'vue'

const {
  tone = 'neutral',
  layout = 'inline',
  compact = false,
  embedded = false,
  role,
} = defineProps<{
  title: string
  description?: string
  icon?: string
  tone?: 'neutral' | 'warning' | 'danger'
  layout?: 'inline' | 'stacked'
  compact?: boolean
  embedded?: boolean
  role?: 'status' | 'alert'
}>()

const slots = useSlots()
</script>

<template>
  <section
    class="ui-empty-state"
    :class="[`layout-${layout}`, `tone-${tone}`, { compact, embedded }]"
    :role
  >
    <div v-if="icon || slots.visual" class="empty-visual" aria-hidden="true">
      <slot name="visual">
        <UIcon :name="icon" />
      </slot>
    </div>
    <div class="empty-copy">
      <h3 class="text-balance">{{ title }}</h3>
      <p v-if="description" class="text-pretty">{{ description }}</p>
      <small v-if="slots.detail"><slot name="detail" /></small>
    </div>
    <div v-if="slots.action" class="empty-action">
      <slot name="action" />
    </div>
  </section>
</template>

<style scoped>
.ui-empty-state {
  min-height: 240px;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-md);
  padding: var(--space-xl);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-8);
  background: var(--color-inset);
}

.ui-empty-state.compact {
  min-height: 120px;
}

.ui-empty-state.embedded {
  border: 0;
  background: transparent;
}

.layout-stacked {
  flex-direction: column;
  text-align: center;
}

.empty-visual {
  width: 44px;
  height: 44px;
  flex: 0 0 auto;
  display: grid;
  place-items: center;
  border-radius: 50%;
  background: var(--color-accent-soft);
  color: var(--color-accent);
}

.empty-visual :deep(svg) {
  width: 20px;
  height: 20px;
}

.tone-warning .empty-visual {
  background: var(--color-warning-soft);
  color: var(--color-warning);
}

.tone-danger .empty-visual {
  background: var(--color-danger-soft);
  color: var(--color-danger);
}

.empty-copy {
  min-width: 0;
}

.empty-copy h3 {
  margin: 0;
  color: var(--color-text-strong);
  font-size: var(--font-16);
}

.empty-copy p,
.empty-copy small {
  max-width: 52ch;
  color: var(--color-muted);
  line-height: 1.6;
}

.empty-copy p {
  margin: var(--space-2xs) 0 0;
  font-size: var(--font-13);
}

.empty-copy small {
  display: block;
  margin-top: var(--space-4);
  font-size: var(--font-11);
}

.empty-action {
  flex: 0 0 auto;
}

@media (width <= 560px) {
  .ui-empty-state {
    flex-direction: column;
    text-align: center;
  }
}
</style>
