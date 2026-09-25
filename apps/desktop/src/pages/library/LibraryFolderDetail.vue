<script setup lang="ts">
import { coverUrl } from "../../utils/coverUrl";
import UiButton from '../../ui/Button.vue'
import UiCheckbox from '../../ui/Checkbox.vue'
import UiEmptyState from '../../ui/EmptyState.vue'
import UiInlineNotice from '../../ui/InlineNotice.vue'
import UiPagination from '../../ui/Pagination.vue'
import SelectionActionBar from '../../ui/SelectionActionBar.vue'
import ExternalLinkButton from '../../ui/ExternalLinkButton.vue'
import SourceParseControls from '../parse/SourceParseControls.vue'
import SourceLoadStatus from '../parse/SourceLoadStatus.vue'
import { useLibraryFolderDetail } from "./useLibraryFolderDetail";
import type { AccountLibraryFolder } from "../../api/dto";
const props = defineProps<{folder: AccountLibraryFolder; loadingInitial?: boolean}>();
const emit = defineEmits<{back: []; download: []}>();
const { parse, pageSize, currentPage, failedCoverIds, loadBatchSize, source, sourceId, items, totalCount, totalPages, pageItems, selectedCount, sourceRequestLoading, pacedParsing, pacedWaiting, pacedStopping, loading, activeError, hasMore, itemOwnerName, itemSelected, currentPageSelected, toggleItem, toggleCurrentPageSelection, clearSelection, downloadItem, downloadSelected, parseMore, parseAll, stopParsing, downloadAll, goToPage, formatDuration, bilibiliVideoUrl } = useLibraryFolderDetail(props, () => emit("download"));
</script>
<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4" :aria-busy="loading">
    <header
      class="library-folder-header flex min-w-0 items-center justify-between gap-4 border-b border-(--color-border) pb-4"
    >
      <div class="flex min-w-0 items-center gap-3">
        <button
          type="button"
          class="grid size-8 shrink-0 place-items-center rounded-md text-(--color-muted) hover:bg-(--color-panel) hover:text-(--color-text)"
          aria-label="返回内容集合"
          @click="emit('back')"
        >
          <UIcon name="i-tabler-arrow-left" class="size-4" aria-hidden="true" />
        </button>
        <h2 class="truncate m-0 text-base text-(--color-text)" :title="folder.title">
          <ExternalLinkButton
            :href="folder.source_url"
            :label="`在 Bilibili 打开 ${folder.title}`"
            :show-icon="false"
          >
            <span class="truncate text-(--color-text)">{{ folder.title }}</span>
          </ExternalLinkButton>
        </h2>
      </div>
      <div v-if="!loadingInitial" class="source-function-toolbar">
        <SourceLoadStatus :loaded="source?.source.loaded_count ?? items.length" :total="totalCount" />
        <SourceParseControls
          :source-id="sourceId ?? undefined"
          v-if="hasMore || pacedParsing"
          v-model:batch-size="loadBatchSize"
          :has-more="hasMore"
          :loading="sourceRequestLoading"
          :parsing-all="pacedParsing"
          :waiting="pacedWaiting"
          :stopping="pacedStopping"
          @parse-batch="parseMore"
          @parse-all="parseAll"
          @parse-and-download="sourceId && parse.startBackgroundDownload(sourceId)"
          @stop="stopParsing"
        />
        <UiButton
          v-if="!hasMore"
          size="compact"
          variant="secondary"
          :disabled="loading || totalCount === 0"
          @click="downloadAll"
        >
          下载全部
        </UiButton>
        <UiButton size="compact" :disabled="loading || selectedCount === 0" @click="downloadSelected">
          下载所选 ({{ selectedCount }})
        </UiButton>
      </div>
    </header>

    <UiInlineNotice v-if="parse.notice" :tone="parse.notice.tone">{{ parse.notice.message }}</UiInlineNotice>
    <UiInlineNotice v-if="activeError" tone="danger">{{ activeError }}</UiInlineNotice>

    <div
      v-if="loadingInitial"
      class="grid min-h-0 flex-1 grid-cols-[repeat(auto-fill,minmax(170px,1fr))] content-start gap-3 overflow-hidden pr-0.5"
      role="status"
      aria-label="正在加载合集内容"
    >
      <article v-for="index in 8" :key="index" class="library-detail-skeleton-card" aria-hidden="true">
        <span class="library-detail-skeleton-cover"></span>
        <span class="library-detail-skeleton-line library-detail-skeleton-title"></span>
        <span class="library-detail-skeleton-line library-detail-skeleton-meta"></span>
      </article>
    </div>

    <div
      v-else-if="pageItems.length"
      class="grid min-h-0 flex-1 grid-cols-[repeat(auto-fill,minmax(170px,1fr))] content-start gap-3 overflow-y-auto pr-0.5"
    >
      <article
        v-for="item in pageItems"
        :key="item.id"
        class="library-video-card group grid min-w-0 content-start gap-2 rounded-lg border border-(--color-border) bg-(--color-surface) p-2.5"
        :class="{ 'is-selected': itemSelected(item) }"
        :data-selected="itemSelected(item)"
        @click="toggleItem(item)"
      >
        <div class="relative aspect-video overflow-hidden rounded-md bg-(--color-inset)">
          <img
referrerpolicy="no-referrer"
            v-if="item.cover_url && !failedCoverIds.includes(item.id)"
            :src="coverUrl(item.cover_url) ?? undefined"
            alt=""
            class="size-full object-cover"
            loading="lazy"
            @error="failedCoverIds = [...failedCoverIds, item.id]"
          />
          <span v-else class="grid size-full place-items-center text-(--color-dimmed)" aria-hidden="true">
            <UIcon name="i-tabler-photo-off" class="size-6" />
          </span>
          <UiCheckbox
            class="library-card-selector"
            :model-value="itemSelected(item)"
            :label="`选择 ${item.title}`"
            :disabled="loading"
            compact
            @click.stop
            @update:model-value="toggleItem(item)"
          />
          <span
            class="absolute right-1.5 bottom-1.5 rounded bg-black/70 px-1.5 py-0.5 text-[11px] font-bold text-white"
          >
            {{ formatDuration(item.duration_seconds) }}
          </span>
        </div>
        <ExternalLinkButton
          v-if="bilibiliVideoUrl(item)"
          :href="bilibiliVideoUrl(item) ?? ''"
          :label="`在 Bilibili 打开 ${item.title}`"
          :show-icon="false"
        >
          <span class="library-video-title line-clamp-2 min-h-10 leading-5 text-(--color-text)" :title="item.title">
            {{ item.title }}
          </span>
        </ExternalLinkButton>
        <h3
          v-else
          class="library-video-title line-clamp-2 m-0 min-h-10 text-[13px] leading-5 text-(--color-text)"
          :title="item.title"
        >
          {{ item.title }}
        </h3>
        <div class="flex min-w-0 items-center justify-between gap-2">
          <span class="library-video-card-owner truncate text-[11px] text-(--color-muted)">
            {{ itemOwnerName(item) }}
          </span>
          <UiButton size="compact" variant="ghost" :disabled="loading" @click.stop="downloadItem(item)">下载</UiButton>
        </div>
      </article>
    </div>

    <UiEmptyState
      v-else-if="!parse.notice && !activeError"
      title="这个集合暂时没有内容"
      icon="i-tabler-folder-open"
      layout="stacked"
      compact
      embedded
    />

    <SelectionActionBar v-if="!loadingInitial" :selected-count="selectedCount" :total-count="totalCount">
      <template #leading>
        <div class="library-pagination-status">
          <span class="library-selection-count">已选 {{ selectedCount }} / {{ totalCount }}</span>
          <UiPagination
            :page="currentPage"
            :total="totalCount"
            :items-per-page="pageSize"
            :disabled="loading"
            label="集合内容分页"
            @update:page="goToPage"
          />
          <span>第 {{ currentPage }} / {{ totalPages }} 页</span>
        </div>
      </template>
      <template #selection>
        <UiButton
          size="compact"
          variant="ghost"
          :disabled="loading || pageItems.length === 0"
          @click="toggleCurrentPageSelection"
        >
          {{ currentPageSelected ? '取消本页选择' : '全选本页' }}
        </UiButton>
        <UiButton
          v-if="selectedCount > 0 && !currentPageSelected"
          size="compact"
          variant="ghost"
          :disabled="loading"
          @click="clearSelection"
        >
          取消选择
        </UiButton>
      </template>
    </SelectionActionBar>
  </section>
</template>

<style scoped>
.library-video-card {
  cursor: pointer;
  transition:
    border-color var(--duration-fast) var(--ease-out),
    background var(--duration-fast) var(--ease-out);
}

.library-video-card:hover {
  border-color: var(--color-border-strong);
  background: var(--color-panel);
}

.library-video-card.is-selected {
  border-color: var(--color-accent);
  background: var(--color-surface);
}

.library-video-card.is-selected:hover {
  background: var(--color-accent-faint);
}

.library-card-selector {
  position: absolute;
  z-index: var(--z-content);
  top: var(--space-8);
  left: var(--space-8);
  width: 28px;
  height: 28px;
  min-height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.library-pagination-status {
  min-height: 28px;
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-xs);
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1;
  white-space: nowrap;
}

.library-selection-count {
  color: var(--color-text);
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}

.source-function-toolbar {
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-xs);
}

.library-video-title {
  transition: color var(--duration-fast) var(--ease-out);
}

.library-video-card:hover .library-video-title,
.library-video-card:focus-within .library-video-title {
  color: var(--color-accent-strong);
}

.library-detail-skeleton-card {
  display: grid;
  min-width: 0;
  gap: 0.5rem;
  padding: 0.625rem;
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  background: var(--color-surface);
}

.library-detail-skeleton-cover,
.library-detail-skeleton-line {
  display: block;
  border-radius: 0.375rem;
  background: var(--color-panel);
  animation: library-detail-pulse 1.4s ease-in-out infinite alternate;
}

.library-detail-skeleton-cover {
  aspect-ratio: 16 / 9;
}

.library-detail-skeleton-line {
  height: 0.75rem;
}

.library-detail-skeleton-title {
  width: 82%;
}

.library-detail-skeleton-meta {
  width: 46%;
  height: 0.625rem;
}

@keyframes library-detail-pulse {
  to {
    opacity: 0.48;
  }
}

@media (prefers-reduced-motion: reduce) {
  .library-video-title {
    transition: none;
  }

  .library-video-card {
    transition: none;
  }

  .library-detail-skeleton-cover,
  .library-detail-skeleton-line {
    animation: none;
  }
}

@media (width <= 700px) {
  .library-folder-header {
    align-items: stretch;
    flex-direction: column;
    gap: var(--space-8);
    padding-bottom: var(--space-10);
  }

  .source-function-toolbar {
    width: 100%;
    justify-content: flex-start;
    gap: var(--space-6);
  }

  .library-pagination-status {
    width: 100%;
    height: auto;
    flex-wrap: wrap;
    row-gap: var(--space-6);
  }
}
</style>
