<script setup lang="ts">
import { computed } from 'vue';
import CollectionTree from '../../../shared/ui/CollectionTree.vue';
import { hasCollectionSections } from '../../../shared/features/parse/collectionTree';
import UiEmptyState from '../../../shared/ui/EmptyState.vue';
import UiInlineNotice from '../../../shared/ui/InlineNotice.vue';
import SourceMediaList from './SourceMediaList.vue';

import SourceLoadStatus from './SourceLoadStatus.vue';
import ParseSourceSummary from '../../../shared/features/parse/ParseSourceSummary.vue';
import ParseResultFilters from '../../../shared/features/parse/ParseResultFilters.vue';
import SourceParseControls from '../../../shared/features/parse/SourceParseControls.vue';
import { useParseResultWorkspace } from '../../../shared/features/parse/useParseResultWorkspace';
const {
  parse,
  activeSource,
  selectedIds,
  tableRows,
  query,
  sort,
  selectRange,
  sourceRequestLoading,
  pacedParsing,
  pacedStopping,
  activeLoading,
  activeError,
  activeErrorDetail,
  hasMore,
  toggleAllResults,
  loadMore,
  stopParsing,
  toggleNode,
} = useParseResultWorkspace();
const showLoadToolbar = computed(() => hasMore.value || activeLoading.value);
</script>
<template>
  <section v-if="activeSource" class="flex min-h-0 flex-1 flex-col gap-3 overflow-hidden">
    <header class="source-result-header flex min-w-0 flex-wrap items-center justify-between gap-3 border-b border-(--color-border) pb-4">
      <div class="source-result-summary">
        <ParseSourceSummary :source="activeSource" :hide-count="showLoadToolbar" />
      </div>
      <div v-if="showLoadToolbar" class="source-function-toolbar" role="group" aria-label="解析进度与操作">
        <SourceLoadStatus
          :loaded="activeSource.source.loaded_count"
          :total="activeSource.source.total_count"
          :has-more="hasMore"
        />
        <SourceParseControls
          :source-id="activeSource.source.id"
          :has-more="hasMore"
          :loading="sourceRequestLoading"
          :parsing-all="pacedParsing"
          :stopping="pacedStopping"
          @parse-batch="loadMore"
          @stop="stopParsing"
        />
      </div>
    </header>

    <ParseResultFilters
      v-model:query="query"
      v-model:sort="sort"
      :list-label="activeSource.source.kind === 'uploader' ? '投稿' : ['collection', 'series'].includes(activeSource.source.kind) ? '视频' : undefined"
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
      hide-selection-count
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
.source-result-summary {
  min-width: 0;
  flex: 1 1 15rem;
}

.source-function-toolbar {
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-8);
}

@media (width <= 700px) {
  .source-result-header {
    gap: var(--space-8);
    padding-bottom: var(--space-10);
  }
}
</style>
