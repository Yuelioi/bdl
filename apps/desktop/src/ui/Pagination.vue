<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import UiButton from './Button.vue'

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
const pageCount = computed(() => Math.max(1, Math.ceil(total / itemsPerPage)))
const targetPage = ref(String(page))
watch(() => page, (value) => {
  targetPage.value = String(value)
})
const validTarget = computed(() =>
  /^\d+$/.test(targetPage.value) && Number(targetPage.value) >= 1 && Number(targetPage.value) <= pageCount.value,
)
const jump = () => {
  if (!disabled && validTarget.value && Number(targetPage.value) !== page) emit('update:page', Number(targetPage.value))
}
</script>

<template>
  <nav class="ui-pagination" :aria-label="label">
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
    <form class="page-jump" @submit.prevent="jump">
      <span>跳至</span>
      <input
        v-model="targetPage"
        class="ui-native-control page-jump-input"
        type="text"
        inputmode="numeric"
        :aria-label="`${label}：跳转页码`"
        :title="`输入 1–${pageCount} 页，按回车跳转`"
        :disabled
      />
      <span>页</span>
      <UiButton type="submit" size="compact" variant="secondary" :disabled="disabled || !validTarget">跳转</UiButton>
    </form>
  </nav>
</template>

<style scoped>
.ui-pagination,
.page-jump {
  display: flex;
  align-items: center;
  gap: var(--space-8);
}

.ui-pagination {
  flex-wrap: wrap;
}

.page-jump {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.page-jump-input {
  width: 58px;
  min-height: 28px;
  padding: var(--space-4) var(--space-6);
  text-align: center;
}

.ui-pagination :deep([aria-current='page']) {
  color: #fff;
  background: #ef5b7d;
  background: var(--color-accent);
}
</style>
