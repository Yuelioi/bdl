<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import type { AccountLibraryFolder, NormalizedItem } from '../../api/dto'
import { useParseStore } from '../../stores/parse'
import UiButton from '../../ui/Button.vue'
import UiCheckbox from '../../ui/Checkbox.vue'
import UiEmptyState from '../../ui/EmptyState.vue'
import UiInlineNotice from '../../ui/InlineNotice.vue'
import UiPagination from '../../ui/Pagination.vue'

const { folder } = defineProps<{ folder: AccountLibraryFolder }>()
const emit = defineEmits<{ back: []; download: [] }>()

const parse = useParseStore()
const pageSize = 20
const currentPage = ref(1)
const failedCoverIds = ref<string[]>([])

const source = computed(() => parse.activeSource)
const sourceId = computed(() => source.value?.source.id ?? null)
const items = computed(() => source.value?.groups.flatMap((group) => group.items) ?? [])
const totalCount = computed(() => source.value?.source.total_count ?? folder.media_count)
const totalPages = computed(() => Math.max(1, Math.ceil(totalCount.value / pageSize)))
const pageItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return items.value.slice(start, start + pageSize)
})
const selectedSet = computed(() => new Set(parse.activeSelection))
const selectedCount = computed(() => parse.activeSelection.length)
const loading = computed(() => Boolean(sourceId.value && parse.loadingBySource[sourceId.value]))
const activeError = computed(() => (sourceId.value ? parse.errorsBySource[sourceId.value] : null))
const hasMore = computed(() => Boolean(source.value?.source.has_more))
const loadedLabel = computed(() => `已加载 ${items.value.length} / ${totalCount.value}`)

watch(sourceId, () => {
  currentPage.value = 1
  failedCoverIds.value = []
})

const itemPartIds = (item: NormalizedItem): string[] => item.parts.map((part) => part.id)
const itemSelected = (item: NormalizedItem): boolean => {
  const partIds = itemPartIds(item)
  return partIds.length > 0 && partIds.every((partId) => selectedSet.value.has(partId))
}

const toggleItem = (item: NormalizedItem) => {
  if (sourceId.value) parse.toggleNode(sourceId.value, item.id)
}

const selectCurrentPage = () => {
  if (!sourceId.value) return
  parse.selectPartIds(sourceId.value, pageItems.value.flatMap(itemPartIds))
}

const clearSelection = () => {
  if (sourceId.value) parse.clearSelection(sourceId.value)
}

const downloadItem = (item: NormalizedItem) => {
  if (!sourceId.value) return
  parse.selectPartIds(sourceId.value, itemPartIds(item), 'replace')
  emit('download')
}

const downloadSelected = () => {
  if (selectedCount.value > 0) emit('download')
}

const parseMore = async () => {
  if (sourceId.value && hasMore.value) await parse.loadMore(sourceId.value)
}

const downloadAll = async () => {
  if (!sourceId.value) return
  if (hasMore.value) await parse.parseAll(sourceId.value)
  parse.selectAllLoaded(sourceId.value)
  emit('download')
}

const goToPage = async (targetPage: number) => {
  if (!sourceId.value) return
  const nextPage = Math.min(Math.max(1, targetPage), totalPages.value)
  const nextStart = (nextPage - 1) * pageSize
  while (nextStart >= items.value.length && source.value?.source.has_more) {
    const loadedBefore = items.value.length
    await parse.loadMore(sourceId.value)
    if (items.value.length === loadedBefore) break
  }
  if (nextStart < items.value.length) currentPage.value = nextPage
}

const formatDuration = (seconds: number | null): string => {
  if (!seconds || seconds <= 0) return '--:--'
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const remaining = Math.floor(seconds % 60)
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, '0')}:${String(remaining).padStart(2, '0')}`
    : `${minutes}:${String(remaining).padStart(2, '0')}`
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4" :aria-busy="loading">
    <header class="flex min-w-0 items-center justify-between gap-4 border-b border-(--color-border) pb-4">
      <div class="flex min-w-0 items-center gap-3">
        <button
          type="button"
          class="grid size-8 shrink-0 place-items-center rounded-md text-(--color-muted) hover:bg-(--color-panel) hover:text-(--color-text)"
          aria-label="返回内容集合"
          @click="emit('back')"
        >
          <UIcon name="i-tabler-arrow-left" class="size-4" aria-hidden="true" />
        </button>
        <img
          v-if="folder.cover_url"
          :src="folder.cover_url"
          alt=""
          class="h-12 w-20 shrink-0 rounded-md object-cover"
          referrerpolicy="no-referrer"
        />
        <div class="grid min-w-0 gap-1">
          <h2 class="truncate m-0 text-base text-(--color-text)" :title="folder.title">{{ folder.title }}</h2>
          <p class="m-0 text-xs text-(--color-muted)">{{ folder.media_count }} 个视频 · {{ loadedLabel }}</p>
        </div>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <UiButton size="compact" variant="secondary" :disabled="loading || !hasMore" @click="parseMore">
          {{ hasMore ? '解析更多' : '已全部加载' }}
        </UiButton>
        <UiButton size="compact" :disabled="loading || totalCount === 0" @click="downloadAll">下载全部</UiButton>
      </div>
    </header>

    <div class="flex min-w-0 items-center justify-between gap-3">
      <p class="m-0 text-xs text-(--color-muted)">第 {{ currentPage }} / {{ totalPages }} 页</p>
      <UiButton size="compact" variant="ghost" :disabled="loading || pageItems.length === 0" @click="selectCurrentPage">
        全选本页
      </UiButton>
    </div>

    <UiInlineNotice v-if="parse.notice" :tone="parse.notice.tone">{{ parse.notice.message }}</UiInlineNotice>
    <UiInlineNotice v-if="activeError" tone="danger">{{ activeError }}</UiInlineNotice>

    <div
      v-if="pageItems.length"
      class="grid min-h-0 flex-1 grid-cols-[repeat(auto-fill,minmax(170px,1fr))] content-start gap-3 overflow-y-auto pr-0.5"
    >
      <article
        v-for="item in pageItems"
        :key="item.id"
        class="group grid min-w-0 content-start gap-2 rounded-lg border border-(--color-border) bg-(--color-surface) p-2.5"
        :class="itemSelected(item) ? 'border-(--color-accent) bg-(--color-accent-faint)' : 'hover:bg-(--color-panel)'"
      >
        <div class="relative aspect-video overflow-hidden rounded-md bg-(--color-inset)">
          <img
            v-if="item.cover_url && !failedCoverIds.includes(item.id)"
            :src="item.cover_url"
            alt=""
            class="size-full object-cover"
            loading="lazy"
            referrerpolicy="no-referrer"
            @error="failedCoverIds = [...failedCoverIds, item.id]"
          />
          <span v-else class="grid size-full place-items-center text-(--color-dimmed)" aria-hidden="true">
            <UIcon name="i-tabler-photo-off" class="size-6" />
          </span>
          <span class="absolute top-2 left-2 rounded bg-(--color-surface-raised) p-1" @click.stop @keydown.stop>
            <UiCheckbox
              :model-value="itemSelected(item)"
              :label="itemSelected(item) ? '取消选择' : '选择视频'"
              compact
              :disabled="loading"
              @update:model-value="toggleItem(item)"
            />
          </span>
          <span
            class="absolute right-1.5 bottom-1.5 rounded bg-black/70 px-1.5 py-0.5 text-[11px] font-bold text-white"
          >
            {{ formatDuration(item.duration_seconds) }}
          </span>
        </div>
        <h3 class="line-clamp-2 m-0 min-h-10 text-[13px] leading-5 text-(--color-text)" :title="item.title">
          {{ item.title }}
        </h3>
        <div class="flex min-w-0 items-center justify-between gap-2">
          <span class="truncate text-[11px] text-(--color-muted)" :title="item.owner_name ?? undefined">{{
            item.owner_name ?? '未知 UP 主'
          }}</span>
          <UiButton size="compact" variant="ghost" :disabled="loading" @click="downloadItem(item)">下载</UiButton>
        </div>
      </article>
    </div>

    <UiEmptyState v-else title="这个集合暂时没有内容" icon="i-tabler-folder-open" layout="stacked" compact embedded />

    <footer class="flex min-w-0 items-center justify-between gap-4 border-t border-(--color-border) pt-3">
      <UiPagination
        :page="currentPage"
        :total="totalCount"
        :items-per-page="pageSize"
        :disabled="loading"
        label="集合内容分页"
        @update:page="goToPage"
      />
      <div v-if="selectedCount > 0" class="flex items-center gap-2">
        <strong class="text-[13px] text-(--color-text)">已选 {{ selectedCount }} 个</strong>
        <UiButton size="compact" variant="ghost" :disabled="loading" @click="clearSelection">取消选择</UiButton>
        <UiButton size="compact" :disabled="loading" @click="downloadSelected">下载所选</UiButton>
      </div>
    </footer>
  </section>
</template>
