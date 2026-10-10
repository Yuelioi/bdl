<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { TaskActionKind, TransferTaskView } from '../../shared/stores/transferView';
import { calculateVirtualWindow } from '../../shared/utils/virtualWindow';
import { formatDuration } from '../../shared/utils/duration';
import UiCheckbox from '../../shared/ui/Checkbox.vue';
import UiButton from '../../shared/ui/Button.vue';
import MobileSheet from './MobileSheet.vue';
import MediaArtwork from './MediaArtwork.vue';
import MobileListCard from './MobileListCard.vue';
import { useQueueStore } from '../../shared/stores/queue';
import { useParseStore } from '../../shared/stores/parse';
const props = defineProps<{
  views: TransferTaskView[];
  selectedTaskIds: string[];
  managing: boolean;
  loading?: boolean;
}>();
const emit = defineEmits<{
  inspectTask: [id: string];
  toggleTaskSelection: [id: string];
  longPressTask: [id: string];
  taskAction: [id: string, action: Exclude<TaskActionKind, 'none'>];
}>();
const queue = useQueueStore();
const parse = useParseStore();
const artwork = computed(() => {
  const parts = new Map(
    Object.values(parse.sources).flatMap((source) =>
      source.groups.flatMap((group) =>
        group.items.flatMap((item) =>
          item.parts
            .filter((part) => part.cid != null)
            .map(
              (part) =>
                [part.cid, { src: item.cover_url, duration: part.duration_seconds ?? item.duration_seconds }] as const,
            ),
        ),
      ),
    ),
  );
  return new Map(
    queue.tasks.map((task) => {
      const metadata = task.refresh_intent ? parts.get(task.refresh_intent.cid) : undefined;
      return [
        task.id,
        {
          src:
            task.refresh_intent?.cover_url ??
            metadata?.src ??
            task.resources.find((resource) => resource.intent === 'cover')?.current_urls[0],
          duration: task.refresh_intent?.duration_seconds ?? metadata?.duration,
        },
      ];
    }),
  );
});
const offset = ref(0);
const height = ref(600);
const listCard = ref<InstanceType<typeof MobileListCard>>();
const scroller = computed(() => listCard.value?.element);
const rowHeight = ref(62);
let paddingTop = 0;
let paddingBottom = 0;
// Measure rendered rows and the card's content viewport, including after rotation.
const updateGeometry = () => {
  if (!scroller.value) return;
  const row = scroller.value.querySelector<HTMLElement>('.mobile-task');
  if (row) rowHeight.value = row.getBoundingClientRect().height;
  const style = getComputedStyle(scroller.value);
  paddingTop = parseFloat(style.paddingTop) || 0;
  paddingBottom = parseFloat(style.paddingBottom) || 0;
  height.value = scroller.value.clientHeight - paddingTop - paddingBottom;
  offset.value = Math.max(0, scroller.value.scrollTop - paddingTop);
};
let resizeObserver: ResizeObserver | undefined;
onMounted(() => {
  updateGeometry();
  resizeObserver = new ResizeObserver(updateGeometry);
  if (scroller.value) resizeObserver.observe(scroller.value);
});
watch(() => props.views.length, updateGeometry, { flush: 'post' });
onUnmounted(() => {
  resizeObserver?.disconnect();
  cancelPress();
});
const window = computed(() => calculateVirtualWindow(props.views.length, offset.value, height.value, rowHeight.value));
const visible = computed(() => props.views.slice(window.value.start, window.value.end));
const selected = computed(() => new Set(props.selectedTaskIds));
const actionTask = ref<TransferTaskView | null>(null);
const actionsOpen = ref(false);
let pressTimer: ReturnType<typeof setTimeout> | undefined;
let pressX = 0;
let pressY = 0;
let longPressedTaskId: string | null = null;
const cancelPress = () => {
  if (pressTimer) clearTimeout(pressTimer);
  pressTimer = undefined;
};
const startPress = (view: TransferTaskView, event: PointerEvent) => {
  if ((event.target as Element).closest('button, input, label')) return;
  cancelPress();
  pressX = event.clientX;
  pressY = event.clientY;
  pressTimer = setTimeout(() => {
    longPressedTaskId = view.id;
    emit('longPressTask', view.id);
  }, 420);
};
const movePress = (event: PointerEvent) => {
  if (Math.abs(event.clientX - pressX) > 8 || Math.abs(event.clientY - pressY) > 8) cancelPress();
};
const activateTask = (view: TransferTaskView) => {
  if (props.managing) emit('toggleTaskSelection', view.id);
  else emit('inspectTask', view.id);
};
const clickTask = (view: TransferTaskView, event: MouseEvent) => {
  if ((event.target as Element).closest('button, input, label')) return;
  if (longPressedTaskId === view.id) {
    longPressedTaskId = null;
    return;
  }
  activateTask(view);
};
const onScroll = (event: Event) => {
  cancelPress();
  const el = event.currentTarget as HTMLElement;
  offset.value = Math.max(0, el.scrollTop - paddingTop);
  height.value = el.clientHeight - paddingTop - paddingBottom;
};
const openActions = (view: TransferTaskView) => {
  actionTask.value = view;
  actionsOpen.value = true;
};
const run = (kind: Exclude<TaskActionKind, 'none'>) => {
  if (actionTask.value) emit('taskAction', actionTask.value.id, kind);
  actionsOpen.value = false;
};
const artworkActionLabel = (view: TransferTaskView) =>
  view.primaryAction === 'open_file' ? '播放' : view.primaryActionLabel;
const artworkActionIcon = (view: TransferTaskView) =>
  view.primaryAction === 'open_file' ? 'play' : view.primaryActionIcon;
const taskMetaLabel = (view: TransferTaskView) =>
  view.isCompleted
    ? [rowHeight.value < 62 ? view.statusLabel : '', view.subtitle || view.shortLocation].filter(Boolean).join(' · ')
    : view.statusLabel;
const taskMetaValue = (view: TransferTaskView) => {
  if (!view.isCompleted) {
    return [view.speedLabel === '--' ? '' : view.speedLabel, view.stageProgressLabel ?? view.progressLabel]
      .filter(Boolean)
      .join(' · ');
  }
  const duration = artwork.value.get(view.id)?.duration;
  return [
    rowHeight.value < 62 && duration != null && duration > 0 ? formatDuration(duration) : '',
    view.sizeLabel === '--' ? '' : view.sizeLabel,
  ]
    .filter(Boolean)
    .join(' · ');
};
</script>
<template>
  <MobileListCard ref="listCard" class="mobile-task-list" role="list" aria-label="传输任务" @scroll="onScroll">
    <div :style="window.virtualized ? { height: `${window.totalSize}px`, position: 'relative' } : undefined">
      <div
        :style="window.virtualized ? { position: 'absolute', top: `${window.offset}px`, left: 0, right: 0 } : undefined"
      >
        <article
          v-for="view in visible"
          :key="view.id"
          class="mobile-task"
          :class="{ selected: selected.has(view.id), managing, compact: rowHeight < 62 }"
          role="listitem"
          tabindex="0"
          @click="clickTask(view, $event)"
          @keydown.enter="activateTask(view)"
          @pointerdown="startPress(view, $event)"
          @pointermove="movePress"
          @pointerup="cancelPress"
          @pointercancel="cancelPress"
          @pointerleave="cancelPress"
          @contextmenu.prevent
        >
          <UiCheckbox
            v-if="managing"
            class="task-select"
            :model-value="selected.has(view.id)"
            :label="`选择 ${view.displayTitle}`"
            compact
            :disabled="loading"
            @update:model-value="emit('toggleTaskSelection', view.id)"
          />
          <div class="task-artwork-wrap">
            <MediaArtwork
              class="task-artwork"
              :src="artwork.get(view.id)?.src"
              :duration="rowHeight < 62 ? undefined : artwork.get(view.id)?.duration"
            />
            <button
              v-if="view.primaryAction !== 'none'"
              type="button"
              class="task-playback-action"
              :aria-label="artworkActionLabel(view)"
              :disabled="loading"
              @click="emit('taskAction', view.id, view.primaryAction)"
            >
              <UIcon :name="`i-tabler-${artworkActionIcon(view)}`" />
            </button>
            <span
              v-if="view.isCompleted"
              class="task-status-badge"
              :class="{ issue: view.statusBadge === 'warning' }"
              >{{ view.statusLabel }}</span
            >
          </div>
          <div class="task-heading">
            <span class="task-title">{{ view.displayTitle }}</span>
            <button
              class="mobile-icon-button"
              type="button"
              :aria-label="`${view.displayTitle}：更多操作`"
              :disabled="loading"
              @click="openActions(view)"
            >
              <UIcon name="i-tabler-menu-2" />
            </button>
          </div>
          <div class="task-meta">
            <span :class="{ issue: view.statusBadge === 'error' }">{{ taskMetaLabel(view) }}</span>
            <span>{{ taskMetaValue(view) }}</span>
          </div>
        </article>
      </div>
    </div>
  </MobileListCard>
  <MobileSheet v-model="actionsOpen" title="任务操作" :description="actionTask?.displayTitle">
    <template v-if="actionTask">
      <UiButton v-if="actionTask.primaryAction !== 'none'" :disabled="loading" @click="run(actionTask.primaryAction)">
        {{ actionTask.primaryActionLabel }}
      </UiButton>
      <UiButton
        variant="secondary"
        @click="
          emit('inspectTask', actionTask.id);
          actionsOpen = false;
        "
        >详情和诊断</UiButton
      >
      <UiButton
        v-for="action in actionTask.secondaryActions"
        :key="action.kind"
        variant="ghost"
        @click="run(action.kind)"
        >{{ action.label }}</UiButton
      >
    </template>
  </MobileSheet>
</template>
<style scoped>
.mobile-task-list > div,
.mobile-task-list > div > div {
  width: 100%;
  max-width: 100%;
  overflow-x: hidden;
}

.mobile-task {
  position: relative;
  border-bottom: 0.0625rem solid var(--color-border);
  height: var(--mobile-list-row-height);
  display: grid;
  grid-template-columns: var(--mobile-cover-width) minmax(0, 1fr);
  grid-template-rows: repeat(2, var(--mobile-list-line-height));
  align-content: center;
  gap: 0 var(--mobile-list-gap);
  padding: var(--mobile-list-padding) 0.75rem;
  background: var(--mobile-card);
  transition: background var(--duration-fast) var(--ease-out);
}

.mobile-task.selected {
  background: var(--color-accent-faint);
}

.mobile-task:last-child {
  border-bottom-color: transparent;
}

.mobile-task.managing {
  grid-template-columns: 1.375rem var(--mobile-cover-width) minmax(0, 1fr);
}

.task-select {
  grid-column: 1;
  grid-row: 1 / -1;
  place-self: center;
}

.task-heading {
  display: flex;
  align-items: center;
  min-height: var(--mobile-list-line-height);
  gap: 0;
}

.task-title {
  flex: 1;
  min-width: 0;
  border: 0;
  padding: 0;
  text-align: left;
  color: var(--color-text);
  background: transparent;
  font-weight: var(--mobile-weight-item);
  line-height: var(--mobile-list-line-height);
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
  padding-right: 0.3125rem;
  font-size: var(--mobile-font-item-title);
}

.task-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.625rem;
  font-size: var(--mobile-font-caption);
  color: var(--color-muted);
}

.task-meta > span:first-child {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-meta > span:last-child {
  flex-shrink: 0;
}

.issue {
  color: var(--color-danger);
}

.task-artwork-wrap {
  position: relative;
  width: var(--mobile-cover-width);
  height: var(--mobile-cover-height);
  grid-column: 1;
  grid-row: 1 / -1;
  align-self: center;
}

.task-artwork {
  width: 100%;
  height: 100%;
}

.task-playback-action {
  position: absolute;
  z-index: 1;
  left: 0.3125rem;
  top: calc((var(--mobile-cover-height) - 1.5rem) / 2);
  width: 1.5rem;
  height: 1.5rem;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 50%;
  background: rgb(17 24 39 / 72%);
  color: #fff;
}

.task-status-badge {
  position: absolute;
  top: 0.1875rem;
  right: 0.1875rem;
  padding: 0.0625rem 0.25rem;
  border-radius: 0.1875rem;
  background: rgb(17 24 39 / 72%);
  color: #fff;
  font-size: var(--mobile-font-micro);
  font-weight: var(--mobile-weight-item);
  line-height: 1.4;
}

.task-status-badge.issue {
  background: var(--color-danger);
  color: var(--color-on-accent);
}

.task-playback-action svg {
  width: 0.9375rem;
  height: 0.9375rem;
}

.task-heading,
.task-meta {
  grid-column: 2;
  min-width: 0;
  padding-right: 1.75rem;
}

.mobile-task.managing .task-artwork-wrap {
  width: var(--mobile-cover-width);
  grid-column: 2;
}

.mobile-task.managing .task-heading,
.mobile-task.managing .task-meta {
  grid-column: 3;
}

.task-heading > .mobile-icon-button {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 1.75rem;
  flex-basis: 1.75rem;
  height: 1.75rem;
  margin: 0;
}

.task-heading > .mobile-icon-button svg {
  width: 1.0625rem;
  height: 1.0625rem;
}

/* In compact rows, metadata carries status/duration so artwork remains actionable. */
.mobile-task.compact .task-status-badge {
  display: none;
}

.mobile-task.compact .task-playback-action {
  inset: 0;
  width: 100%;
  height: 100%;
  background: transparent;
}

.mobile-task.compact .task-playback-action svg {
  width: 1.5rem;
  height: 1.5rem;
  padding: 0.3125rem;
  border-radius: 50%;
  background: rgb(17 24 39 / 72%);
}
</style>
