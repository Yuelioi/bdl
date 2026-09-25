<script setup lang="ts">
import ParseResultListMobile from '../parse/ParseResultListMobile.vue';
import MobilePagination from './MobilePagination.vue';
import UiButton from '../../ui/Button.vue';
import UiCheckbox from '../../ui/Checkbox.vue';
import UiEmptyState from '../../ui/EmptyState.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';

import MobileSourceMenu from './MobileSourceMenu.vue';

import { useLibraryFolderDetail } from '../library/useLibraryFolderDetail';
import type { AccountLibraryFolder } from '../../api/dto';
const props = defineProps<{ folder: AccountLibraryFolder; loadingInitial?: boolean }>();
const emit = defineEmits<{ back: []; download: [] }>();
const {
  parse,
  pageSize,
  currentPage,
  loadBatchSize,
  source,

  items,
  totalCount,
  pageItems,
  selectedCount,
  sourceRequestLoading,
  pacedParsing,

  pacedStopping,
  loading,
  activeError,
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
  downloadAll,
  goToPage,
} = useLibraryFolderDetail(props, () => emit('download'));
</script>
<template>
  <section class="mobile-folder" :aria-busy="loading">
    <header class="mobile-detail-heading folder-heading">
      <button type="button" class="mobile-icon-button" aria-label="返回内容集合" @click="emit('back')">
        <UIcon name="i-tabler-arrow-left" />
      </button>
      <div class="mobile-detail-copy">
        <h2>{{ folder.title }}</h2>
        <p>{{ totalCount }} 个视频 · 已加载 {{ source?.source.loaded_count ?? items.length }}</p>
      </div>
      <MobileSourceMenu
        :href="folder.source_url"
        v-model:batch-size="loadBatchSize"
        :has-more="hasMore"
        :busy="sourceRequestLoading || !!loadingInitial"
        :parsing="pacedParsing"
        :stopping="pacedStopping"
        @load="parseMore"
        @all="parseAll"
        @download-all="downloadAll"
        @stop="stopParsing"
      />
    </header>
    <UiInlineNotice v-if="parse.notice" :tone="parse.notice.tone">{{ parse.notice.message }}</UiInlineNotice>
    <UiInlineNotice v-if="activeError" tone="danger">{{ activeError }}</UiInlineNotice>
    <div v-if="loadingInitial" class="folder-loading" role="status">正在加载内容…</div>
    <template v-else>
      <div class="mobile-selection-bar">
        <UiCheckbox
          :model-value="currentPageSelected"
          label="全选本页"
          :disabled="loading || !pageItems.length"
          @update:model-value="toggleCurrentPageSelection"
        />
        <UiButton v-if="selectedCount" variant="ghost" :disabled="loading" @click="clearSelection">取消选择</UiButton>
        <span v-else>点选视频</span>
      </div>
      <ParseResultListMobile
        v-if="source && pageItems.length"
        :rows="
          pageItems.map((item) => ({
            id: item.id,
            title: item.title,
            meta: itemOwnerName(item),
            partIds: item.parts.map((part) => part.id),
          }))
        "
        :source="source"
        :selected-ids="parse.activeSelection"
        :disabled="loading"
        @toggle="
          (id) => {
            const item = pageItems.find((item) => item.id === id);
            if (item) toggleItem(item);
          }
        "
      >
        <button
          v-if="hasMore || pacedParsing"
          class="mobile-load-more"
          type="button"
          :disabled="loading"
          @click="parseMore"
        >
          {{ pacedParsing ? '正在加载更多内容…' : sourceRequestLoading ? '加载中…' : '继续加载内容' }}
        </button>
      </ParseResultListMobile>
      <UiEmptyState
        v-else-if="!activeError"
        title="这个集合暂时没有内容"
        icon="i-tabler-folder-open"
        layout="stacked"
        compact
        embedded
      />
      <MobilePagination
        v-if="totalCount > pageSize"
        :page="currentPage"
        :total="totalCount"
        :items-per-page="pageSize"
        :disabled="loading"
        label="集合内容分页"
        @update:page="goToPage"
      />
      <footer class="mobile-download-footer folder-download">
        <span
          >已选 <strong>{{ selectedCount }}</strong> 项</span
        ><UiButton size="compact" :disabled="loading || !selectedCount" @click="downloadSelected"
          >下载所选 ({{ selectedCount }})</UiButton
        >
      </footer>
    </template>
  </section>
</template>
<style scoped>
.mobile-folder {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
}

.folder-loading {
  flex: 1;
  display: grid;
  place-items: center;
  color: var(--color-muted);
}
</style>
