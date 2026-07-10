<script setup lang="ts">
import { useSlots } from 'vue'

defineProps<{
  label: string
}>()

const slots = useSlots()
</script>

<template>
  <div class="ui-section-toolbar">
    <span>{{ label }}</span>
    <div v-if="slots.actions" class="toolbar-actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<style scoped>
.ui-section-toolbar {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-8);
}

.ui-section-toolbar > span {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.toolbar-actions {
  display: inline-flex;
  align-items: center;
  gap: var(--space-4);
}
</style>
