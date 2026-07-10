<script setup lang="ts">
import { computed } from 'vue'

export interface TabItem {
  label: string
  value: string
  count?: number
}

const model = defineModel<string>({ required: true })
const { tabs } = defineProps<{
  tabs: TabItem[]
}>()

const items = computed(() =>
  tabs.map((tab) => ({
    label: tab.label,
    value: tab.value,
    badge: tab.count,
  })),
)
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
      root: 'w-full',
      list: 'min-h-9 items-stretch gap-5 overflow-x-auto overflow-y-hidden border-b border-(--color-border) bg-transparent p-0',
      indicator: 'hidden',
      trigger: 'relative min-w-fit grow-0 justify-start rounded-none px-1 py-0 text-[13px] font-bold',
      trailingBadge: 'min-w-5 bg-(--color-panel) px-1.5 text-[11px] tabular-nums text-(--color-muted) ring-0',
    }"
    @update:model-value="(value: string | number) => (model = String(value))"
  />
</template>

<style scoped>
.ui-tabs {
  width: 100%;
}

.ui-tabs :deep([data-slot="trigger"]) {
  min-height: 36px;
  color: var(--color-muted);
}

.ui-tabs :deep([data-slot="trigger"][data-state="active"]) {
  color: var(--color-accent-strong);
}

.ui-tabs :deep([data-slot="trigger"][data-state="active"]::after) {
  position: absolute;
  right: 4px;
  bottom: -1px;
  left: 4px;
  height: 2px;
  border-radius: 999px 999px 0 0;
  background: var(--color-accent);
  content: "";
}

.ui-tabs :deep([data-slot="trigger"]:hover:not(:disabled)) {
  color: var(--color-text);
}
</style>
