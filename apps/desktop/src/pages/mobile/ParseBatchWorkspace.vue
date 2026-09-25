<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import UiCheckbox from '../../ui/Checkbox.vue';
import UiTextField from '../../ui/TextField.vue';

import { useParseBatchWorkspace } from '../parse/useParseBatchWorkspace';
const { embedded = false } = defineProps<{ embedded?: boolean }>();
const emit = defineEmits<{ download: [] }>();
const {
  parse,
  query,
  entries,
  selectedSet,
  selectedCount,
  allSelected,
  loading,
  toggleAll,

  returnToSource,
} = useParseBatchWorkspace();
</script>
<template>
  <section class="mobile-batch" :class="{ embedded }">
    <header class="mobile-detail-heading">
      <button
        class="mobile-icon-button"
        type="button"
        aria-label="返回解析首页"
        :disabled="loading"
        @click="returnToSource"
      >
        <UIcon name="i-tabler-arrow-left" />
      </button>
      <div class="mobile-detail-copy">
        <h2>批量视频</h2>
        <p>{{ parse.batchEntries.length }} 个链接</p>
      </div>
    </header>
    <UiTextField
      v-if="parse.batchEntries.length > 8 || query"
      v-model="query"
      label="搜索视频"
      placeholder="标题或链接"
      :disabled="loading"
    />
    <div class="mobile-selection-bar">
      <UiCheckbox
        :model-value="allSelected"
        label="全选全部"
        :disabled="loading"
        @update:model-value="toggleAll"
      /><span>点选视频</span>
    </div>
    <div class="batch-list" role="list" aria-label="批量解析结果">
      <div v-for="entry in entries" :key="entry.id" class="batch-row" role="listitem">
        <UiCheckbox
          :model-value="selectedSet.has(entry.id)"
          :label="`选择 ${entry.title}`"
          compact
          :disabled="loading"
          @update:model-value="parse.toggleBatchEntry(entry.id)"
        /><button class="batch-copy" type="button" :disabled="loading" @click="parse.toggleBatchEntry(entry.id)">
          <strong>{{ entry.title }}</strong
          ><span>{{ entry.input }}</span></button
        ><button
          class="mobile-icon-button"
          type="button"
          aria-label="移除这个链接"
          :disabled="loading"
          @click="parse.removeBatchEntry(entry.id)"
        >
          <UIcon name="i-tabler-x" />
        </button>
      </div>
      <p v-if="!entries.length">没有匹配的视频</p>
    </div>
    <footer class="mobile-download-footer">
      <span
        >已选 <strong>{{ selectedCount }}</strong> 项</span
      ><UiButton :disabled="!selectedCount || loading" @click="emit('download')"
        >下载所选 ({{ selectedCount }})</UiButton
      >
    </footer>
  </section>
</template>
<style scoped>
.mobile-batch {
  min-height: 0;
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 8px;
}

.batch-list {
  min-height: 0;
  flex: 1;
  overflow-y: auto;
}

.batch-row {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 88px;
  padding: 12px 0;
  border-bottom: 1px solid var(--color-border);
}

.batch-copy {
  display: grid;
  gap: 8px;
  flex: 1;
  min-width: 0;
  border: 0;
  background: transparent;
  text-align: left;
  color: var(--color-text);
}

.batch-copy strong {
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  font-size: var(--mobile-font-item-title);
  font-weight: var(--mobile-weight-item);
  line-height: 1.5;
}

.batch-copy span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--mobile-font-body);
  color: var(--color-muted);
}
</style>
