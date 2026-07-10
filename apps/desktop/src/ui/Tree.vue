<script setup lang="ts">
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

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
const rowElements = useTemplateRef<HTMLElement[]>('rows')

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
        partIds.length > 0 && selectedCount === partIds.length
          ? 'checked'
          : selectedCount > 0
            ? 'mixed'
            : 'empty',
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
  await nextTick()
  rowElements.value?.[nextIndex]?.focus()
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
  <div class="tree-list" role="tree" aria-label="解析结果">
    <div
      v-for="(node, index) in flatNodes"
      :key="node.id"
      ref="rows"
      class="tree-row"
      :class="{ selected: node.checkState === 'checked', partial: node.checkState === 'mixed' }"
      role="treeitem"
      :aria-checked="node.checkState === 'mixed' ? 'mixed' : node.checkState === 'checked'"
      :aria-expanded="node.hasChildren ? true : undefined"
      :aria-level="node.depth + 1"
      :style="{ paddingLeft: `${node.depth * 18 + 10}px` }"
      :tabindex="index === activeIndex ? 0 : -1"
      @focus="activeIndex = index"
      @click="emit('toggle', node.id)"
      @keydown="handleKeydown($event, node, index)"
    >
      <span class="tree-check" :class="node.checkState" aria-hidden="true"></span>
      <span class="tree-arrow" aria-hidden="true">{{ node.hasChildren ? ">" : "" }}</span>
      <span class="tree-label">{{ node.label }}</span>
      <span v-if="node.meta" class="tree-meta">{{ node.meta }}</span>
    </div>
  </div>
</template>

<style scoped>
.tree-list {
  min-height: 0;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: var(--space-4);
}

.tree-row {
  min-height: 34px;
  display: grid;
  grid-template-columns: 16px 14px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-8);
  padding-right: 10px;
  border-radius: var(--radius-6);
  color: var(--color-text);
  cursor: pointer;
  font-size: var(--font-13);
}

.tree-row:focus-visible {
  outline: 2px solid rgb(8 127 91 / 30%);
  outline-offset: 2px;
}

.tree-row.selected {
  background: #e8f3ee;
}

.tree-row.partial {
  background: #f1f7f4;
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
  content: "";
  width: 7px;
  height: 4px;
  border-left: 2px solid var(--color-on-accent);
  border-bottom: 2px solid var(--color-on-accent);
  transform: rotate(-45deg) translate(1px, -1px);
}

.tree-check.mixed::after {
  content: "";
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
