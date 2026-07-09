<script setup lang="ts">
import { computed } from 'vue'

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
</script>

<template>
  <div class="tree-list" role="tree" aria-label="解析结果">
    <div
      v-for="node in flatNodes"
      :key="node.id"
      class="tree-row"
      :class="{ selected: node.checkState === 'checked', partial: node.checkState === 'mixed' }"
      role="treeitem"
      :aria-checked="node.checkState === 'mixed' ? 'mixed' : node.checkState === 'checked'"
      :style="{ paddingLeft: `${node.depth * 18 + 10}px` }"
      tabindex="0"
      @click="emit('toggle', node.id)"
      @keydown.enter.prevent="emit('toggle', node.id)"
      @keydown.space.prevent="emit('toggle', node.id)"
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
  border-left: 2px solid #ffffff;
  border-bottom: 2px solid #ffffff;
  transform: rotate(-45deg) translate(1px, -1px);
}

.tree-check.mixed::after {
  content: "";
  width: 8px;
  height: 2px;
  border-radius: 999px;
  background: #ffffff;
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
