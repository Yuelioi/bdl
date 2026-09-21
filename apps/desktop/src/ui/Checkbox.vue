<script setup lang="ts">
const model = defineModel<boolean | 'indeterminate'>({ default: false })
const { label, disabled = false, compact = false } = defineProps<{
  label: string
  disabled?: boolean
  compact?: boolean
}>()
</script>

<template>
  <UCheckbox
    v-model="model"
    class="ui-checkbox"
    :class="{ compact }"
    color="primary"
    size="sm"
    icon="i-tabler-check"
    :label
    :disabled
    :ui="{
      root: compact ? 'items-center gap-0' : 'items-center gap-2',
      container: 'h-4',
      base: 'ui-checkbox-box size-4 rounded-[4px]',
      wrapper: 'ms-0',
      label: compact ? 'sr-only' : 'text-[13px] leading-5 font-semibold text-(--color-text)',
      icon: 'size-3',
    }"
  />
</template>

<style scoped>
.ui-checkbox {
  min-height: 28px;
  align-items: center;
}

.ui-checkbox.compact {
  min-height: 18px;
}

.ui-checkbox :deep(.ui-checkbox-box) {
  border: 1px solid var(--color-border-strong, var(--color-border));
  background: var(--color-surface);
  box-shadow: inset 0 0 0 1px var(--color-border-strong, var(--color-border));
}

.ui-checkbox :deep(.ui-checkbox-box[data-state='checked']),
.ui-checkbox :deep(.ui-checkbox-box[data-state='indeterminate']) {
  border-color: var(--color-accent);
  background: var(--color-accent);
  color: var(--color-on-accent);
  box-shadow: none;
}
</style>
