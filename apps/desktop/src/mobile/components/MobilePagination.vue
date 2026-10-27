<script setup lang="ts">
import { usePagination, type PaginationProps } from '../../shared/ui/usePagination';
const props = withDefaults(defineProps<PaginationProps>(), { itemsPerPage: 20, disabled: false, label: '分页' });
const emit = defineEmits<{ 'update:page': [page: number] }>();
const { pageCount, targetPage, validTarget, jump } = usePagination(props, (page) => emit('update:page', page));

const commitTarget = () => {
  if (!validTarget.value) {
    targetPage.value = String(props.page);
    return;
  }
  jump();
};

const selectTarget = (event: FocusEvent) => {
  (event.currentTarget as HTMLInputElement).select();
};
</script>
<template>
  <nav class="mobile-pagination" :aria-label="label">
    <UButton
      variant="ghost"
      color="neutral"
      :ui="{ base: 'p-0', label: 'contents' }"
      class="page-step page-step-previous"
      type="button"
      :disabled="disabled || page <= 1"
      aria-label="上一页"
      @click="emit('update:page', page - 1)"
    >
      <UIcon name="i-tabler-chevron-left" aria-hidden="true" />
    </UButton>
    <form class="page-position" @submit.prevent="commitTarget">
      <div class="page-current">
        <UInput
          class="page-target"
          color="neutral"
          variant="outline"
          size="xs"
          :ui="{ base: 'h-6 px-0 py-0 text-center text-(length:--font-13) tabular-nums' }"
          v-model="targetPage"
          type="text"
          inputmode="numeric"
          enterkeyhint="go"
          :aria-label="`${label}：当前第 ${page} 页，输入 1 到 ${pageCount} 跳转`"
          :disabled="disabled"
          @focus="selectTarget"
          @change="commitTarget"
        />
        <span aria-hidden="true">/ {{ pageCount }}</span>
      </div>
    </form>
    <UButton
      variant="ghost"
      color="neutral"
      :ui="{ base: 'p-0', label: 'contents' }"
      class="page-step page-step-next"
      type="button"
      :disabled="disabled || page >= pageCount"
      aria-label="下一页"
      @click="emit('update:page', page + 1)"
    >
      <UIcon name="i-tabler-chevron-right" aria-hidden="true" />
    </UButton>
  </nav>
</template>
<style scoped>
.mobile-pagination {
  width: 100%;
  min-width: 0;
  min-height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
  flex-shrink: 0;
  padding: 0 0.125rem;
  border: 0;
  background: transparent;
}

.page-step {
  width: 1.875rem;
  height: 1.875rem;
  flex: 0 0 1.875rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--color-text);
  font-family: inherit;
  font-size: var(--mobile-font-body);
  font-weight: var(--mobile-weight-item);
  transition:
    color var(--duration-fast) var(--ease-out),
    background var(--duration-fast) var(--ease-out);
}

.page-step svg {
  width: 1.0625rem;
  height: 1.0625rem;
  flex-shrink: 0;
}

.page-step:disabled {
  color: var(--color-dimmed);
  opacity: 0.55;
}

.page-step:active:not(:disabled) {
  background: var(--color-accent-faint);
  color: var(--color-accent-strong);
}

.page-position {
  min-width: 3.625rem;
  height: 1.875rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  color: var(--color-text);
}

.page-current {
  display: flex;
  align-items: baseline;
  gap: 0.1875rem;
  font-size: var(--mobile-font-body);
  font-weight: var(--mobile-weight-item);
  white-space: nowrap;
}

.page-target {
  width: 2.25rem;
}

@media (hover: hover) {
  .page-step:hover:not(:disabled) {
    background: var(--color-accent-faint);
    color: var(--color-accent-strong);
  }
}
</style>
