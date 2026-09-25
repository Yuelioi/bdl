<script setup lang="ts">
const model = defineModel<string>({ required: true });
defineProps<{ tabs: Array<{ value: string; label: string; count?: number }> }>();
</script>
<template>
  <div class="mobile-tabs" role="tablist">
    <button
      v-for="tab in tabs"
      :key="tab.value"
      type="button"
      role="tab"
      :aria-selected="model === tab.value"
      @click="model = tab.value"
    >
      <span>{{ tab.label }}</span
      ><span v-if="tab.count !== undefined" class="tab-count">{{ tab.count }}</span>
    </button>
  </div>
</template>
<style scoped>
.mobile-tabs {
  display: flex;
  width: 100%;
  min-width: 0;
  border-bottom: 1px solid var(--color-border);
}

button {
  position: relative;
  flex: 1 1 0;
  min-width: 0;
  min-height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border: 0;
  background: transparent;
  color: var(--color-muted);
  font-size: var(--mobile-font-item-title);
  white-space: nowrap;
  padding: 5px 2px;
}

button[aria-selected='true'] {
  color: var(--color-accent-strong);
  font-weight: 700;
}

button[aria-selected='true']::after {
  content: '';
  position: absolute;
  height: 3px;
  width: 36px;
  bottom: 0;
  left: calc(50% - 18px);
  background: var(--color-accent);
  border-radius: 2px;
}

.tab-count {
  display: inline;
  flex: 0 0 auto;
  font-size: var(--mobile-font-body);
  font-weight: 400;
  font-variant-numeric: tabular-nums;
  color: var(--color-muted);
  line-height: 1.4;
}

button[aria-selected='true'] .tab-count {
  color: currentcolor;
}

button:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: -3px;
}
</style>
