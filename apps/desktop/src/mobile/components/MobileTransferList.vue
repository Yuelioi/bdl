<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { TaskActionKind, TransferTaskView } from '../../shared/stores/transferView';
import { calculateVirtualWindow } from '../../shared/utils/virtualWindow';
import UiCheckbox from '../../shared/ui/Checkbox.vue';
import UiButton from '../../shared/ui/Button.vue';
import UiIconButton from '../../shared/ui/IconButton.vue';
import UiProgressBar from '../../shared/ui/ProgressBar.vue';
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
const rowHeight = ref(64);
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
const hasProgress = (view: TransferTaskView) =>
  ['downloading', 'paused'].includes(view.statusBadge) && view.stageProgressLabel !== '';
const taskMetaLabel = (view: TransferTaskView) =>
  view.isCompleted
    ? [view.statusLabel, view.subtitle || view.shortLocation].filter(Boolean).join(' · ')
    : hasProgress(view)
      ? `${view.statusLabel} · ${view.progressLabel}`
      : view.statusLabel;
const taskMetaValue = (view: TransferTaskView) =>
  view.isCompleted ? (view.sizeLabel === '--' ? '' : view.sizeLabel) : view.speedLabel === '--' ? '' : view.speedLabel;
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
          :class="{ selected: selected.has(view.id), managing }"
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
              :duration="artwork.get(view.id)?.duration"
            />
          </div>

          <div class="task-heading">
            <span class="task-title">{{ view.displayTitle }}</span>
          </div>
          <div class="task-row-actions">
            <UiIconButton
              v-if="view.primaryAction !== 'none'"
              :icon="artworkActionIcon(view)"
              :label="artworkActionLabel(view)"
              variant="ghost"
              size="compact"
              :disabled="loading"
              @click="emit('taskAction', view.id, view.primaryAction)"
            /><UiIconButton
              icon="dots"
              :label="`${view.displayTitle}：更多操作`"
              variant="ghost"
              size="compact"
              :disabled="loading"
              @click="openActions(view)"
            />
          </div>
          <div v-if="hasProgress(view)" class="task-row-progress">
            <UiProgressBar :value="view.progressValue" />
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
.mobile-task-list {
  flex: 1;
  padding: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.mobile-task-list > div,
.mobile-task-list > div > div {
  width: 100%;
  min-width: 0;
}

.mobile-task {
  position: relative;
  height: 4rem;
  min-width: 0;
  display: grid;
  grid-template-columns: 4rem minmax(0, 1fr) auto;
  grid-template-rows: 1.25rem 1.125rem;
  align-content: center;
  gap: 0.25rem 0.625rem;
  padding: 0.5rem;
  border: 1px solid transparent;
  border-bottom-color: var(--color-border);
  background: var(--color-surface);
}

.mobile-task.managing {
  grid-template-columns: 1.25rem 4rem minmax(0, 1fr) auto;
}

.mobile-task:focus-visible {
  outline: none;
  border-color: var(--color-accent);
}

.task-select {
  grid-column: 1;
  grid-row: 1 / -1;
  align-self: center;
}

.task-artwork-wrap {
  width: 4rem;
  height: 2.5rem;
  grid-column: 1;
  grid-row: 1 / -1;
  align-self: center;
}

.task-artwork {
  width: 100%;
  height: 100%;
}

.managing .task-artwork-wrap {
  grid-column: 2;
}

.task-heading,
.task-meta {
  min-width: 0;
  grid-column: 2;
}

.task-heading {
  grid-row: 1;
  display: flex;
  align-items: center;
}

.task-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  color: var(--color-text);
  font-size: var(--font-13);
  font-weight: 400;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.task-row-actions {
  grid-column: 3;
  grid-row: 1;
  align-self: center;
  display: flex;
  align-items: center;
  gap: 0.125rem;
}

.task-row-progress {
  position: absolute;
  right: 0.5rem;
  bottom: 0.25rem;
  left: calc(0.5rem + 4rem + 0.625rem);
}

.task-row-progress :deep(.progress-track) {
  height: 0.125rem;
}

.task-meta {
  grid-column: 2 / -1;
  grid-row: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.task-meta > span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-meta > span:first-child {
  min-width: 0;
  flex: 1;
}

.task-meta > span:last-child {
  flex-shrink: 0;
}

.task-meta .issue {
  color: var(--color-danger);
}

.managing .task-heading {
  grid-column: 3;
}

.managing .task-meta {
  grid-column: 3 / -1;
}

.managing .task-row-actions {
  grid-column: 4;
}

.managing .task-row-progress {
  left: calc(0.5rem + 1.25rem + 4rem + 1.25rem);
}
</style>
