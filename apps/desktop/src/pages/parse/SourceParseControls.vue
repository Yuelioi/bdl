<script setup lang="ts">
import { computed } from 'vue'

import UiButton from '../../ui/Button.vue'
import UiIconButton from '../../ui/IconButton.vue'
import ParseBatchSizeSelect from './ParseBatchSizeSelect.vue'

const batchSize = defineModel<string>('batchSize', { default: '200' })
const {
  hasMore,
  loading = false,
  parsingAll = false,
  waiting = false,
  stopping = false,
} = defineProps<{
  hasMore: boolean
  loading?: boolean
  parsingAll?: boolean
  waiting?: boolean
  stopping?: boolean
}>()
const emit = defineEmits<{ parseBatch: []; parseAll: []; parseAndDownload: []; stop: [] }>()
const parseMenuItems = computed(() => [
  {
    label: '解析全部',
    icon: 'i-tabler-list-check',
    onSelect: () => emit('parseAll'),
  },
  {
    label: '解析后下载',
    icon: 'i-tabler-download',
    onSelect: () => emit('parseAndDownload'),
  },
])
</script>

<template>
  <div class="source-parse-controls">
    <template v-if="parsingAll">
      <span class="paced-parse-status" role="status">
        {{ stopping ? '正在停止…' : waiting ? '等待 3 秒后继续…' : '正在解析当前批…' }}
      </span>
      <UiButton size="compact" variant="secondary" :disabled="stopping" @click="emit('stop')">
        {{ stopping ? '正在停止' : '停止解析' }}
      </UiButton>
    </template>
    <template v-else-if="hasMore">
      <ParseBatchSizeSelect v-model="batchSize" :disabled="loading" />
      <div class="parse-action-split">
        <UiButton size="compact" variant="secondary" :disabled="loading" @click="emit('parseBatch')">
          {{ loading ? '解析中' : '解析' }}
        </UiButton>
        <UDropdownMenu
          :items="parseMenuItems"
          :disabled="loading"
          :content="{ align: 'end', sideOffset: 4, collisionPadding: 12 }"
          :ui="{ content: 'min-w-36' }"
        >
          <UiIconButton
            icon="chevron-down"
            label="更多解析方式"
            variant="secondary"
            size="compact"
            :disabled="loading"
          />
        </UDropdownMenu>
      </div>
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
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-8);
}

.parse-action-split {
  display: inline-flex;
  align-items: center;
}

.parse-action-split :deep(.ui-button) {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

.parse-action-split :deep(.ui-icon-button) {
  margin-left: -1px;
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}
</style>
