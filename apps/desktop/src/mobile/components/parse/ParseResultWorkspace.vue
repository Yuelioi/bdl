<script setup lang="ts">
import { computed, ref } from 'vue';
import { useMobileBack } from '../../app/useMobileBack';
import CollectionTree from '../../../shared/ui/CollectionTree.vue';
import { hasCollectionSections } from '../../../shared/features/parse/collectionTree';
import UiButton from '../../../shared/ui/Button.vue';
import UiCheckbox from '../../../shared/ui/Checkbox.vue';
import UiEmptyState from '../../../shared/ui/EmptyState.vue';
import UiInlineNotice from '../../../shared/ui/InlineNotice.vue';
import ParseSourceSummary from '../../../shared/features/parse/ParseSourceSummary.vue';
import ParseResultFilters from '../../../shared/features/parse/ParseResultFilters.vue';
import ParseResultListMobile from './ParseResultListMobile.vue';
import SourceParseControls from '../../../shared/features/parse/SourceParseControls.vue';
import { useParseResultWorkspace } from '../../../shared/features/parse/useParseResultWorkspace';
const { embedded = false, inlineDownload = false } = defineProps<{ embedded?: boolean; inlineDownload?: boolean }>();
const emit = defineEmits<{ download: []; openCollection: [] }>();
const {
  parse,
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
  pacedStopping,
  activeLoading: sourceLoading,
  activeError,
  activeErrorDetail,
  hasMore,
  allRowsSelected,
  canCreateTasks,
  toggleAllResults,
  loadMore,
  stopParsing,
  toggleNode,
} = useParseResultWorkspace();
const activeLoading = computed(() => sourceLoading.value || Boolean(parse.loadingBySource.__create__));
const selectAllLabel = computed(() => query.value.trim() ? '全选搜索结果' : '全选已加载');
const searching = ref(false);
useMobileBack('parse', () => {
  if (!searching.value) return false;
  searching.value = false;
  query.value = '';
  return true;
}, 1);
</script>

<template>
  <section v-if="activeSource" class="mobile-result-workspace" :class="{ embedded }">
    <div class="result-source-row">
      <div class="result-source"><ParseSourceSummary :source="activeSource" wrap-metadata /></div>
      <UiButton
        v-if="activeSource.source.kind === 'video'"
        class="collection-link"
        variant="ghost"
        :disabled="activeLoading"
        @click="emit('openCollection')"
      >
        所属合集
        <UIcon name="i-tabler-chevron-right" class="size-4" aria-hidden="true" />
      </UiButton>
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
    <ParseResultFilters
      v-if="totalPartCount > 1 || hasMore"
      v-model:query="query"
      v-model:sort="sort"
      v-model:searching="searching"
      mobile
      :grouped="hasCollectionSections(activeSource)"
      :disabled="activeLoading"
      :apply-range="selectRange"
    >
      <template #selection>
        <UiCheckbox
          :key="selectAllLabel"
          :model-value="allRowsSelected ? true : selectedCount > 0 ? 'indeterminate' : false"
          :label="selectAllLabel"
          :disabled="activeLoading || !tableRows.length"
          @update:model-value="toggleAllResults"
        />
      </template>
    </ParseResultFilters>
    <CollectionTree
      v-if="hasCollectionSections(activeSource)"
      class="result-collection"
      :source="activeSource"
      :selected-ids="selectedIds"
      :disabled="activeLoading"
      :query="query"
      @toggle="parse.togglePartIds(activeSource.source.id, $event)"
    />
    <ParseResultListMobile
      v-else-if="tableRows.length"
      roomy
      managing
      :rows="tableRows"
      :source="activeSource"
      :selected-ids="selectedIds"
      :disabled="activeLoading"
      @toggle="toggleNode"
    />
    <UiEmptyState
      v-else
      :title="query ? '没有匹配的视频' : '没有可选择内容'"
      :description="query ? '修改关键词或清除搜索。' : '当前来源没有可下载的视频或分集。'"
      icon="i-tabler-folder-open"
      compact
      embedded
    />
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
  gap: var(--space-12);
  overflow: hidden;
}

.result-source-row {
  display: flex;
  align-items: center;
  gap: var(--space-8);
  flex-shrink: 0;
  padding-block: var(--space-4);
}

.result-source {
  flex: 1;
  min-width: 0;
}

.collection-link {
  flex-shrink: 0;
  min-height: 2.75rem;
  padding-inline: var(--space-4);
}

@media (orientation: landscape) and (height <= 480px) {
  .mobile-result-workspace {
    --result-list-flex: 0 0 auto;
    --result-list-overflow: visible;

    gap: var(--space-6);
    overflow: hidden auto;
  }

  .result-source-row {
    padding-block: 0;
  }

  .result-collection {
    flex: 0 0 auto;
  }
}
</style>
