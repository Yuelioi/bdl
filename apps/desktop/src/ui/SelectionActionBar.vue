<script setup lang="ts">
import { useSlots } from 'vue'

const {
  selectedCount,
  totalCount,
  unit = '项',
} = defineProps<{
  selectedCount: number
  totalCount: number
  unit?: string
}>()
const slots = useSlots()
</script>

<template>
  <footer class="selection-action-bar">
    <slot name="leading">
      <p class="selection-summary">
        <strong>{{ selectedCount }}</strong> {{ unit }}已选
        <span>共 {{ totalCount }} {{ unit }}</span>
      </p>
    </slot>
    <div v-if="slots.actions" class="selection-actions">
      <slot name="actions" />
    </div>
  </footer>
</template>

<style scoped>
.selection-action-bar {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
  padding-top: var(--space-12);
  border-top: 1px solid var(--color-border);
}

.selection-summary {
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-13);
}

.selection-summary strong {
  color: var(--color-text);
  font-variant-numeric: tabular-nums;
}

.selection-summary span {
  margin-left: var(--space-8);
  font-size: var(--font-12);
}

.selection-actions {
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-8);
}

@media (width <= 700px) {
  .selection-action-bar {
    align-items: flex-start;
    flex-direction: column;
  }

  .selection-actions {
    justify-content: flex-start;
  }
}
</style>
