<script setup lang="ts">
import LibrarySourceSummary from '../../../shared/features/library/LibrarySourceSummary.vue';
import CollectionTree from '../../../shared/ui/CollectionTree.vue';
import { hasCollectionSections } from '../../../shared/features/parse/collectionTree';
import ParseResultListMobile from '../parse/ParseResultListMobile.vue';
import MobilePagination from '../MobilePagination.vue';
import UiButton from '../../../shared/ui/Button.vue';
import { computed, ref } from 'vue';
import { useMobileBack } from '../../app/useMobileBack';
import MobileListActions from '../MobileListActions.vue';
import MobileListSearch from '../MobileListSearch.vue';
import UiCheckbox from '../../../shared/ui/Checkbox.vue';
import UiEmptyState from '../../../shared/ui/EmptyState.vue';
import UiInlineNotice from '../../../shared/ui/InlineNotice.vue';

import MobileSourceMenu from '../MobileSourceMenu.vue';
import MobileDownloadScope from '../MobileDownloadScope.vue';

import { useLibraryFolderDetail } from '../../../shared/features/library/useLibraryFolderDetail';
import type { LibraryContentSource } from '../../../shared/api/dto';
const props = defineProps<{
  folder: LibraryContentSource;
  categoryLabel?: string;
  embedded?: boolean;
  loadingInitial?: boolean;
}>();
const emit = defineEmits<{ back: []; download: []; downloadAll: [sourceId: string] }>();
const {
  parse,
  pageSize,
  currentPage,
  loadBatchSize,
  source,

  totalCount,
  pageItems,
  selectedCount,
  sourceRequestLoading,
  pacedParsing,

  pacedStopping,
  loading,
  activeError,
  activeErrorDetail,
  hasMore,
  itemOwnerName,
  currentPageSelected,
  toggleItem,
  toggleCurrentPageSelection,
  clearSelection,
  downloadSelected,
  parseMore,
  parseAll,
  stopParsing,
  goToPage,
} = useLibraryFolderDetail(props, () => emit('download'));
const query = ref('');
const searching = ref(false);
const searchOpen = computed({
  get: () => searching.value,
  set: (open: boolean) => {
    searching.value = open;
    if (!open) query.value = '';
  },
});
useMobileBack('library', () => {
  if (!searchOpen.value) return false;
  searchOpen.value = false;
  return true;
}, 1);
const visibleItems = computed(() => {
  const term = query.value.trim().toLocaleLowerCase();
  return term
    ? pageItems.value.filter((item) => `${item.title} ${itemOwnerName(item)}`.toLocaleLowerCase().includes(term))
    : pageItems.value;
});
</script>
<template>
  <section class="mobile-folder" :aria-busy="loading">
    <header class="mobile-detail-heading folder-heading">
      <UButton
        variant="ghost"
        color="neutral"
        type="button"
        class="mobile-icon-button"
        v-if="!embedded"
        aria-label="返回内容库"
        @click="emit('back')"
      >
        <UIcon name="i-tabler-arrow-left" />
      </UButton>
      <div class="mobile-detail-copy">
        <h2>{{ categoryLabel || '内容库' }}</h2>
      </div>
      <MobileListActions v-model:search-open="searchOpen" search-label="搜索本页视频" />
    </header>
    <LibrarySourceSummary :folder="folder" :total="totalCount" :loading="loadingInitial" />
    <MobileListSearch v-model="query" v-model:open="searchOpen" label="搜索本页视频" placeholder="搜索本页标题或作者" />
    <UiInlineNotice v-if="parse.notice" :tone="parse.notice.tone" :detail="parse.notice.detail">{{
      parse.notice.message
    }}</UiInlineNotice>
    <UiInlineNotice v-if="activeError" tone="danger" :detail="activeErrorDetail ?? undefined">{{
      activeError
    }}</UiInlineNotice>
    <div v-if="loadingInitial" class="folder-loading" role="status">正在加载内容…</div>
    <template v-else>
      <CollectionTree
        v-if="source && hasCollectionSections(source)"
        :source="source"
        :selected-ids="parse.activeSelection"
        :visible-item-ids="pageItems.map((item) => item.id)"
        :disabled="loading"
        :query="query"
        @toggle="parse.togglePartIds(source.source.id, $event)"
      />
      <ParseResultListMobile
        v-else-if="source && visibleItems.length"
        roomy
        :rows="
          visibleItems.map((item) => ({
            id: item.id,
            title: item.title,
            meta: itemOwnerName(item),
            partIds: item.parts.map((part) => part.id),
          }))
        "
        :source="source"
        managing
        :selected-ids="parse.activeSelection"
        :disabled="loading"
        @toggle="
          (id) => {
            const item = pageItems.find((item) => item.id === id);
            if (item) toggleItem(item);
          }
        "
      />
      <UiEmptyState
        v-else-if="!activeError"
        :title="query ? '没有匹配的视频' : '这个集合暂时没有内容'"
        icon="i-tabler-folder-open"
        layout="stacked"
        compact
        embedded
      />
      <div v-if="totalCount > pageSize || hasMore || pacedParsing" class="folder-pagination">
        <MobilePagination
          v-if="totalCount > pageSize"
          :page="currentPage"
          :total="totalCount"
          :items-per-page="pageSize"
          :disabled="loading"
          label="集合内容分页"
          @update:page="goToPage"
        />
        <MobileSourceMenu
          v-model:batch-size="loadBatchSize"
          preload
          :has-more="hasMore"
          :busy="sourceRequestLoading"
          :parsing="pacedParsing"
          :stopping="pacedStopping"
          @load="parseMore"
          @all="parseAll"
          @stop="stopParsing"
        />
      </div>
      <footer class="mobile-download-footer folder-download">
        <div class="folder-selection-controls">
          <UiCheckbox
            :model-value="
              currentPageSelected
                ? true
                : pageItems.some((item) => item.parts.some((part) => parse.activeSelection.includes(part.id)))
                  ? 'indeterminate'
                  : false
            "
            label="全选本页"
            :disabled="loading"
            @update:model-value="toggleCurrentPageSelection"
          />
          <UiButton v-if="selectedCount" class="folder-clear-selection" size="compact" variant="ghost" :disabled="loading" aria-label="清空选择" @click="clearSelection">
            清空
          </UiButton>
        </div>
        <div class="folder-download-actions" :class="{ split: source && (hasMore || totalCount > 1) }">
          <UiButton
            class="folder-download-action"
            :variant="selectedCount ? 'primary' : 'secondary'"
            :aria-label="`下载所选 (${selectedCount})`"
            :disabled="loading || !selectedCount"
            @click="downloadSelected"
          >
            <UIcon name="i-tabler-download" class="size-4" aria-hidden="true" />
            {{ selectedCount ? `下载 ${selectedCount} 项` : '下载所选' }}
          </UiButton>
          <MobileDownloadScope
            v-if="source && (hasMore || totalCount > 1)"
            :disabled="loading"
            @download="emit('downloadAll', source.source.id)"
          />
        </div>
      </footer>
    </template>
  </section>
</template>
<style scoped>
.folder-pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  color: var(--color-muted);
  font-size: var(--font-12);
  flex-shrink: 0;
}

.mobile-folder {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: var(--space-8);
  min-height: 0;
  min-width: 0;
}

.folder-loading {
  flex: 1;
  display: grid;
  place-items: center;
  color: var(--color-muted);
}

.folder-download {
  flex-wrap: wrap;
}

.folder-selection-controls {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  white-space: nowrap;
}

.folder-selection-controls > .folder-clear-selection {
  min-width: 2.75rem;
  min-height: 2.75rem;
}

.folder-download-actions {
  display: flex;
  margin-inline-start: auto;
}

.folder-download-action {
  min-height: 2.75rem;
}

.folder-download-actions.split .folder-download-action {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}
</style>
