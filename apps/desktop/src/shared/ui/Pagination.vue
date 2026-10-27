<script setup lang="ts">
import UiButton from './Button.vue';
import { usePagination, type PaginationProps } from './usePagination';
const props = withDefaults(defineProps<PaginationProps>(), { itemsPerPage: 20, disabled: false, label: '分页' });
const emit = defineEmits<{ 'update:page': [page: number] }>();
const { pageCount, targetPage, validTarget, jump } = usePagination(props, (page) => emit('update:page', page));
</script>
<template>
  <nav class="ui-pagination" :aria-label="label">
    <UPagination
      class="page-buttons"
      :page
      :total
      :items-per-page="itemsPerPage"
      :disabled
      :sibling-count="1"
      show-edges
      size="xs"
      color="neutral"
      variant="outline"
      active-color="primary"
      active-variant="solid"
      @update:page="emit('update:page', $event)"
    />
    <form class="page-jump" @submit.prevent="jump">
      <UInput
        v-model="targetPage"
        class="page-jump-input"
        color="neutral"
        variant="outline"
        size="xs"
        :ui="{ base: 'h-7 min-h-7 rounded-e-none px-1.5 py-1 text-center text-(length:--font-12)' }"
        type="text"
        inputmode="numeric"
        :aria-label="`${label}：跳转页码`"
        :title="`输入 1–${pageCount} 页，按回车跳转`"
        :disabled
      />
      <UiButton type="submit" size="compact" variant="secondary" :disabled="disabled || !validTarget">跳转</UiButton>
    </form>
  </nav>
</template>

<style scoped>
.ui-pagination,
.page-jump {
  display: flex;
  align-items: center;
  gap: var(--space-8);
}

.ui-pagination {
  flex-wrap: wrap;
}

.page-jump {
  gap: 0;
  color: var(--color-muted);
  font-size: var(--font-12);
}

.page-jump-input {
  width: 2.75rem;
}

.page-jump :deep(.ui-button) {
  margin-inline-start: -0.0625rem;
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}

.page-buttons :deep(button) {
  min-width: 1.75rem;
  height: 1.75rem;
  justify-content: center;
  background: var(--ui-bg);
}

.page-buttons :deep(button:hover:not(:disabled, [aria-current='page'])) {
  background: var(--color-hover-surface);
}

.ui-pagination :deep([aria-current='page']) {
  color: var(--color-on-accent);
  background: var(--color-accent);
}
</style>
