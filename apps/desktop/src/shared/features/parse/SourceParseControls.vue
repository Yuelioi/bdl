<script setup lang="ts">
import { computed } from 'vue'
import { useParseStore } from '../../stores/parse'
import UiButton from '../../ui/Button.vue'
import ParseActivityStatus from '../../ui/ParseActivityStatus.vue'

const {
  library,
  hasMore,
  loading = false,
  parsingAll = false,
  stopping = false,
  sourceId,
} = defineProps<{
  library?: boolean
  hasMore: boolean
  loading?: boolean
  parsingAll?: boolean
  stopping?: boolean
  sourceId?: string
}>()
const parse = useParseStore()
const batchInProgress = computed(() => {
  if (library || parsingAll || !sourceId) return false
  const source = parse.sources[sourceId]?.source
  return source != null && parse.batchTargetBySource[sourceId] > source.loaded_count
})
const emit = defineEmits<{ parseBatch: []; parseAll: []; stop: [] }>()
</script>

<template>
  <div class="source-parse-controls">
    <template v-if="parsingAll || loading">
      <span v-if="!stopping" class="paced-parse-status" role="status">
        <ParseActivityStatus v-if="sourceId" :source-id="sourceId" />
        <span v-else>正在解析…</span>
      </span>
      <UiButton size="compact" variant="secondary" :disabled="stopping" @click="emit('stop')">
        {{ batchInProgress ? (stopping ? '正在暂停…' : '暂停解析') : (stopping ? '正在停止' : '停止解析') }}
      </UiButton>
    </template>
    <template v-else-if="hasMore">
      <UiButton size="compact" variant="secondary" @click="emit('parseBatch')">
        <UIcon name="i-tabler-list-details" class="size-4" aria-hidden="true" />
        {{ library || batchInProgress ? '继续解析' : '解析' }}
      </UiButton>
      <UiButton v-if="library" size="compact" variant="ghost" @click="emit('parseAll')">
        解析全部
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
  .source-parse-controls > .ui-button {
    min-height: 2.75rem;
  }

  .source-parse-controls {
    justify-content: flex-start;
    gap: var(--space-6);
  }
}
</style>
