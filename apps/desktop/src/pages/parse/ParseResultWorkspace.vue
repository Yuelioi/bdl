<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import UiEmptyState from '../../ui/EmptyState.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import SelectionActionBar from '../../ui/SelectionActionBar.vue';
import ParseResultTable from './ParseResultTable.vue';

import SourceLoadStatus from './SourceLoadStatus.vue';
import SourceParseControls from './SourceParseControls.vue';
import { useParseResultWorkspace } from './useParseResultWorkspace';
const { embedded = false } = defineProps<{ embedded?: boolean }>();
const emit = defineEmits<{ download: [] }>();
const {
  loadBatchSize,
  activeSource,
  selectedIds,
  selectedCount,
  totalPartCount,
  tableRows,
  sourceRequestLoading,
  pacedParsing,
  pacedWaiting,
  pacedStopping,
  activeLoading,
  activeError,
  hasMore,
  allRowsSelected,
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
  sourceKindLabels,
} = useParseResultWorkspace(() => emit('download'));
</script>
<template>
  <section
    v-if="activeSource"
    class="min-h-0 overflow-hidden"
    :class="embedded ? 'flex flex-1 flex-col gap-3' : 'panel'"
  >
    <div
      class="source-result-header grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-3 border-b border-(--color-border) pb-3 max-[840px]:grid-cols-1"
    >
      <div class="flex min-w-0 items-center gap-3">
        <button
          type="button"
          class="grid size-8 shrink-0 place-items-center rounded-md text-(--color-muted) hover:bg-(--color-panel) hover:text-(--color-text) disabled:cursor-not-allowed disabled:opacity-55"
          aria-label="返回解析首页"
          :disabled="activeLoading"
          @click="returnToSource"
        >
          <UIcon name="i-tabler-arrow-left" class="size-4" aria-hidden="true" />
        </button>
        <div class="flex min-w-0 items-center gap-2">
          <strong class="source-title truncate text-base text-(--color-text)" :title="activeSource.source.title">
            {{ activeSource.source.title }}
          </strong>
          <span class="shrink-0 text-[11px] font-bold text-(--color-muted)">
            {{ sourceKindLabels[activeSource.source.kind] }}
          </span>
        </div>
      </div>

      <div class="source-function-toolbar flex flex-wrap items-center justify-end gap-2 max-[840px]:justify-start">
        <SourceLoadStatus :loaded="activeSource.source.loaded_count" :total="activeSource.source.total_count" />
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
          v-if="!hasMore"
          size="compact"
          variant="secondary"
          :disabled="activeLoading || totalPartCount === 0"
          @click="downloadAllLoaded"
        >
          下载全部
        </UiButton>
        <UiButton size="compact" :disabled="!canCreateTasks" @click="emit('download')">{{ createTaskLabel }}</UiButton>
      </div>
    </div>

    <ParseResultTable
      v-if="tableRows.length"
      class="min-h-0 flex-1"
      :rows="tableRows"
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
    <UiInlineNotice v-if="activeError" tone="danger">{{ activeError }}</UiInlineNotice>

    <SelectionActionBar :selected-count="selectedCount" :total-count="totalPartCount">
      <template #selection>
        <UiButton
          size="compact"
          variant="secondary"
          :disabled="activeLoading || tableRows.length === 0"
          @click="toggleAllResults"
        >
          {{ allRowsSelected ? '取消全选' : '全选已加载' }}
        </UiButton>
        <UiButton
          v-if="selectedCount > 0 && !allRowsSelected"
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
