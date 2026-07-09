<script setup lang="ts">
import { computed } from 'vue'

const { value, max = 100 } = defineProps<{
  value: number
  max?: number
}>()

const percent = computed(() => {
  if (max <= 0) {
    return 0
  }
  return Math.min(100, Math.max(0, (value / max) * 100))
})
</script>

<template>
  <div class="progress-track" role="progressbar" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100">
    <span :style="{ width: `${percent}%` }" />
  </div>
</template>

<style scoped>
.progress-track {
  width: 100%;
  height: 8px;
  overflow: hidden;
  border-radius: 999px;
  background: #e3e8e5;
}

.progress-track span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: var(--color-accent);
}
</style>
