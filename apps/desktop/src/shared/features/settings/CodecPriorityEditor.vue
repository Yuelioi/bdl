<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import type { SelectOption } from '../../ui/Select.vue';

const model = defineModel<string[]>({ required: true });
const {
  label,
  options,
  disabled = false,
} = defineProps<{
  label: string;
  options: readonly SelectOption[];
  disabled?: boolean;
}>();
const move = (index: number, offset: number) => {
  const next = [...model.value];
  const destination = index + offset;
  if (destination < 0 || destination >= next.length) return;
  [next[index], next[destination]] = [next[destination]!, next[index]!];
  model.value = next;
};
</script>

<template>
  <section class="codec-priority">
    <div class="codec-heading">
      <h4>{{ label }}优先级</h4>
    </div>
    <ol class="codec-list" :aria-label="`${label}优先级`">
      <li v-for="(value, index) in model" :key="index" class="codec-row">
        <span class="codec-position" aria-hidden="true">{{ index + 1 }}</span>
        <span class="codec-name">{{ options.find((option) => option.value === value)?.label ?? value }}</span>
        <div class="codec-actions">
          <UiButton
            variant="ghost"
            size="compact"
            :disabled="disabled || index === 0"
            :aria-label="`上移第 ${index + 1} 条${label}`"
            @click="move(index, -1)"
            ><UIcon name="i-tabler-arrow-up"
          /></UiButton>
          <UiButton
            variant="ghost"
            size="compact"
            :disabled="disabled || index === model.length - 1"
            :aria-label="`下移第 ${index + 1} 条${label}`"
            @click="move(index, 1)"
            ><UIcon name="i-tabler-arrow-down"
          /></UiButton>
        </div>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.codec-priority,
.codec-list {
  display: grid;
  gap: var(--space-8);
}

.codec-priority {
  min-width: 0;
  padding-top: var(--space-12);
  border-top: 1px solid var(--color-border);
}

.codec-heading,
.codec-actions {
  display: flex;
  align-items: center;
}

.codec-heading {
  justify-content: space-between;
  gap: var(--space-8);
}

.codec-heading h4 {
  margin: 0;
  font-size: var(--font-13);
}

.codec-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.codec-row {
  display: grid;
  grid-template-columns: 1.25rem minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-8);
}

.codec-position {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.codec-name {
  min-width: 0;
  padding-block: var(--space-6);
  font-size: var(--font-13);
}

.codec-actions :deep(button) {
  min-width: 2rem;
  padding-inline: var(--space-8);
}
</style>
