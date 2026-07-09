<script setup lang="ts">
import { computed, useTemplateRef } from 'vue'

import type { NormalizedSourceTree, SourceKind } from '../api/dto'
import UiButton from '../ui/Button.vue'
import UiIconButton from '../ui/IconButton.vue'
import UiStatusBadge from '../ui/StatusBadge.vue'
import UiTextarea from '../ui/Textarea.vue'
import UiTree from '../ui/Tree.vue'
import { useParseStore } from '../stores/parse'

interface PageTreeNode {
  id: string
  label: string
  meta?: string
  partIds: string[]
  children?: PageTreeNode[]
}

const sourceKindLabels: Record<SourceKind, string> = {
  video: '视频',
  bangumi: '番剧',
  cheese: '课程',
  favorite: '收藏夹',
  collection: '合集',
  series: '系列',
  uploader: 'UP 主',
  unknown: '未知',
}

const parse = useParseStore()
const inputFile = useTemplateRef<HTMLInputElement>('input-file')

const activeSource = computed(() => parse.activeSource)
const selectedIds = computed(() => parse.activeSelection)
const selectedCount = computed(() => selectedIds.value.length)
const treeNodes = computed(() => (activeSource.value ? toTreeNodes(activeSource.value) : []))
const createLoading = computed(() => Boolean(parse.loadingBySource.__create__))
const activeLoading = computed(() => Boolean(activeSource.value && parse.loadingBySource[activeSource.value.source.id]))
const activeError = computed(() => (activeSource.value ? parse.errorsBySource[activeSource.value.source.id] : null))
const canCreateTasks = computed(() => Boolean(activeSource.value && selectedCount.value > 0 && !activeLoading.value))
const canLoadMore = computed(() => Boolean(activeSource.value?.source.has_more && !activeLoading.value))
const createTaskLabel = computed(() => {
  if (activeLoading.value) {
    return '处理中'
  }
  return selectedCount.value > 0 ? `下载已选择 (${selectedCount.value})` : '先选择分集'
})
const loadedLabel = computed(() => {
  if (!activeSource.value) {
    return '--'
  }

  const total = activeSource.value.source.total_count ?? activeSource.value.source.loaded_count
  return `${activeSource.value.source.loaded_count} / ${total}`
})
const activeSourceKindLabel = computed(() => (activeSource.value ? sourceKindLabels[activeSource.value.source.kind] : '--'))

const submitInput = () => {
  void parse.createSource()
}

const importTextFile = () => {
  inputFile.value?.click()
}

const handleTextFile = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) {
    return
  }

  parse.appendInput(await file.text())
  target.value = ''
}

const createTasks = () => {
  if (activeSource.value) {
    void parse.createTasksForSelection(activeSource.value.source.id)
  }
}

const loadMore = () => {
  if (activeSource.value) {
    void parse.loadMore(activeSource.value.source.id)
  }
}

const parseAll = () => {
  if (activeSource.value) {
    const source = activeSource.value.source
    const total = source.total_count ?? '未知'
    const confirmed = window.confirm(
      `解析全部会继续加载这个来源的远端列表，当前最多解析到 100 项。\n\n当前已加载 ${source.loaded_count} / ${total}。是否继续？`,
    )
    if (confirmed) {
      void parse.parseAll(source.id, 100)
    }
  }
}

const selectAllLoaded = () => {
  if (activeSource.value) {
    parse.selectAllLoaded(activeSource.value.source.id)
  }
}

const clearSelection = () => {
  if (activeSource.value) {
    parse.clearSelection(activeSource.value.source.id)
  }
}

const refreshSource = () => {
  if (activeSource.value) {
    void parse.refreshSource(activeSource.value.source.id)
  }
}

const closeSource = (sourceId: string) => {
  void parse.closeSource(sourceId)
}

const toggleNode = (nodeId: string) => {
  if (activeSource.value) {
    parse.toggleNode(activeSource.value.source.id, nodeId)
  }
}

const toTreeNodes = (tree: NormalizedSourceTree): PageTreeNode[] =>
  tree.groups.flatMap((group) => {
    const itemNodes = group.items.map((item) => itemNode(item, tree.source.kind)).flat()

    if (tree.source.kind === 'video' && group.items.length === 1) {
      return videoItemNodes(group.items[0])
    }

    if (tree.groups.length === 1 || sameTitle(group.title, tree.source.title)) {
      return itemNodes
    }

    return [
      {
        id: group.id,
        label: group.title,
        meta: `${group.items.length} 项`,
        partIds: group.items.flatMap((item) => item.parts.map((part) => part.id)),
        children: itemNodes,
      },
    ]
  })

const itemNode = (item: NormalizedSourceTree['groups'][number]['items'][number], sourceKind: SourceKind): PageTreeNode[] => {
  if (item.parts.length === 1) {
    const part = item.parts[0]
    return [
      {
        id: item.id,
        label: item.title || part.title,
        meta: item.owner_name ?? partMeta(item, part),
        partIds: [part.id],
      },
    ]
  }

  if (sourceKind === 'video') {
    return videoItemNodes(item)
  }

  return [
    {
      id: item.id,
      label: item.title,
      meta: `${item.parts.length} P`,
      partIds: item.parts.map((part) => part.id),
      children: item.parts.map((part, index) => partNode(item, part, index)),
    },
  ]
}

const videoItemNodes = (item: NormalizedSourceTree['groups'][number]['items'][number]): PageTreeNode[] =>
  item.parts.map((part, index) => partNode(item, part, index))

const partNode = (
  item: NormalizedSourceTree['groups'][number]['items'][number],
  part: NormalizedSourceTree['groups'][number]['items'][number]['parts'][number],
  index: number,
): PageTreeNode => ({
  id: part.id,
  label: part.title || item.title || `P${index + 1}`,
  meta: partMeta(item, part),
  partIds: [part.id],
})

const partMeta = (
  item: NormalizedSourceTree['groups'][number]['items'][number],
  part: NormalizedSourceTree['groups'][number]['items'][number]['parts'][number],
): string => item.duration_seconds ? formatDuration(item.duration_seconds) : streamSummary(part.streams.length)

const sameTitle = (left: string, right: string): boolean => left.trim() !== '' && left.trim() === right.trim()

const formatDuration = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60)
  const rest = seconds % 60
  return `${minutes}:${rest.toString().padStart(2, '0')}`
}

const streamSummary = (count: number): string => (count > 0 ? `${count} 条流` : '未拉流')

const sourceLoadedLabel = (tree: NormalizedSourceTree): string =>
  `${tree.source.loaded_count} / ${tree.source.total_count ?? tree.source.loaded_count}`

const sourceSelectedCount = (sourceId: string): number => parse.selectionBySource[sourceId]?.length ?? 0
</script>

<template>
  <section class="page-grid parse-page">
    <section class="panel parse-input-panel">
      <div class="panel-heading">
        <h2>输入</h2>
        <UiStatusBadge :status="createLoading ? 'downloading' : 'ready'">
          {{ createLoading ? "解析中" : "就绪" }}
        </UiStatusBadge>
      </div>
      <form class="parse-form" @submit.prevent="submitInput">
        <UiTextarea v-model="parse.input" label="链接或 BV/AV" placeholder="BV1xx411c7mD&#10;https://www.bilibili.com/video/..." :rows="3" />
        <div class="parse-actions">
          <UiButton type="submit" :disabled="createLoading">解析</UiButton>
          <UiButton type="button" variant="secondary" :disabled="createLoading" @click="importTextFile">导入文本</UiButton>
        </div>
        <input ref="input-file" class="visually-hidden-file" type="file" accept=".txt,.list,.csv,text/plain" @change="handleTextFile" />
      </form>
    </section>

    <aside class="panel source-panel">
      <div class="panel-heading">
        <h2>来源</h2>
        <span class="muted-text">{{ parse.sourceOrder.length }} 个</span>
      </div>

      <div v-if="parse.orderedSources.length" class="source-list">
        <button
          v-for="tree in parse.orderedSources"
          :key="tree.source.id"
          class="source-row"
          :class="{ active: parse.activeSourceId === tree.source.id }"
          type="button"
          @click="parse.setActiveSource(tree.source.id)"
        >
          <span>{{ tree.source.title }}</span>
          <small>{{ sourceLoadedLabel(tree) }} · 已选 {{ sourceSelectedCount(tree.source.id) }}</small>
        </button>
      </div>
      <div v-else class="empty-state">暂无来源</div>
    </aside>

    <section class="panel result-panel">
      <div class="panel-heading">
        <h2>结果</h2>
        <span v-if="activeSource" class="muted-text">{{ activeSource.source.kind }}</span>
      </div>

      <UiTree v-if="activeSource" :nodes="treeNodes" :selected-ids="selectedIds" @toggle="toggleNode" />
      <div v-else class="empty-state">暂无结果</div>
    </section>

    <aside class="panel selection-panel">
      <div class="panel-heading">
        <div>
          <h2>准备下载</h2>
          <span class="muted-text">{{ activeSourceKindLabel }}</span>
        </div>
        <div v-if="activeSource" class="source-heading-actions">
          <UiIconButton icon="refresh" label="刷新来源" variant="ghost" :disabled="activeLoading" @click="refreshSource" />
          <UiIconButton icon="x" label="关闭来源" variant="ghost" @click="closeSource(activeSource.source.id)" />
        </div>
      </div>

      <div v-if="activeSource" class="selection-body">
        <section class="selection-summary" aria-label="当前下载选择">
          <div>
            <span>已选</span>
            <strong>{{ selectedCount }}</strong>
            <small>个分集</small>
          </div>
          <dl>
            <div>
              <dt>来源</dt>
              <dd>{{ activeSource.source.title }}</dd>
            </div>
            <div>
              <dt>解析</dt>
              <dd>{{ loadedLabel }}</dd>
            </div>
            <div>
              <dt>输入</dt>
              <dd>{{ activeSource.source.input }}</dd>
            </div>
          </dl>
        </section>

        <div class="selection-actions">
          <UiButton :disabled="!canCreateTasks" @click="createTasks">{{ createTaskLabel }}</UiButton>
        </div>

        <div class="selection-tools">
          <UiButton variant="secondary" :disabled="!activeSource || activeLoading" @click="selectAllLoaded">全选已加载</UiButton>
          <UiButton variant="secondary" :disabled="!activeSource || activeLoading || selectedCount === 0" @click="clearSelection">
            清空选择
          </UiButton>
        </div>

        <div class="parse-more-actions">
          <UiButton variant="secondary" :disabled="!canLoadMore" @click="loadMore">解析更多</UiButton>
          <UiButton variant="secondary" :disabled="!canLoadMore" @click="parseAll">解析全部</UiButton>
        </div>

        <p v-if="activeError" class="inline-alert">{{ activeError }}</p>
      </div>
      <div v-else class="empty-state">选择一个来源后创建下载任务</div>
    </aside>
  </section>
</template>

<style scoped>
.parse-page {
  grid-template-columns: minmax(190px, 220px) minmax(0, 1fr) minmax(240px, 280px);
  grid-template-rows: auto minmax(0, 1fr);
}

.parse-input-panel {
  grid-column: 1 / -1;
}

.parse-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 104px;
  align-items: start;
  gap: var(--space-12);
}

.parse-actions {
  display: grid;
  gap: var(--space-8);
  padding-top: 24px;
}

.parse-actions :deep(.ui-button) {
  width: 100%;
}

.visually-hidden-file {
  position: fixed;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}

.source-panel,
.result-panel,
.selection-panel {
  min-height: 0;
}

.source-list {
  min-height: 0;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: var(--space-8);
}

.source-row {
  min-height: 52px;
  display: grid;
  gap: var(--space-4);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
  color: var(--color-text);
  padding: var(--space-8) var(--space-12);
  text-align: left;
}

.source-row.active {
  border-color: rgb(8 127 91 / 40%);
  background: #e8f3ee;
}

.source-row span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-13);
}

.source-row small,
.empty-state {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.selection-body {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-16);
  overflow: auto;
}

.source-heading-actions {
  display: inline-flex;
  align-items: center;
  gap: var(--space-4);
}

.selection-summary {
  display: grid;
  gap: var(--space-16);
}

.selection-summary > div {
  min-height: 82px;
  display: grid;
  grid-template-columns: auto minmax(56px, auto) minmax(0, 1fr);
  align-items: baseline;
  column-gap: var(--space-8);
  padding: var(--space-16);
  border: 1px solid rgb(8 127 91 / 20%);
  border-radius: var(--radius-8);
  background: #eef8f3;
}

.selection-summary > div span,
.selection-summary > div small {
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 650;
}

.selection-summary > div strong {
  color: var(--color-accent-strong);
  font-size: 38px;
  line-height: 1;
}

.selection-summary dl {
  display: grid;
  gap: var(--space-10, 10px);
  margin: 0;
}

.selection-summary dl div {
  min-width: 0;
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr);
  gap: var(--space-8);
  align-items: start;
}

.selection-summary dt,
.selection-summary dd {
  margin: 0;
  font-size: var(--font-12);
  line-height: 1.5;
}

.selection-summary dt {
  color: var(--color-muted);
  font-weight: 700;
}

.selection-summary dd {
  min-width: 0;
  color: var(--color-text);
  overflow-wrap: anywhere;
}

.selection-actions {
  display: grid;
}

.selection-actions :deep(.ui-button) {
  width: 100%;
}

.selection-tools {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: var(--space-8);
}

.selection-tools :deep(.ui-button) {
  width: 100%;
}

.parse-more-actions {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: var(--space-8);
}

.parse-more-actions :deep(.ui-button) {
  width: 100%;
}

.empty-state {
  min-height: 120px;
  display: grid;
  place-items: center;
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

.inline-alert {
  margin: 0;
  padding: var(--space-8) var(--space-12);
  border: 1px solid rgb(201 42 42 / 20%);
  border-radius: var(--radius-6);
  background: #fffafa;
  color: var(--color-danger);
  font-size: var(--font-12);
  line-height: 1.5;
  overflow-wrap: anywhere;
}

@media (max-width: 1040px) {
  .parse-page {
    grid-template-columns: minmax(180px, 220px) minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr) auto;
  }

  .selection-panel {
    grid-column: 1 / -1;
  }
}
</style>
