<script setup lang="ts">
import { computed } from 'vue'

import type { NormalizedSourceTree } from '../api/dto'
import UiButton from '../ui/Button.vue'
import UiIconButton from '../ui/IconButton.vue'
import UiStatusBadge from '../ui/StatusBadge.vue'
import UiTextField from '../ui/TextField.vue'
import UiTree from '../ui/Tree.vue'
import { useParseStore } from '../stores/parse'

interface PageTreeNode {
  id: string
  label: string
  meta?: string
  children?: PageTreeNode[]
}

const parse = useParseStore()

const activeSource = computed(() => parse.activeSource)
const selectedIds = computed(() => parse.activeSelection)
const selectedCount = computed(() => selectedIds.value.length)
const treeNodes = computed(() => (activeSource.value ? toTreeNodes(activeSource.value) : []))
const createLoading = computed(() => Boolean(parse.loadingBySource.__create__))
const activeLoading = computed(() => Boolean(activeSource.value && parse.loadingBySource[activeSource.value.source.id]))
const activeError = computed(() => (activeSource.value ? parse.errorsBySource[activeSource.value.source.id] : null))

const submitInput = () => {
  void parse.createSource()
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
    void parse.parseAll(activeSource.value.source.id)
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
  tree.groups.map((group) => ({
    id: group.id,
    label: group.title,
    meta: `${group.items.length} 项`,
    children: group.items.map((item) => ({
      id: item.id,
      label: item.title,
      meta: item.owner_name ?? undefined,
      children: item.parts.map((part, index) => ({
        id: part.id,
        label: part.title || `P${index + 1}`,
        meta: item.duration_seconds ? formatDuration(item.duration_seconds) : streamSummary(part.streams.length),
      })),
    })),
  }))

const formatDuration = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60)
  const rest = seconds % 60
  return `${minutes}:${rest.toString().padStart(2, '0')}`
}

const streamSummary = (count: number): string => (count > 0 ? `${count} 条流` : '未拉流')
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
        <UiTextField v-model="parse.input" label="链接或 BV/AV" placeholder="BV1xx411c7mD" />
        <UiButton type="submit" :disabled="createLoading">解析</UiButton>
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
          <small>{{ tree.source.loaded_count }} / {{ tree.source.total_count ?? tree.source.loaded_count }}</small>
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
        <h2>选择</h2>
        <span class="muted-text">{{ selectedCount }} 项</span>
      </div>

      <div v-if="activeSource" class="source-detail">
        <strong>{{ activeSource.source.title }}</strong>
        <span>{{ activeSource.source.input }}</span>
        <span>已加载 {{ activeSource.source.loaded_count }} 项</span>
      </div>

      <p v-if="activeError" class="error-text">{{ activeError }}</p>

      <div class="selection-actions">
        <UiButton :disabled="!activeSource || activeLoading" @click="createTasks">下载已选择</UiButton>
        <UiButton variant="secondary" :disabled="!activeSource || !activeSource.source.has_more || activeLoading" @click="loadMore">
          解析更多
        </UiButton>
        <UiButton variant="secondary" :disabled="!activeSource || !activeSource.source.has_more || activeLoading" @click="parseAll">
          解析全部
        </UiButton>
      </div>

      <UiIconButton v-if="activeSource" icon="x" label="关闭来源" variant="ghost" @click="closeSource(activeSource.source.id)" />
    </aside>
  </section>
</template>

<style scoped>
.parse-page {
  grid-template-columns: 220px minmax(0, 1fr) 280px;
  grid-template-rows: auto minmax(0, 1fr);
}

.parse-input-panel {
  grid-column: 1 / -1;
}

.parse-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 96px;
  align-items: end;
  gap: var(--space-12);
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

.source-row span,
.source-detail strong {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-13);
}

.source-row small,
.source-detail span,
.empty-state,
.error-text {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.source-detail {
  display: grid;
  gap: var(--space-6, 6px);
}

.selection-actions {
  display: grid;
  gap: var(--space-8);
}

.selection-actions :deep(.ui-button) {
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

.error-text {
  margin: 0;
  color: var(--color-danger);
  line-height: 1.5;
}
</style>
