<script setup lang="ts">
export interface TabItem {
  label: string
  value: string
  count?: number
}

const model = defineModel<string>({ required: true })
defineProps<{
  tabs: TabItem[]
}>()
</script>

<template>
  <div class="ui-tabs" role="tablist">
    <button
      v-for="tab in tabs"
      :key="tab.value"
      class="tab-button"
      :class="{ active: model === tab.value }"
      type="button"
      role="tab"
      :aria-selected="model === tab.value"
      @click="model = tab.value"
    >
      <span>{{ tab.label }}</span>
      <span v-if="tab.count !== undefined" class="tab-count">{{ tab.count }}</span>
    </button>
  </div>
</template>

<style scoped>
.ui-tabs {
  height: var(--height-toolbar);
  display: inline-flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-4);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

.tab-button {
  height: 30px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-8);
  border: 0;
  border-radius: var(--radius-6);
  background: transparent;
  color: var(--color-muted);
  padding: 0 var(--space-12);
  font-size: var(--font-13);
  font-weight: 650;
  white-space: nowrap;
}

.tab-button.active {
  background: var(--color-surface);
  color: var(--color-text);
  box-shadow: var(--shadow-panel);
}

.tab-count {
  min-width: 18px;
  height: 18px;
  display: inline-grid;
  place-items: center;
  border-radius: 999px;
  background: #dfe7e3;
  color: var(--color-muted);
  font-size: var(--font-12);
}
</style>
