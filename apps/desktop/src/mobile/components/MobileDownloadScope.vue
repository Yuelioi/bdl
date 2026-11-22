<script setup lang="ts">
import UiButton from '../../shared/ui/Button.vue';

defineProps<{ disabled?: boolean }>();
const emit = defineEmits<{ download: [] }>();
</script>

<template>
  <UiButton
    class="download-scope-trigger"
    variant="secondary"
    size="compact"
    aria-label="下载全部"
    aria-haspopup="dialog"
    :disabled="disabled"
    @click="emit('download')"
  >
    <span class="download-scope-label">全部</span>
    <UIcon name="i-tabler-chevron-down" class="size-4" aria-hidden="true" />
  </UiButton>
</template>

<style scoped>
.ui-button.download-scope-trigger {
  flex: 0 0 auto;
  gap: var(--space-4);
  min-height: 2.75rem;
  height: auto;
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}

.download-scope-label {
  white-space: nowrap;
}
</style>
