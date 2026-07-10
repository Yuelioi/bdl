<script setup lang="ts">
export interface DefinitionItem {
  label: string
  value: string | number
}

defineProps<{
  items: DefinitionItem[]
}>()
</script>

<template>
  <dl class="ui-definition-list">
    <div v-for="item in items" :key="item.label">
      <dt>{{ item.label }}</dt>
      <dd>{{ item.value }}</dd>
    </div>
  </dl>
</template>

<style scoped>
.ui-definition-list {
  display: grid;
  gap: var(--space-12);
}

.ui-definition-list div {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

dt {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

dd {
  min-width: 0;
  margin: 0;
  overflow-wrap: anywhere;
  color: var(--color-text);
  font-size: var(--font-12);
  line-height: 1.55;
}
</style>
