<script setup lang="ts">
const {
  loaded,
  total = null,
  unit = '项',
  hasMore = false,
} = defineProps<{
  loaded: number
  total?: number | null
  unit?: string
  hasMore?: boolean
}>()
</script>

<template>
  <span class="source-load-status" role="status">
    已加载 <strong>{{ loaded.toLocaleString('zh-CN') }}</strong>
    <template v-if="total !== null"> / <strong>{{ total.toLocaleString('zh-CN') }}</strong></template>
    {{ unit }}<template v-if="hasMore && total === null">，还有更多</template>
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
