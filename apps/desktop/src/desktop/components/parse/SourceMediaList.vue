<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { calculateVirtualWindow } from '../../../shared/utils/virtualWindow';
import type { NormalizedSourceTree } from '../../../shared/api/dto';
import MediaListRow from '../../../shared/ui/MediaListRow.vue';
import UiCheckbox from '../../../shared/ui/Checkbox.vue';
import { useQueueStore } from '../../../shared/stores/queue';
import { useUiStore } from '../../../shared/stores/ui';
import { formatBytes } from '../../../shared/stores/transferView';
import { openExternalUrl } from '../../../shared/api/tauri';
import { sourceMediaByPart, completedTaskForMedia } from '../../../shared/utils/sourceMedia';
import type { ParseResultRow } from '../../../shared/features/parse/parseResultTree';

const props = defineProps<{
  rows: ParseResultRow[];
  source: NormalizedSourceTree;
  selectedIds: string[];
  selectedRowIds?: string[];
  disabled?: boolean;
  fallbackOwner?: string | null;
  selectionLabel?: string;
  label?: string;
  compact?: boolean;
}>();
const emit = defineEmits<{ toggle: [id: string]; toggleAll: [] }>();
const queue = useQueueStore();
const ui = useUiStore();
const selected = computed(() => new Set(props.selectedIds));
const selectedRows = computed(() => new Set(props.selectedRowIds));
const allIds = computed(() => props.rows.flatMap((row) => row.partIds));
const selection = computed<boolean | 'indeterminate'>(() => {
  if (props.selectedRowIds) {
    const count = props.rows.filter((row) => selectedRows.value.has(row.id)).length;
    return count === 0 ? false : count === props.rows.length ? true : 'indeterminate';
  }
  const count = allIds.value.filter((id) => selected.value.has(id)).length;
  return count === 0 ? false : count === allIds.value.length ? true : 'indeterminate';
});
const media = computed(() => sourceMediaByPart(props.source, props.fallbackOwner));
const rowStride = computed(() => (props.compact ? 48 : 72));
const scrollElement = ref<HTMLElement | null>(null);
const scrollOffset = ref(0);
const viewportSize = ref(600);
const listWindow = computed(() =>
  calculateVirtualWindow(props.rows.length, scrollOffset.value, viewportSize.value, rowStride.value),
);
let observer: ResizeObserver | undefined;
onMounted(() => {
  if (!scrollElement.value || typeof ResizeObserver === 'undefined') return;
  observer = new ResizeObserver(() => {
    viewportSize.value = scrollElement.value?.clientHeight || 600;
  });
  observer.observe(scrollElement.value);
});
onBeforeUnmount(() => observer?.disconnect());
function updateViewport() {
  if (!scrollElement.value) return;
  scrollOffset.value = scrollElement.value.scrollTop;
  viewportSize.value = scrollElement.value.clientHeight;
}
watch(
  () => props.source.source.id,
  () => {
    scrollOffset.value = 0;
    if (scrollElement.value) scrollElement.value.scrollTop = 0;
  },
);
const partIndices = computed(
  () =>
    new Map(
      props.source.groups.flatMap((group) =>
        group.items.flatMap((item) => item.parts.map((part, index) => [part.id, `P${index + 1}`] as const)),
      ),
    ),
);
const entries = computed(() =>
  props.rows.slice(listWindow.value.start, listWindow.value.end).map((row, offset) => {
    const details = media.value.get(row.partIds[0] ?? '');
    const completed = details ? completedTaskForMedia(details, queue.tasks) : undefined;
    const progress = completed ? queue.taskTransferProgress(completed.id) : null;
    return {
      ...row,
      index: listWindow.value.start + offset,
      partIndex: props.compact ? partIndices.value.get(row.partIds[0] ?? '') : undefined,
      details,
      completed,
      selected: props.selectedRowIds
        ? selectedRows.value.has(row.id)
        : row.partIds.length > 0 && row.partIds.every((id) => selected.value.has(id)),
      size:
        completed && queue.outputSizesByTask[completed.id]?.bytes != null
          ? formatBytes(queue.outputSizesByTask[completed.id]!.bytes!)
          : progress && progress.downloadedBytes > 0
            ? formatBytes(progress.downloadedBytes)
            : null,
    };
  }),
);
watch(
  () => entries.value.map((entry) => entry.completed?.id ?? '').join('\n'),
  () => {
    void queue.loadOutputSizes(entries.value.flatMap((entry) => (entry.completed ? [entry.completed.id] : [])));
  },
  { immediate: true },
);
async function play(entry: (typeof entries.value)[number]) {
  if (entry.completed) {
    await queue.openFile(entry.completed.id);
  } else if (entry.details?.url) {
    try {
      await openExternalUrl(entry.details.url);
    } catch (error) {
      ui.pushToast(error instanceof Error ? error.message : String(error), 'danger');
    }
  }
}
</script>

<template>
  <section class="source-media-list" :class="{ compact }" :aria-label="label || '解析结果'">
    <div class="source-list-toolbar">
      <UiCheckbox
        :model-value="selection"
        :label="selectionLabel || '全选已加载'"
        :disabled="disabled || !rows.length"
        @update:model-value="emit('toggleAll')"
      />
      <span v-if="selectedIds.length" class="selected-count">已选 {{ selectedIds.length }} 项</span>
    </div>
    <div
      ref="scrollElement"
      class="source-list-body"
      role="list"
      :aria-label="label || '解析结果'"
      @scroll="updateViewport"
    >
      <div
        v-if="listWindow.virtualized"
        class="source-list-spacer"
        aria-hidden="true"
        :style="{ height: `${listWindow.offset}px` }"
      />
      <MediaListRow
        v-for="entry in entries"
        :key="entry.id"
        class="source-media-row"
        :class="{ 'is-virtual': listWindow.virtualized }"
        :aria-posinset="entry.index + 1"
        :aria-setsize="rows.length"
        :title="entry.title"
        :compact="compact"
        :index="entry.partIndex"
        :cover="entry.details?.cover"
        :secondary="entry.details?.secondary || entry.meta"
        :published="entry.details?.published"
        :duration="entry.details?.duration"
        :size-label="compact ? null : entry.size"
        :selected="entry.selected"
        :disabled="disabled"
        :playable="Boolean(entry.completed || entry.details?.url)"
        :playback-label="entry.completed ? `播放本地文件 ${entry.title}` : `播放 ${entry.title}`"
        @toggle="!disabled && emit('toggle', entry.id)"
        @play="play(entry)"
      >
        <template v-if="entry.completed && !compact" #detail><span class="downloaded-label">已下载</span></template>
        <template v-if="$slots.actions" #actions><slot name="actions" :row="entry" /></template>
      </MediaListRow>
      <div
        v-if="listWindow.virtualized"
        class="source-list-spacer"
        aria-hidden="true"
        :style="{ height: `${(rows.length - listWindow.end) * rowStride}px` }"
      />
    </div>
  </section>
</template>

<style scoped>
.source-media-list {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.source-list-toolbar {
  min-height: 2rem;
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 1rem;
  padding: 0.125rem 0.125rem 0.375rem;
  color: var(--color-muted);
  font-size: var(--font-12);
}

.selected-count {
  color: var(--color-accent-strong);
}

.source-list-body {
  min-height: 0;
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
  scrollbar-gutter: stable;
}

.source-media-row {
  flex-shrink: 0;
  margin: 0;
  height: 72px;
}

.compact .source-media-row {
  height: 48px;
}

.source-list-spacer {
  flex-shrink: 0;
}

.downloaded-label {
  color: var(--color-success);
  font-size: var(--font-11);
}
</style>
