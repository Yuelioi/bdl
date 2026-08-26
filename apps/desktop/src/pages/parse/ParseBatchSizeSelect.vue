<script setup lang="ts">
const model = defineModel<string>({ default: '200' })
const { disabled = false } = defineProps<{ disabled?: boolean }>()

const options = [
  { label: '每批 200', value: '200' },
  { label: '每批 500', value: '500' },
  { label: '每批 1000', value: '1000' },
]
</script>

<template>
  <USelect
    v-model="model"
    :items="options"
    value-key="value"
    label-key="label"
    size="xs"
    color="neutral"
    variant="outline"
    trailing-icon="i-tabler-chevron-down"
    :disabled
    :portal="true"
    aria-label="每批解析数量"
    :content="{ side: 'bottom', align: 'end', sideOffset: 4, collisionPadding: 12, position: 'popper' }"
    :ui="{ base: 'h-7 min-h-7 w-25 py-0', content: 'z-50 shadow-md', item: 'font-medium' }"
  />
</template>
