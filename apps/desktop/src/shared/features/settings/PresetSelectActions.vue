<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
const {
  disabled = false,
  createDisabled = false,
  manageLabel = '管理预设',
} = defineProps<{
  disabled?: boolean;
  createDisabled?: boolean;
  manageLabel?: string;
}>();
const emit = defineEmits<{ create: []; manage: [] }>();
</script>

<template>
  <div class="preset-select-actions">
    <UiButton variant="secondary" :disabled="disabled || createDisabled" @click="emit('create')">新增预设</UiButton>
    <UiButton
      class="preset-manage"
      variant="ghost"
      :aria-label="manageLabel"
      :disabled="disabled"
      @click="emit('manage')"
      >管理</UiButton
    >
  </div>
</template>

<style scoped>
.preset-select-actions {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.preset-manage {
  min-width: 0;
  padding-inline: var(--space-8);
}
</style>
