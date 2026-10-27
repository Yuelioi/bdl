<script setup lang="ts">
const searchOpen = defineModel<boolean>('searchOpen', { default: false });
withDefaults(
  defineProps<{
    searchLabel: string;
    optionsLabel?: string;
    optionsIcon?: string;
    manageLabel?: string;
    managing?: boolean;
    showSearch?: boolean;
    disabled?: boolean;
  }>(),
  { showSearch: true },
);
const emit = defineEmits<{ options: []; manage: [] }>();
</script>
<template>
  <div class="mobile-list-actions">
    <UButton
      variant="ghost"
      color="neutral"
      :ui="{ label: 'contents' }"
      v-if="showSearch !== false"
      type="button"
      class="mobile-icon-button"
      :aria-label="searchLabel"
      :aria-pressed="searchOpen"
      @click="searchOpen = !searchOpen"
    >
      <UIcon name="i-tabler-search" />
    </UButton>
    <UButton
      variant="ghost"
      color="neutral"
      :ui="{ label: 'contents' }"
      v-if="optionsLabel"
      type="button"
      class="mobile-icon-button"
      :aria-label="optionsLabel"
      :disabled="disabled"
      @click="emit('options')"
    >
      <UIcon :name="optionsIcon || 'i-tabler-adjustments-horizontal'" />
    </UButton>
    <UButton
      variant="ghost"
      color="neutral"
      :ui="{ label: 'contents' }"
      v-if="manageLabel"
      type="button"
      class="mobile-icon-button"
      :aria-label="managing ? '完成管理' : manageLabel"
      :aria-pressed="!!managing"
      :disabled="disabled"
      @click="emit('manage')"
    >
      <UIcon :name="managing ? 'i-tabler-check' : 'i-tabler-list-check'" />
    </UButton>
  </div>
</template>
<style scoped>
.mobile-list-actions {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.mobile-list-actions > .mobile-icon-button {
  width: 2.25rem;
  height: 2.125rem;
  flex-basis: 2.25rem;
}

.mobile-list-actions > .mobile-icon-button :deep(svg) {
  width: 1.25rem;
  height: 1.25rem;
}
</style>
