<script setup lang="ts">
import { computed } from 'vue';

const { value, max = 100 } = defineProps<{
  value: number;
  max?: number;
}>();

const percent = computed(() => {
  if (max <= 0) {
    return 0;
  }
  return Math.min(100, Math.max(0, (value / max) * 100));
});
</script>

<template>
  <UProgress
    class="progress-track"
    :model-value="percent"
    :max="100"
    size="xs"
    color="primary"
    :ui="{ base: 'h-full bg-(--color-progress-track)', indicator: 'bg-(--color-accent)' }"
  />
</template>

<style scoped>
.progress-track {
  width: 100%;
  height: 0.5rem;
  overflow: hidden;
  border-radius: 62.4375rem;
  background: var(--color-progress-track);
}
</style>
