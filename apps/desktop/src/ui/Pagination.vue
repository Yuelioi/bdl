<script setup lang="ts">
const {
  page,
  total,
  itemsPerPage = 20,
  disabled = false,
  label = '分页',
} = defineProps<{
  page: number
  total: number
  itemsPerPage?: number
  disabled?: boolean
  label?: string
}>()

const emit = defineEmits<{ 'update:page': [page: number] }>()
</script>

<template>
  <nav :aria-label="label">
    <UPagination
      :page
      :total
      :items-per-page="itemsPerPage"
      :disabled
      :sibling-count="1"
      show-edges
      size="xs"
      color="neutral"
      variant="ghost"
      active-color="primary"
      active-variant="soft"
      @update:page="emit('update:page', $event)"
    />
  </nav>
</template>
