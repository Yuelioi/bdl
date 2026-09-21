<script setup lang="ts">
import { computed, ref } from 'vue'

import { useParseStore } from '../../stores/parse'
import UiButton from '../../ui/Button.vue'
import UiEmptyState from '../../ui/EmptyState.vue'
import UiInlineNotice from '../../ui/InlineNotice.vue'
import SelectionActionBar from '../../ui/SelectionActionBar.vue'
import ParseResultTable from './ParseResultTable.vue'
import SourceLoadStatus from './SourceLoadStatus.vue'
import SourceParseControls from './SourceParseControls.vue'
import {
  flattenResultRows,
  sourceKindLabels,
  sourcePartCount,
  toTreeNodes,
} from './parseResultTree'

const emit = defineEmits<{ download: [] }>()
const { embedded = false } = defineProps<{ embedded?: boolean }>()
const parse = useParseStore()
const loadBatchSize = ref('50')
const activeSource = computed(() => parse.activeSource)
const selectedIds = computed(() => parse.activeSelection)
const selectedCount = computed(() => selectedIds.value.length)
const totalPartCount = computed(() => (activeSource.value ? sourcePartCount(activeSource.value) : 0))
const tableRows = computed(() => (activeSource.value ? flattenResultRows(toTreeNodes(activeSource.value)) : []))
const sourceRequestLoading = computed(() =>
  Boolean(activeSource.value && parse.loadingBySource[activeSource.value.source.id]),
)
const pacedParsing = computed(() =>
  Boolean(activeSource.value && parse.pacedParsingBySource[activeSource.value.source.id]),
)
const pacedWaiting = computed(() =>
  Boolean(activeSource.value && parse.pacedParsingWaitingBySource[activeSource.value.source.id]),
)
const pacedStopping = computed(() =>
  Boolean(activeSource.value && parse.pacedParsingStopRequestedBySource[activeSource.value.source.id]),
)
const activeLoading = computed(() => sourceRequestLoading.value || pacedParsing.value)
const activeError = computed(() => (activeSource.value ? parse.errorsBySource[activeSource.value.source.id] : null))
const hasMore = computed(() => Boolean(activeSource.value?.source.has_more))
const allRowsSelected = computed(
  () =>
    tableRows.value.length > 0 &&
    tableRows.value.flatMap((row) => row.partIds).every((partId) => selectedIds.value.includes(partId)),
)
const canCreateTasks = computed(() => Boolean(activeSource.value && selectedCount.value > 0 && !activeLoading.value))
const createTaskLabel = computed(() => `下载所选 (${selectedCount.value})`)

const toggleAllResults = () => {
  if (!activeSource.value) return
  if (allRowsSelected.value) {
    parse.clearSelection(activeSource.value.source.id)
    return
  }
  parse.selectPartIds(
    activeSource.value.source.id,
    tableRows.value.flatMap((row) => row.partIds),
  )
}

const clearSelection = () => {
  if (activeSource.value) parse.clearSelection(activeSource.value.source.id)
}

const loadMore = () => {
  if (activeSource.value?.source.has_more) {
    void parse.loadChunk(activeSource.value.source.id, Number(loadBatchSize.value))
  }
}

const parseAll = () => {
  if (activeSource.value?.source.has_more) {
    void parse.parseAllPaced(activeSource.value.source.id, Number(loadBatchSize.value))
  }
}

const stopParsing = () => {
  if (activeSource.value) parse.stopPacedParsing(activeSource.value.source.id)
}

const downloadAllLoaded = () => {
  if (!activeSource.value) return
  parse.selectAllLoaded(activeSource.value.source.id)
  emit('download')
}

const parseAndDownload = () => {
  if (activeSource.value) void parse.startBackgroundDownload(activeSource.value.source.id)
}

const returnToSource = () => {
  if (!activeLoading.value) void parse.clearWorkspace()
}

const toggleNode = (nodeId: string) => {
  if (activeSource.value) parse.toggleNode(activeSource.value.source.id, nodeId)
}

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
          <strong class="truncate text-base text-(--color-text)" :title="activeSource.source.title">
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
