<script setup lang="ts">
import { ref } from 'vue';
import CollectionTree from '../../../shared/ui/CollectionTree.vue';
import { hasCollectionSections } from '../../../shared/features/parse/collectionTree';
import UiButton from '../../../shared/ui/Button.vue';
import UiIconButton from '../../../shared/ui/IconButton.vue';
import UiCheckbox from '../../../shared/ui/Checkbox.vue';
import UiEmptyState from '../../../shared/ui/EmptyState.vue';
import UiInlineNotice from '../../../shared/ui/InlineNotice.vue';
import ParseSourceSummary from '../../../shared/features/parse/ParseSourceSummary.vue';
import ParseResultFilters from '../../../shared/features/parse/ParseResultFilters.vue';
import ParseResultListMobile from './ParseResultListMobile.vue';
import MobileSourceMenu from '../MobileSourceMenu.vue';
import { useParseResultWorkspace } from '../../../shared/features/parse/useParseResultWorkspace';
const { embedded = false, inlineDownload = false } = defineProps<{ embedded?: boolean; inlineDownload?: boolean }>();
const emit = defineEmits<{ download: [] }>();
const {
  parse,
  downloadAllLoaded,
  loadBatchSize,
  activeSource,
  selectedIds,
  selectedCount,
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
  allRowsSelected,
  canCreateTasks,
  toggleAllResults,
  loadMore,
  parseAll,
  stopParsing,
  parseAndDownload,
  returnToSource,
  toggleNode,
} = useParseResultWorkspace(() => emit('download'));
const toolsOpen = ref(false);
</script>

<template>
  <section v-if="activeSource" class="mobile-result-workspace" :class="{ embedded }">
    <header class="result-source-heading">
      <ParseSourceSummary :source="activeSource" />
      <UiIconButton
        icon="x"
        label="返回解析首页"
        variant="ghost"
        size="compact"
        :disabled="activeLoading"
        @click="returnToSource"
      />
    </header>
    <ParseResultFilters
      v-model:query="query"
      v-model:sort="sort"
      :loaded="tableRows.length"
      :grouped="hasCollectionSections(activeSource)"
      :disabled="activeLoading"
      :apply-range="selectRange"
    />
    <div class="result-selection-toolbar">
      <UiCheckbox
        :model-value="allRowsSelected ? true : selectedCount > 0 ? 'indeterminate' : false"
        label="全选已加载"
        :disabled="activeLoading"
        @update:model-value="toggleAllResults"
      />
      <span>已选 {{ selectedCount }} 项</span>
      <UButton
        class="result-content-actions"
        variant="link"
        color="neutral"
        size="xs"
        trailing-icon="i-tabler-chevron-down"
        :disabled="activeLoading"
        aria-label="内容操作"
        @click="toolsOpen = true"
        >内容操作</UButton
      >
    </div>
    <MobileSourceMenu
      v-model:open="toolsOpen"
      v-model:batch-size="loadBatchSize"
      :has-more="hasMore"
      :busy="sourceRequestLoading"
      :parsing="pacedParsing"
      :stopping="pacedStopping"
      @load="loadMore"
      @all="parseAll"
      @download-all="hasMore ? parseAndDownload() : downloadAllLoaded()"
      @stop="stopParsing"
    />
    <CollectionTree
      v-if="hasCollectionSections(activeSource)"
      :source="activeSource"
      :selected-ids="selectedIds"
      :disabled="activeLoading"
      :query="query"
      @toggle="parse.togglePartIds(activeSource.source.id, $event)"
    />
    <ParseResultListMobile
      v-else-if="tableRows.length"
      managing
      :rows="tableRows"
      :source="activeSource"
      :selected-ids="selectedIds"
      :disabled="activeLoading"
      @toggle="toggleNode"
    >
      <UButton
        variant="ghost"
        color="neutral"
        :ui="{ base: 'p-0', label: 'contents' }"
        v-if="hasMore || pacedParsing"
        type="button"
        class="mobile-load-more"
        :disabled="activeLoading"
        @click="loadMore"
      >
        {{ pacedParsing ? '正在加载更多内容…' : sourceRequestLoading ? '加载中…' : '继续加载内容' }}
      </UButton>
    </ParseResultListMobile>
    <UiEmptyState
      v-else
      :title="query ? '没有匹配的视频' : '没有可选择内容'"
      :description="query ? '修改关键词或清除搜索。' : '当前来源没有可下载的视频或分集。'"
      icon="i-tabler-folder-open"
      compact
      embedded
    />
    <UiButton
      v-if="hasCollectionSections(activeSource) && hasMore"
      variant="secondary"
      :disabled="activeLoading"
      @click="loadMore"
      >继续加载内容</UiButton
    >
    <UiInlineNotice v-if="activeError" tone="danger" :detail="activeErrorDetail ?? undefined">{{
      activeError
    }}</UiInlineNotice>
    <footer v-if="!inlineDownload" class="mobile-download-footer mobile-download-bar">
      <span
        >已选 <strong>{{ selectedCount }}</strong> 项</span
      >
      <UiButton size="compact" :disabled="!canCreateTasks" @click="emit('download')"
        >下载所选 ({{ selectedCount }})</UiButton
      >
    </footer>
  </section>
</template>

<style scoped>
.mobile-result-workspace {
  min-height: 0;
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 0.375rem;
  overflow: hidden;
}

.result-source-heading {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  flex-shrink: 0;
}

.result-source-heading > :first-child {
  flex: 1;
  min-width: 0;
}

.result-selection-toolbar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
  min-height: 1.875rem;
  color: var(--color-muted);
  font-size: var(--font-12);
}

.result-content-actions {
  margin-left: auto;
}
</style>
