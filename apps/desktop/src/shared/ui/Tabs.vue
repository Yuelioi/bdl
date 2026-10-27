<script setup lang="ts">
import { computed } from 'vue';

export interface TabItem {
  label: string;
  value: string;
  count?: number;
}

const model = defineModel<string>({ required: true });
const { tabs, stretch = false } = defineProps<{
  tabs: TabItem[];
  stretch?: boolean;
}>();

const items = computed(() =>
  tabs.map((tab) => ({
    label: tab.label,
    value: tab.value,
    badge: tab.count,
  })),
);
</script>

<template>
  <UTabs
    :model-value="model"
    :items="items"
    class="ui-tabs min-w-0"
    color="primary"
    variant="link"
    size="sm"
    :content="false"
    :ui="{
      root: stretch ? 'w-full overflow-hidden' : 'w-full overflow-x-auto overflow-y-hidden',
      list: [
        'mb-0 min-h-9 items-stretch border-(--color-border) bg-transparent p-0',
        stretch ? 'gap-0' : 'gap-5',
      ].join(' '),
      trigger: [
        'rounded-none border-0 px-1 py-0 text-[0.8125rem] focus-visible:bg-(--color-hover-surface) focus-visible:outline-none',
        stretch ? 'min-w-0 flex-1 justify-center' : 'min-w-fit grow-0 justify-start',
      ].join(' '),
      trailingBadge:
        'min-w-0 rounded-none bg-transparent px-0 text-[0.75rem] font-normal tabular-nums text-(--color-muted) ring-0',
    }"
    @update:model-value="(value: string | number) => (model = String(value))"
  />
</template>

<style scoped>
.ui-tabs {
  width: 100%;
}

.ui-tabs :deep([data-slot='trigger']) {
  min-height: 2.25rem;
  color: var(--color-muted);
  font-weight: 500;
}

.ui-tabs :deep([data-slot='trigger'][data-state='active']) {
  color: var(--color-accent-strong);
  font-weight: 500;
}

.ui-tabs :deep([data-slot='trigger'][data-state='inactive']:hover:not(:disabled)) {
  color: var(--color-text);
}

</style>
