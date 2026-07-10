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
      :portal="true"
      :content="{ side: 'bottom', align: 'start', sideOffset: 4, collisionPadding: 12, position: 'popper' }"
      :ui="{
        base: 'h-9 min-h-9 py-0',
        trailingIcon: 'size-4',
        content: 'z-50 !max-h-44 shadow-md',
        item: 'font-medium',
      }"
      class="ui-select"
    />
  </label>
</template>

<style scoped>
.ui-field {
  display: grid;
  grid-template-rows: 18px var(--height-input);
  gap: var(--space-6);
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 600;
}

.ui-field > span {
  display: flex;
  align-items: center;
  line-height: 18px;
}

.ui-select {
  width: 100%;
}

.ui-field :deep(button) {
  height: var(--height-input);
  min-height: var(--height-input);
  font-weight: 650;
}
</style>
