<script setup lang="ts">
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'
import { calculateVirtualWindow } from '../utils/virtualWindow'

const TREE_ROW_STRIDE = 38

export interface TreeNode {
  id: string
  label: string
  meta?: string
  partIds?: string[]
  children?: TreeNode[]
}

const props = defineProps<{
  nodes: TreeNode[]
  selectedIds?: string[]
}>()

const emit = defineEmits<{
  toggle: [id: string]
}>()

interface FlatTreeNode {
  id: string
  label: string
  meta?: string
  depth: number
  hasChildren: boolean
  checkState: 'checked' | 'mixed' | 'empty'
}

const selectedSet = computed(() => new Set(props.selectedIds ?? []))
const activeIndex = ref(0)
const listElement = useTemplateRef<HTMLElement>('tree-list')
const scrollOffset = ref(0)
const viewportSize = ref(600)

const flatNodes = computed(() => {
  const rows: FlatTreeNode[] = []

  const visit = (node: TreeNode, depth: number) => {
    const partIds = partIdsForNode(node)
    const selectedCount = partIds.filter((id) => selectedSet.value.has(id)).length
    rows.push({
      id: node.id,
      label: node.label,
      meta: node.meta,
      depth,
      hasChildren: Boolean(node.children?.length),
      checkState:
        partIds.length > 0 && selectedCount === partIds.length ? 'checked' : selectedCount > 0 ? 'mixed' : 'empty',
    })
    node.children?.forEach((child) => visit(child, depth + 1))
  }

  props.nodes.forEach((node) => visit(node, 0))
  return rows
})

const partIdsForNode = (node: TreeNode): string[] => {
  if (node.partIds?.length) {
    return node.partIds
  }

  return node.children?.flatMap(partIdsForNode) ?? []
}

const treeWindow = computed(() =>
  calculateVirtualWindow(flatNodes.value.length, scrollOffset.value, viewportSize.value, TREE_ROW_STRIDE),
)
const renderedNodes = computed(() =>
  flatNodes.value.slice(treeWindow.value.start, treeWindow.value.end).map((node, offset) => ({
    node,
    index: treeWindow.value.start + offset,
  })),
)

const updateViewport = (event: Event) => {
  const element = event.currentTarget as HTMLElement
  scrollOffset.value = element.scrollTop
  viewportSize.value = element.clientHeight
}

watch(
  () => flatNodes.value.length,
  (length) => {
    activeIndex.value = Math.min(activeIndex.value, Math.max(0, length - 1))
  },
)

const focusRow = async (index: number) => {
  if (flatNodes.value.length === 0) return
  const nextIndex = Math.min(Math.max(index, 0), flatNodes.value.length - 1)
  activeIndex.value = nextIndex
  const element = listElement.value
  if (element && (nextIndex < treeWindow.value.start || nextIndex >= treeWindow.value.end)) {
    element.scrollTop = nextIndex * TREE_ROW_STRIDE
    scrollOffset.value = element.scrollTop
  }
  await nextTick()
  listElement.value?.querySelector<HTMLElement>(`[data-tree-index="${nextIndex}"]`)?.focus()
}

const handleKeydown = (event: KeyboardEvent, node: FlatTreeNode, index: number) => {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault()
    emit('toggle', node.id)
    return
  }

  let nextIndex: number | null = null
  if (event.key === 'ArrowDown') nextIndex = index + 1
  if (event.key === 'ArrowUp') nextIndex = index - 1
  if (event.key === 'Home') nextIndex = 0
  if (event.key === 'End') nextIndex = flatNodes.value.length - 1
  if (event.key === 'ArrowRight' && node.hasChildren && flatNodes.value[index + 1]?.depth > node.depth) {
    nextIndex = index + 1
  }
  if (event.key === 'ArrowLeft' && node.depth > 0) {
    for (let parentIndex = index - 1; parentIndex >= 0; parentIndex -= 1) {
      if (flatNodes.value[parentIndex]?.depth === node.depth - 1) {
        nextIndex = parentIndex
        break
      }
    }
  }

  if (nextIndex === null) return
  event.preventDefault()
  void focusRow(nextIndex)
}
</script>

<template>
  <div ref="tree-list" class="tree-list" role="tree" aria-label="内容选择" @scroll="updateViewport">
    <div class="tree-window" :style="{ height: `${treeWindow.totalSize}px` }">
      <div
        v-for="row in renderedNodes"
        :key="row.node.id"
        class="tree-row"
        :class="{ selected: row.node.checkState === 'checked', partial: row.node.checkState === 'mixed' }"
        role="treeitem"
        :data-tree-index="row.index"
        :aria-checked="row.node.checkState === 'mixed' ? 'mixed' : row.node.checkState === 'checked'"
        :aria-expanded="row.node.hasChildren ? true : undefined"
        :aria-level="row.node.depth + 1"
        :style="{
          paddingLeft: `${row.node.depth * 18 + 10}px`,
          transform: `translateY(${row.index * TREE_ROW_STRIDE}px)`,
        }"
        :tabindex="row.index === activeIndex ? 0 : -1"
        @focus="activeIndex = row.index"
        @click="emit('toggle', row.node.id)"
        @keydown="handleKeydown($event, row.node, row.index)"
      >
        <span class="tree-check" :class="row.node.checkState" aria-hidden="true"></span>
        <span class="tree-arrow" aria-hidden="true">{{ row.node.hasChildren ? '>' : '' }}</span>
        <span class="tree-label">{{ row.node.label }}</span>
        <span v-if="row.node.meta" class="tree-meta">{{ row.node.meta }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tree-list {
  min-height: 0;
  overflow: auto;
}

.tree-window {
  position: relative;
  min-width: 0;
}

.tree-row {
  position: absolute;
  inset: 0 0 auto;
  height: 34px;
  width: 100%;
  min-height: 34px;
  display: grid;
  grid-template-columns: 16px 14px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-8);
  padding-right: 10px;
  border: 1px solid transparent;
  border-radius: var(--radius-6);
  color: var(--color-text);
  cursor: pointer;
  font-size: var(--font-13);
  transition:
    border-color var(--duration-fast) var(--ease-out),
    background var(--duration-fast) var(--ease-out);
}

.tree-row:hover {
  border-color: color-mix(in oklab, var(--color-accent) 30%, var(--color-border));
  background: var(--color-hover-surface);
}

.tree-row:focus-visible {
  outline: 2px solid var(--color-focus-outline);
  outline-offset: 2px;
}

.tree-row.selected {
  border-color: color-mix(in oklab, var(--color-accent) 62%, var(--color-border));
  background: color-mix(in oklab, var(--color-accent-faint) 38%, var(--color-surface));
}

.tree-row.partial {
  border-color: color-mix(in oklab, var(--color-accent) 38%, var(--color-border));
  background: var(--color-hover-surface);
}

:global(:root[data-theme='dark']) .tree-row {
  border-color: color-mix(in oklab, var(--color-accent) 24%, var(--color-border));
  background: color-mix(in oklab, var(--color-accent-faint) 16%, var(--color-surface));
}

:global(:root[data-theme='dark']) .tree-row:hover {
  border-color: color-mix(in oklab, var(--color-accent) 50%, var(--color-border));
  background: color-mix(in oklab, var(--color-accent-faint) 34%, var(--color-surface));
}

:global(:root[data-theme='dark']) .tree-row.selected {
  border-color: var(--color-accent);
  background: color-mix(in oklab, var(--color-accent-faint) 68%, var(--color-surface));
}

:global(:root[data-theme='dark']) .tree-row.partial {
  border-color: color-mix(in oklab, var(--color-accent) 58%, var(--color-border));
  background: color-mix(in oklab, var(--color-accent-faint) 44%, var(--color-surface));
}

.tree-check {
  width: 14px;
  height: 14px;
  display: inline-grid;
  place-items: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-4);
  background: var(--color-surface);
}

.tree-check.checked,
.tree-check.mixed {
  border-color: var(--color-accent);
  background: var(--color-accent);
}

.tree-check.checked::after {
  content: '';
  width: 7px;
  height: 4px;
  border-left: 2px solid var(--color-on-accent);
  border-bottom: 2px solid var(--color-on-accent);
  transform: rotate(-45deg) translate(1px, -1px);
}

.tree-check.mixed::after {
  content: '';
  width: 8px;
  height: 2px;
  border-radius: 999px;
  background: var(--color-on-accent);
}

.tree-arrow {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.tree-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 600;
}

.tree-meta {
  color: var(--color-muted);
  font-size: var(--font-12);
}
</style>
