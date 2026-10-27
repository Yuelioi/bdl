<script setup lang="ts">
import CollectionTree from '../../../shared/ui/CollectionTree.vue';
import { hasCollectionSections } from '../../../shared/features/parse/collectionTree';
import UiButton from '../../../shared/ui/Button.vue';
import UiEmptyState from '../../../shared/ui/EmptyState.vue';
import UiInlineNotice from '../../../shared/ui/InlineNotice.vue';
import SelectionActionBar from '../../../shared/ui/SelectionActionBar.vue';
import SourceMediaList from './SourceMediaList.vue';

import SourceLoadStatus from './SourceLoadStatus.vue';
import ParseSourceSummary from '../../../shared/features/parse/ParseSourceSummary.vue';
import ParseResultFilters from '../../../shared/features/parse/ParseResultFilters.vue';
import SourceParseControls from './SourceParseControls.vue';
import { useParseResultWorkspace } from '../../../shared/features/parse/useParseResultWorkspace';
const { embedded = false, inlineDownload = false } = defineProps<{ embedded?: boolean; inlineDownload?: boolean }>();
const emit = defineEmits<{ download: [] }>();
const {
  parse,
  loadBatchSize,
  activeSource,
  selectedIds,
  selectedCount,
  totalPartCount,
  tableRows,
  query,
  sort,
  selectRange,
  sourceRequestLoading,
  pacedParsing,
  pacedWaiting,
  pacedStopping,
  activeLoading,
  activeError,
  activeErrorDetail,
  hasMore,
  canCreateTasks,
  createTaskLabel,
  toggleAllResults,
  clearSelection,
  loadMore,
  parseAll,
  stopParsing,
  downloadAllLoaded,
  parseAndDownload,
  returnToSource,
  toggleNode,
} = useParseResultWorkspace(() => emit('download'));
</script>
<template>
  <section
    v-if="activeSource"
    class="min-h-0 overflow-hidden"
    :class="embedded ? 'flex flex-1 flex-col gap-3' : 'panel'"
  >
    <div class="source-result-header flex min-w-0 flex-wrap items-center gap-3 border-b border-(--color-border) pb-4">
      <div class="flex min-w-0 flex-1 items-center gap-3">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ base: 'p-0', label: 'contents' }"
          type="button"
          class="order-last ml-auto grid size-7 shrink-0 place-items-center rounded-md text-(--color-muted) hover:bg-(--color-panel) hover:text-(--color-text) disabled:cursor-not-allowed disabled:opacity-55"
          aria-label="返回解析首页"
          :disabled="activeLoading"
          @click="returnToSource"
        >
          <UIcon name="i-tabler-arrow-left" class="size-4" aria-hidden="true" />
        </UButton>
        <ParseSourceSummary :source="activeSource" />
      </div>

      <div class="source-function-toolbar flex flex-wrap items-center justify-end gap-2 max-[840px]:justify-start">
        <SourceLoadStatus
          v-if="hasMore || pacedParsing"
          :loaded="activeSource.source.loaded_count"
          :total="activeSource.source.total_count"
        />
        <SourceParseControls
          :source-id="activeSource.source.id"
          v-if="hasMore || pacedParsing"
          v-model:batch-size="loadBatchSize"
          :has-more="hasMore"
          :loading="sourceRequestLoading"
          :parsing-all="pacedParsing"
          :waiting="pacedWaiting"
          :stopping="pacedStopping"
          @parse-batch="loadMore"
          @parse-all="parseAll"
          @parse-and-download="parseAndDownload"
          @stop="stopParsing"
        />
        <UiButton
          v-if="!hasMore && !inlineDownload"
          size="compact"
          variant="secondary"
          :disabled="activeLoading || totalPartCount === 0"
          @click="downloadAllLoaded"
        >
          下载全部
        </UiButton>
        <UiButton v-if="!inlineDownload" size="compact" :disabled="!canCreateTasks" @click="emit('download')">{{
          createTaskLabel
        }}</UiButton>
      </div>
    </div>

    <ParseResultFilters
      v-model:query="query"
      v-model:sort="sort"
      :loaded="tableRows.length"
      :grouped="hasCollectionSections(activeSource)"
      :disabled="activeLoading"
      :apply-range="selectRange"
    />
    <CollectionTree
      v-if="hasCollectionSections(activeSource)"
      :source="activeSource"
      :selected-ids="selectedIds"
      :disabled="activeLoading"
      :query="query"
      @toggle="parse.togglePartIds(activeSource.source.id, $event)"
    />
    <SourceMediaList
      v-else-if="tableRows.length"
      compact
      class="min-h-0 flex-1"
      :rows="tableRows"
      :source="activeSource"
      :selected-ids="selectedIds"
      :disabled="activeLoading"
      @toggle="toggleNode"
      @toggle-all="toggleAllResults"
    />
    <UiEmptyState
      v-else
      title="没有可选择内容"
      description="当前来源没有可下载的视频或分集。"
      icon="i-tabler-folder-open"
      layout="stacked"
      compact
      embedded
    />
    <UiInlineNotice v-if="activeError" tone="danger" :detail="activeErrorDetail ?? undefined">{{
      activeError
    }}</UiInlineNotice>

    <SelectionActionBar v-if="!inlineDownload" :selected-count="selectedCount" :total-count="totalPartCount">
      <template #selection>
        <UiButton
          v-if="selectedCount > 0"
          size="compact"
          variant="ghost"
          :disabled="activeLoading"
          @click="clearSelection"
        >
          取消选择
        </UiButton>
      </template>
    </SelectionActionBar>
  </section>

  <UiEmptyState
    v-else
    title="从一个来源开始"
    description="输入视频、合集、收藏夹、UP 主空间、番剧或课程链接。"
    icon="i-tabler-link"
    layout="stacked"
  >
    <template #detail>解析结果只保留在当前会话；确认选择后才会创建下载任务。</template>
  </UiEmptyState>
</template>

<style scoped>
@media (width <= 700px) {
  .source-result-header {
    gap: var(--space-8);
    padding-bottom: var(--space-10);
  }

  .source-function-toolbar {
    width: 100%;
    justify-content: flex-start;
    gap: var(--space-6);
  }
}
</style>
