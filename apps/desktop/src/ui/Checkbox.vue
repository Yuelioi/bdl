<script setup lang="ts">
const model = defineModel<boolean>({ default: false })
const { label, disabled = false } = defineProps<{
  label: string
  disabled?: boolean
}>()
</script>

<template>
  <UCheckbox
    v-model="model"
    class="ui-checkbox"
    color="primary"
    size="sm"
    icon="i-tabler-check"
    :label
    :disabled
    :ui="{
      root: 'items-center gap-2',
      container: 'h-4',
      base: 'size-4 rounded-[4px]',
      wrapper: 'ms-0',
      label: 'text-[13px] leading-5 font-semibold text-[var(--color-text)]',
      icon: 'size-3',
    }"
  />
</template>

<style scoped>
.ui-checkbox {
  min-height: 28px;
  align-items: center;
}
</style>
