<script setup lang="ts">
const {
  title,
  description,
  variant = 'section',
} = defineProps<{
  title: string;
  description?: string;
  variant?: 'section' | 'panel';
}>();
</script>

<template>
  <details class="ui-disclosure" :class="`variant-${variant}`">
    <summary>
      <span class="disclosure-copy">
        <strong class="text-balance">{{ title }}</strong>
        <span v-if="description" class="text-pretty">{{ description }}</span>
      </span>
      <UIcon class="disclosure-chevron" name="i-tabler-chevron-down" aria-hidden="true" />
    </summary>
    <div class="disclosure-content">
      <slot />
    </div>
  </details>
</template>

<style scoped>
.ui-disclosure {
  min-width: 0;
  display: grid;
  gap: var(--space-12);
}

.variant-section {
  padding-top: var(--space-16);
  border-top: 0.0625rem solid var(--color-border);
}

.variant-panel {
  padding: var(--space-10, 0.625rem) var(--space-12);
  border: 0.0625rem solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

summary {
  min-width: 0;
  min-height: 2.375rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
  cursor: pointer;
  list-style: none;
}

summary::-webkit-details-marker {
  display: none;
}

.disclosure-copy {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.disclosure-copy strong {
  margin: 0;
  color: var(--color-text);
  font-size: var(--font-14);
  font-weight: 500;
  line-height: 1.3;
}

.variant-panel .disclosure-copy strong {
  font-size: var(--font-13);
}

.disclosure-copy > span {
  min-width: 0;
  overflow: hidden;
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.5;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.disclosure-chevron {
  width: 1rem;
  height: 1rem;
  flex: 0 0 auto;
  color: var(--color-muted);
  transition: transform var(--duration-fast) var(--ease-out);
}

.ui-disclosure[open] .disclosure-chevron {
  transform: rotate(180deg);
}

.disclosure-content {
  min-width: 0;
  display: grid;
  gap: var(--space-12);
}
</style>
