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
    <div class="selection-leading">
      <slot name="leading">
        <p class="selection-summary">
          已选 <strong>{{ selectedCount }}</strong> {{ unit }}
          <span>共 {{ totalCount }} {{ unit }}</span>
        </p>
      </slot>
    </div>
    <div v-if="slots.selection" class="selection-actions">
      <div class="selection-action-group selection-control-group">
        <slot name="selection" />
      </div>
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
  line-height: 1.4;
}

.selection-summary strong {
  color: var(--color-text);
  font-variant-numeric: tabular-nums;
}

.selection-summary span {
  margin-left: var(--space-8);
  font-size: var(--font-12);
}

.selection-leading,
.selection-actions {
  min-width: 0;
  min-height: 28px;
  display: flex;
  align-items: center;
}

.selection-actions {
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-12);
}

.selection-action-group {
  min-height: 28px;
  display: flex;
  align-items: center;
  gap: var(--space-8);
}

@media (width <= 700px) {
  .selection-action-bar {
    align-items: stretch;
    flex-direction: column;
  }

  .selection-leading,
  .selection-actions {
    justify-content: flex-start;
  }
}
</style>
