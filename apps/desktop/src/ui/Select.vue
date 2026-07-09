<script setup lang="ts">
export interface SelectOption {
  label: string
  value: string
}

const model = defineModel<string>({ default: '' })
const { label, options, disabled = false } = defineProps<{
  label: string
  options: SelectOption[]
  disabled?: boolean
}>()
</script>

<template>
  <label class="ui-field">
    <span>{{ label }}</span>
    <USelect
      v-model="model"
      :items="options"
      value-key="value"
      label-key="label"
      color="neutral"
      variant="outline"
      size="md"
      :disabled
      trailing-icon="i-tabler-chevron-down"
      selected-icon="i-tabler-check"
      :portal="false"
      :content="{ side: 'bottom', sideOffset: 4, collisionPadding: 12, position: 'popper', avoidCollisions: false }"
      :ui="{ content: 'z-50 !max-h-44 shadow-lg', item: 'font-medium' }"
      class="ui-select"
    />
  </label>
</template>

<style scoped>
.ui-field {
  display: grid;
  gap: var(--space-8);
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 600;
}

.ui-select {
  width: 100%;
}

.ui-field :deep(button) {
  min-height: var(--height-input);
  font-weight: 650;
}
</style>
