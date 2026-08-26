<script setup lang="ts">
const {
  loaded,
  total = null,
  unit = '项',
} = defineProps<{
  loaded: number
  total?: number | null
  unit?: string
}>()
</script>

<template>
  <span class="source-load-status" role="status">
    已加载 <strong>{{ loaded }}</strong>
    <template v-if="total !== null"> / <strong>{{ total }}</strong></template>
    {{ unit }}
  </span>
</template>

<style scoped>
.source-load-status {
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2xs);
  color: var(--color-muted);
  font-size: var(--font-12);
  white-space: nowrap;
}

.source-load-status strong {
  color: var(--color-text);
  font-variant-numeric: tabular-nums;
}
</style>
