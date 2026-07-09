<script setup lang="ts">
import { computed } from 'vue'

export interface TreeNode {
  id: string
  label: string
  meta?: string
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
}

const selectedSet = computed(() => new Set(props.selectedIds ?? []))

const flatNodes = computed(() => {
  const rows: FlatTreeNode[] = []

  const visit = (node: TreeNode, depth: number) => {
    rows.push({
      id: node.id,
      label: node.label,
      meta: node.meta,
      depth,
      hasChildren: Boolean(node.children?.length),
    })
    node.children?.forEach((child) => visit(child, depth + 1))
  }

  props.nodes.forEach((node) => visit(node, 0))
  return rows
})
</script>

<template>
  <div class="tree-list" role="tree" aria-label="解析结果">
    <div
      v-for="node in flatNodes"
      :key="node.id"
      class="tree-row"
      :class="{ selected: selectedSet.has(node.id) }"
      role="treeitem"
      :aria-selected="selectedSet.has(node.id)"
      :style="{ paddingLeft: `${node.depth * 18 + 10}px` }"
      tabindex="0"
      @click="emit('toggle', node.id)"
      @keydown.enter.prevent="emit('toggle', node.id)"
      @keydown.space.prevent="emit('toggle', node.id)"
    >
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
  grid-template-columns: 16px minmax(0, 1fr) auto;
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
