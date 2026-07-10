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
    class="ui-tabs"
    color="primary"
    variant="link"
    size="sm"
    :content="false"
    :ui="{
      root: 'w-full',
      list: 'bg-transparent p-0 border-0 gap-0',
      indicator: 'hidden',
      trigger: 'grow-0 justify-start',
      trailingBadge: 'bg-transparent text-inherit ring-0',
    }"
    @update:model-value="(value: string | number) => (model = String(value))"
  />
</template>

<style scoped>
.ui-tabs {
  width: 100%;
  min-height: var(--height-toolbar);
  padding: 0;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
  overflow: hidden;
}

.ui-tabs :deep([data-slot="list"]) {
  min-height: calc(var(--height-toolbar) - 2px);
  align-items: stretch;
}

.ui-tabs :deep([data-slot="trigger"]) {
  min-width: 112px;
  min-height: calc(var(--height-toolbar) - 2px);
  border-right: 1px solid var(--color-border);
  border-radius: 0;
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 700;
}

.ui-tabs :deep([data-slot="trigger"]:last-child) {
  border-right: 0;
}

.ui-tabs :deep([data-slot="trigger"][data-state="active"]) {
  background: var(--color-selected-surface);
  color: var(--color-accent-strong);
}

.ui-tabs :deep([data-slot="trigger"][data-state="active"]::after) {
  display: none;
}

.ui-tabs :deep([data-slot="trigger"]:hover:not(:disabled)) {
  background: var(--color-hover-surface);
  color: var(--color-text);
}

.ui-tabs :deep([data-slot="indicator"]) {
  display: none;
}
</style>
