<script setup lang="ts">
import { usePagination, type PaginationProps } from '../../ui/usePagination';
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
    <button
      class="page-step page-step-previous"
      type="button"
      :disabled="disabled || page <= 1"
      aria-label="上一页"
      @click="emit('update:page', page - 1)"
    >
      <UIcon name="i-tabler-chevron-left" aria-hidden="true" />
    </button>
    <form class="page-position" @submit.prevent="commitTarget">
      <div class="page-current">
        <input
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
    <button
      class="page-step page-step-next"
      type="button"
      :disabled="disabled || page >= pageCount"
      aria-label="下一页"
      @click="emit('update:page', page + 1)"
    >
      <UIcon name="i-tabler-chevron-right" aria-hidden="true" />
    </button>
  </nav>
</template>
<style scoped>
.mobile-pagination {
  width: 100%;
  min-width: 0;
  min-height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  flex-shrink: 0;
  padding: 0 2px;
  border: 0;
  background: var(--mobile-page-bg);
}

.page-step {
  width: 30px;
  height: 30px;
  flex: 0 0 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 6px;
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
  width: 17px;
  height: 17px;
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
  min-width: 58px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 6px;
  background: var(--color-inset);
  color: var(--color-text);
}

.page-current {
  display: flex;
  align-items: baseline;
  gap: 3px;
  font-size: var(--mobile-font-body);
  font-weight: var(--mobile-weight-item);
  white-space: nowrap;
}

.page-current input {
  width: 22px;
  height: 24px;
  border: 0;
  border-radius: 4px;
  outline: 0;
  background: transparent;
  color: var(--color-text-strong);
  font: inherit;
  font-size: var(--mobile-font-item-title);
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.page-current input:focus-visible {
  background: var(--mobile-card);
  box-shadow: inset 0 0 0 1px var(--color-accent);
}

.page-current input:disabled {
  color: var(--color-dimmed);
}

@media (hover: hover) {
  .page-step:hover:not(:disabled) {
    background: var(--color-accent-faint);
    color: var(--color-accent-strong);
  }
}
</style>
