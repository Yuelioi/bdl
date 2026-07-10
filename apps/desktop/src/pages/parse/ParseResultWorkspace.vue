<script setup lang="ts">
import { computed, ref } from 'vue'

import { useParseStore } from '../../stores/parse'
import UiButton from '../../ui/Button.vue'
import UiEmptyState from '../../ui/EmptyState.vue'
import UiIconButton from '../../ui/IconButton.vue'
import UiInlineNotice from '../../ui/InlineNotice.vue'
import UiSelect from '../../ui/Select.vue'
import UiTextField from '../../ui/TextField.vue'
import UiTree from '../../ui/Tree.vue'
import {
  filterTreeNodes,
  flattenVisibleParts,
  numberVisibleParts,
  parseRangeExpression,
  type ResultSortMode,
  sortTreeNodes,
  sourceKindLabels,
  sourcePartCount,
  toTreeNodes,
} from './parseResultTree'

const emit = defineEmits<{ download: [] }>()
const parse = useParseStore()
const resultQuery = ref('')
const resultSort = ref<ResultSortMode>('source')
const rangeExpression = ref('')
const rangeError = ref('')

const resultSortOptions = [
  { label: '原始顺序', value: 'source' },
  { label: '标题 A-Z', value: 'title_asc' },
  { label: '时长优先', value: 'duration_desc' },
]

const activeSource = computed(() => parse.activeSource)
const selectedIds = computed(() => parse.activeSelection)
const selectedCount = computed(() => selectedIds.value.length)
const totalPartCount = computed(() => activeSource.value ? sourcePartCount(activeSource.value) : 0)
const treeNodes = computed(() => {
  if (!activeSource.value) return []
  const filtered = filterTreeNodes(toTreeNodes(activeSource.value), resultQuery.value)
  return numberVisibleParts(sortTreeNodes(filtered, resultSort.value))
})
const visiblePartEntries = computed(() => flattenVisibleParts(treeNodes.value))
const visiblePartCount = computed(() => visiblePartEntries.value.length)
const activeLoading = computed(() => Boolean(activeSource.value && parse.loadingBySource[activeSource.value.source.id]))
const activeError = computed(() => activeSource.value ? parse.errorsBySource[activeSource.value.source.id] : null)
const hasResultQuery = computed(() => resultQuery.value.trim().length > 0)
const canCreateTasks = computed(() => Boolean(activeSource.value && selectedCount.value > 0 && !activeLoading.value))
const canSelectResults = computed(() => Boolean(activeSource.value && visiblePartCount.value > 0 && !activeLoading.value))
const canSelectRange = computed(() => Boolean(activeSource.value && rangeExpression.value.trim() && visiblePartCount.value > 0 && !activeLoading.value))
const createTaskLabel = computed(() => activeLoading.value
  ? '处理中'
  : selectedCount.value > 0 ? `下载已选择 (${selectedCount.value})` : '先选择分集')
const selectAllLabel = computed(() => hasResultQuery.value ? `全选搜索结果 (${visiblePartCount.value})` : '全选全部')

const selectAllResults = () => {
  if (!activeSource.value) return
  parse.selectPartIds(activeSource.value.source.id, visiblePartEntries.value.map((entry) => entry.id))
}

const clearSelection = () => {
  if (activeSource.value) parse.clearSelection(activeSource.value.source.id)
}

const refreshSource = () => {
  if (activeSource.value) void parse.refreshSource(activeSource.value.source.id)
}

const closeSource = () => {
  if (activeSource.value) void parse.closeSource(activeSource.value.source.id)
}

const toggleNode = (nodeId: string) => {
  if (activeSource.value) parse.toggleNode(activeSource.value.source.id, nodeId)
}

const selectRange = () => {
  if (!activeSource.value) return
  rangeError.value = ''
  try {
    const indexes = parseRangeExpression(rangeExpression.value, visiblePartEntries.value.length)
    const partIds = indexes.map((index) => visiblePartEntries.value[index].id)
    parse.selectPartIds(activeSource.value.source.id, partIds)
    parse.setNotice(`已选中 ${partIds.length} 个结果`, 'success')
  } catch (error) {
    rangeError.value = error instanceof Error ? error.message : String(error)
  }
}
</script>

<template>
  <section v-if="activeSource" class="panel min-h-0 overflow-hidden">
    <div
      class="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-3 border-b border-(--color-border) pb-3 max-[840px]:grid-cols-1"
    >
      <div class="grid min-w-0 gap-1">
        <div class="flex min-w-0 items-center gap-2">
          <strong class="truncate text-base text-(--color-text)" :title="activeSource.source.title">
            {{ activeSource.source.title }}
          </strong>
          <span class="shrink-0 text-[11px] font-bold text-(--color-muted)">
            {{ sourceKindLabels[activeSource.source.kind] }}
          </span>
        </div>
        <div class="flex flex-wrap items-center gap-3 text-xs text-(--color-muted)" aria-label="内容选择统计">
          <span>共 <b class="font-bold text-(--color-text)">{{ totalPartCount }}</b> 项</span>
          <span>已选 <b class="font-bold text-(--color-text)">{{ selectedCount }}</b> 项</span>
          <span v-if="hasResultQuery">搜索找到 <b class="font-bold text-(--color-text)">{{ visiblePartCount }}</b> 项</span>
        </div>
      </div>

      <div class="flex flex-wrap justify-end gap-1 max-[840px]:justify-start">
        <UiButton size="compact" variant="ghost" :disabled="!canSelectResults" @click="selectAllResults">{{ selectAllLabel }}</UiButton>
        <UiButton size="compact" variant="ghost" :disabled="selectedCount === 0" @click="clearSelection">清空选择</UiButton>
        <UiIconButton icon="refresh" label="刷新当前来源" variant="ghost" size="compact" :disabled="activeLoading" @click="refreshSource" />
        <UiIconButton v-if="parse.sourceOrder.length > 1" icon="x" label="关闭当前结果" variant="ghost" size="compact" @click="closeSource" />
        <UiButton :disabled="!canCreateTasks" @click="emit('download')">{{ createTaskLabel }}</UiButton>
      </div>

      <div class="col-span-full grid min-w-0 grid-cols-[minmax(180px,1fr)_minmax(150px,180px)_minmax(220px,280px)] items-end gap-2.5 max-[840px]:grid-cols-1">
        <UiTextField v-model="resultQuery" label="搜索内容" placeholder="标题 / UP 主 / BV" :disabled="activeLoading" />
        <UiSelect v-model="resultSort" label="排序" :options="resultSortOptions" :disabled="activeLoading" />
        <div class="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-end gap-2">
          <UiTextField v-model="rangeExpression" label="序号范围" placeholder="1-5,7,9-12" :disabled="activeLoading || visiblePartCount === 0" />
          <UiButton class="h-9" variant="secondary" :disabled="!canSelectRange" @click="selectRange">选中</UiButton>
        </div>
        <p v-if="rangeError" class="col-span-full -mt-2 m-0 text-xs font-bold text-(--color-danger)">{{ rangeError }}</p>
      </div>
    </div>

    <UiTree v-if="treeNodes.length" :nodes="treeNodes" :selected-ids="selectedIds" @toggle="toggleNode" />
    <UiEmptyState
      v-else
      title="没有匹配结果"
      description="换一个关键词，或清除搜索与范围条件。"
      icon="i-tabler-filter-off"
      layout="stacked"
      compact
      embedded
    />
    <UiInlineNotice v-if="activeError" tone="danger">{{ activeError }}</UiInlineNotice>
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
