<script setup lang="ts">
import UiButton from '../../../shared/ui/Button.vue'
import ParseBatchSizeSelect from './ParseBatchSizeSelect.vue'
import ParseActivityStatus from '../../../shared/ui/ParseActivityStatus.vue'

const batchSize = defineModel<string>('batchSize', { default: '50' })
const {
  hasMore,
  loading = false,
  parsingAll = false,
  stopping = false,
  sourceId,
  showDownload = true,
} = defineProps<{
  hasMore: boolean
  loading?: boolean
  parsingAll?: boolean
  stopping?: boolean
  sourceId?: string
  showDownload?: boolean
}>()
const emit = defineEmits<{ parseBatch: []; parseAll: []; parseAndDownload: []; stop: [] }>()
</script>

<template>
  <div class="source-parse-controls">
    <template v-if="parsingAll || loading">
      <span class="paced-parse-status" role="status">
        <ParseActivityStatus v-if="sourceId" :source-id="sourceId" :stopping="stopping" />
        <span v-else>{{ stopping ? '正在停止…' : '正在解析…' }}</span>
      </span>
      <UiButton size="compact" variant="secondary" :disabled="stopping" @click="emit('stop')">
        {{ stopping ? '正在停止' : '停止解析' }}
      </UiButton>
    </template>
    <template v-else-if="hasMore">
      <ParseBatchSizeSelect v-model="batchSize" :disabled="loading" />
      <UiButton size="compact" variant="secondary" @click="emit('parseBatch')">
        <UIcon name="i-tabler-list-details" class="size-4" aria-hidden="true" />
        继续解析
      </UiButton>
      <UiButton size="compact" variant="ghost" @click="emit('parseAll')">
        解析全部
      </UiButton>
      <UiButton v-if="showDownload" size="compact" variant="ghost" @click="emit('parseAndDownload')">
        <UIcon name="i-tabler-download" class="size-4" aria-hidden="true" />
        后台下载全部
      </UiButton>
    </template>
  </div>
</template>

<style scoped>
.paced-parse-status {
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  color: var(--color-muted);
  font-size: var(--font-12);
  white-space: nowrap;
}

.source-parse-controls {
  min-width: 0;
  min-height: 28px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-8);
}

@media (width <= 700px) {
  .source-parse-controls {
    justify-content: flex-start;
    gap: var(--space-6);
  }
}
</style>
