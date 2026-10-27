<script setup lang="ts">
import UiButton from '../../../shared/ui/Button.vue';
import UiCheckbox from '../../../shared/ui/Checkbox.vue';
import UiTextField from '../../../shared/ui/TextField.vue';

import { useParseBatchWorkspace } from '../../../shared/features/parse/useParseBatchWorkspace';
const { embedded = false, inlineDownload = false } = defineProps<{ embedded?: boolean; inlineDownload?: boolean }>();
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
      <UButton
        variant="ghost"
        color="neutral"
        :ui="{ base: 'p-0', label: 'contents' }"
        class="mobile-icon-button"
        type="button"
        aria-label="返回解析首页"
        :disabled="loading"
        @click="returnToSource"
      >
        <UIcon name="i-tabler-arrow-left" />
      </UButton>
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
        /><UButton
          variant="ghost"
          color="neutral"
          :ui="{ base: 'p-0', label: 'contents' }"
          class="batch-copy"
          type="button"
          :disabled="loading"
          @click="parse.toggleBatchEntry(entry.id)"
        >
          <strong>{{ entry.title }}</strong
          ><span>{{ entry.input }}</span></UButton
        ><UButton
          variant="ghost"
          color="neutral"
          :ui="{ base: 'p-0', label: 'contents' }"
          class="mobile-icon-button"
          type="button"
          aria-label="移除这个链接"
          :disabled="loading"
          @click="parse.removeBatchEntry(entry.id)"
        >
          <UIcon name="i-tabler-x" />
        </UButton>
      </div>
      <p v-if="!entries.length">没有匹配的视频</p>
    </div>
    <footer v-if="!inlineDownload" class="mobile-download-footer">
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
  gap: 0.5rem;
}

.batch-list {
  min-height: 0;
  flex: 1;
  overflow-y: auto;
}

.batch-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-height: 3.5rem;
  padding: 0.375rem 0;
  border-bottom: 0.0625rem solid var(--color-border);
}

.batch-copy {
  display: grid;
  gap: 0.125rem;
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
  font-weight: 500;
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
