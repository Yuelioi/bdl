<script setup lang="ts">
import UiButton from './Button.vue';

defineProps<{
  selectedCount: number;
  items: { label: string; icon: string; color?: 'error'; onSelect: () => void }[][];
  loading?: boolean;
}>();

const emit = defineEmits<{ refresh: [] }>();
</script>

<template>
  <div class="bulk-action-bar" :class="{ selected: selectedCount > 0 }">
    <div v-if="selectedCount > 0" class="bulk-copy">
      <strong>已选 {{ selectedCount }} 项</strong>
    </div>
    <div class="bulk-actions">
      <UiButton size="compact" variant="ghost" :disabled="loading" @click="emit('refresh')">
        <UIcon name="i-tabler-refresh" class="size-4" aria-hidden="true" />
        刷新
      </UiButton>
      <UDropdownMenu
        v-if="items.length"
        :items="items"
        :content="{ align: 'end', side: 'top', sideOffset: 4, collisionPadding: 12 }"
      >
        <UiButton size="compact" variant="secondary" :disabled="loading">
          批量操作 <UIcon name="i-tabler-chevron-up" class="size-4" aria-hidden="true" />
        </UiButton>
      </UDropdownMenu>
    </div>
  </div>
</template>

<style scoped>
.bulk-action-bar {
  min-width: 0;
  min-height: 2rem;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
}

.bulk-action-bar.selected {
  justify-content: space-between;
  background: transparent;
  border: 0;
  padding: 0;
}

.bulk-copy {
  min-width: 0;
  white-space: nowrap;
}

.bulk-copy strong {
  font-size: var(--font-13);
  font-weight: 500;
}

.bulk-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.25rem;
}

@media (width <= 720px) {
  .bulk-action-bar.selected {
    align-items: flex-start;
    flex-wrap: wrap;
  }
}
</style>
