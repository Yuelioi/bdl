<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { NormalizedSourceTree } from '../api/dto';
import { collectionTree, treeSelection, type CollectionTreeNode } from '../features/parse/collectionTree';
import { useQueueStore } from '../stores/queue';
import { useUiStore } from '../stores/ui';
import { sourceMediaByPart, completedTaskForMedia } from '../utils/sourceMedia';
import { openExternalUrl } from '../api/tauri';
import UiCheckbox from './Checkbox.vue';
import MediaListRow from './MediaListRow.vue';

const props = defineProps<{
  source: NormalizedSourceTree;
  selectedIds: string[];
  visibleItemIds?: string[];
  query?: string;
  disabled?: boolean;
}>();
const emit = defineEmits<{ toggle: [ids: string[]] }>();
const queue = useQueueStore();
const ui = useUiStore();
const media = computed(() => sourceMediaByPart(props.source));
const details = (node: CollectionTreeNode) => media.value.get(node.partIds[0] ?? '');
const completed = (node: CollectionTreeNode) => {
  const details = media.value.get(node.partIds[0] ?? '');
  return details ? completedTaskForMedia(details, queue.tasks) : undefined;
};
async function play(node: CollectionTreeNode) {
  const task = completed(node);
  if (task) {
    await queue.openFile(task.id);
    return;
  }
  const url = media.value.get(node.partIds[0] ?? '')?.url;
  if (!url) return;
  try {
    await openExternalUrl(url);
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger');
  }
}
const nodes = computed(() => collectionTree(props.source, props.visibleItemIds, props.query));
const selected = computed(() => new Set(props.selectedIds));
const flattened = computed(() => {
  const visit = (items: CollectionTreeNode[]): CollectionTreeNode[] =>
    items.flatMap((item) => [item, ...visit(item.children ?? [])]);
  return visit(nodes.value);
});
const visibleNodes = computed(() => {
  const visit = (items: CollectionTreeNode[]): CollectionTreeNode[] =>
    items.flatMap((item) => [item, ...(expanded.value.includes(item.id) ? visit(item.children ?? []) : [])]);
  return visit(nodes.value);
});
const estimateSize = (index: number) => (['video', 'part'].includes(visibleNodes.value[index]?.kind ?? '') ? 96 : 36);
const selectedNodes = computed(() =>
  flattened.value.filter((node) => treeSelection(node.partIds, selected.value) === true),
);
const expanded = ref<string[]>([]);
watch(
  () => props.source.source.id,
  () => {
    expanded.value = [props.source.source.id];
  },
  { immediate: true },
);
watch(
  () => props.query,
  (query) => {
    if (query?.trim()) expanded.value = flattened.value.filter((item) => item.children?.length).map((item) => item.id);
  },
);
function toggle(node: CollectionTreeNode) {
  if (!props.disabled && node.partIds.length) emit('toggle', node.partIds);
}
function select(event: Event, node: CollectionTreeNode) {
  event.preventDefault();
  if (props.disabled) return;
  const original = (event as CustomEvent<{ originalEvent: Event }>).detail?.originalEvent;
  if (node.kind === 'video' || !node.children?.length || (original instanceof KeyboardEvent && original.key === ' '))
    toggle(node);
  else if (original instanceof KeyboardEvent && original.key === 'Enter') {
    expanded.value = expanded.value.includes(node.id)
      ? expanded.value.filter((id) => id !== node.id)
      : [...expanded.value, node.id];
  }
}
function expand(event: Event, node: CollectionTreeNode) {
  const original = (event as CustomEvent<{ originalEvent: Event }>).detail?.originalEvent;
  // Video rows retain click-to-select; their arrow and keyboard arrows expand parts.
  if (props.disabled || (node.kind === 'video' && original instanceof MouseEvent)) event.preventDefault();
}
</script>
<template>
  <section class="collection-tree" aria-label="合集分组">
    <UTree
      :key="visibleItemIds?.join('|') ?? source.source.id"
      v-model:expanded="expanded"
      class="collection-tree-list"
      :as="{ root: 'ul', link: 'div' }"
      :items="nodes"
      :get-key="(node: CollectionTreeNode) => node.id"
      :model-value="selectedNodes"
      multiple
      :disabled="disabled"
      :propagate-select="false"
      :bubble-select="false"
      :virtualize="flattened.length > 200 ? { estimateSize } : false"
      :ui="{ link: 'collection-tree-row', linkLabel: 'min-w-0 truncate', listWithChildren: 'ms-3 ps-2' }"
      @select="select"
      @toggle="expand"
    >
      <template #item="{ item, expanded: opened, handleToggle }">
        <MediaListRow
          v-if="item.kind === 'video' || item.kind === 'part'"
          :key="item.id"
          flat
          :title="item.label"
          :cover="details(item)?.cover"
          :secondary="details(item)?.secondary"
          :published="details(item)?.published"
          :duration="item.kind === 'video' ? details(item)?.item.duration_seconds : details(item)?.duration"
          :selected="treeSelection(item.partIds, selected) === true"
          :indeterminate="treeSelection(item.partIds, selected) === 'indeterminate'"
          :disabled="disabled"
          :playable="Boolean(details(item)?.url || completed(item))"
          :playback-label="`${completed(item) ? '播放本地文件' : '播放'} ${item.label}`"
          @click.stop
          @keydown.stop
          @toggle="toggle(item)"
          @play="play(item)"
        >
          <template v-if="item.children?.length" #facts>
            <UButton
              variant="ghost"
              color="neutral"
              :ui="{ base: 'p-0', label: 'contents' }"
              class="tree-expand"
              type="button"
              :aria-label="`${opened ? '收起' : '展开'} ${item.label}`"
              :aria-expanded="opened"
              :disabled="disabled"
              @click.stop="handleToggle()"
              @keydown.stop
            >
              <UIcon :name="opened ? 'i-tabler-chevron-down' : 'i-tabler-chevron-right'" />
            </UButton>
          </template>
        </MediaListRow>
        <div v-else class="tree-category">
          <UiCheckbox
            :key="item.id"
            :model-value="treeSelection(item.partIds, selected)"
            :label="`选择 ${item.label}`"
            :disabled="disabled || !item.partIds.length"
            compact
            @click.stop
            @keydown.stop
            @update:model-value="toggle(item)"
          />
          <UButton
            variant="ghost"
            color="neutral"
            :ui="{ base: 'p-0', label: 'contents' }"
            v-if="item.children?.length"
            class="tree-expand"
            type="button"
            :aria-label="`${opened ? '收起' : '展开'} ${item.label}`"
            :aria-expanded="opened"
            :disabled="disabled"
            @click.stop="handleToggle()"
            @keydown.stop
          >
            <UIcon :name="opened ? 'i-tabler-chevron-down' : 'i-tabler-chevron-right'" />
          </UButton>
          <strong class="tree-title" :title="item.label">{{ item.label }}</strong>
          <span v-if="item.kind === 'group'" class="tree-count">{{
            item.partIds.length ? item.meta : '尚未加载'
          }}</span>
        </div>
      </template>
    </UTree>
  </section>
</template>
<style scoped>
.collection-tree {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.collection-tree :deep(.collection-tree-list) {
  flex: 1;
  overflow: auto;
  min-height: 0;
  min-width: 0;
}

.collection-tree :deep(.collection-tree-row) {
  min-width: 0;
  padding-block: 0;
  padding-inline: 0;
  border-radius: 0;
  cursor: pointer;
}

.tree-category {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  min-width: 0;
  height: 2.25rem;
  padding-inline: 0.5rem;
}

.tree-expand {
  display: grid;
  place-items: center;
  width: 1.75rem;
  height: 2rem;
  flex-shrink: 0;
  border: 1px solid transparent;
  color: inherit;
  background: transparent;
  cursor: pointer;
}

.tree-expand:focus-visible {
  outline: none;
  border-color: var(--color-accent);
}

.tree-title {
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  font-size: var(--font-13);
}

.tree-count {
  flex-shrink: 0;
  color: var(--color-muted);
  font-size: var(--font-12);
}
</style>
