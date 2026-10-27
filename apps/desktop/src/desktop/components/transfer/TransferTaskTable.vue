<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import type { TaskActionKind, TransferTaskView } from '../../../shared/stores/transferView';
import { coverUrl } from '../../../shared/utils/coverUrl';
import UiCheckbox from '../../../shared/ui/Checkbox.vue';
import UiIconButton from '../../../shared/ui/IconButton.vue';
import UiProgressBar from '../../../shared/ui/ProgressBar.vue';
import UiStatusBadge from '../../../shared/ui/StatusBadge.vue';
import TaskActionMenu from '../../../shared/ui/TaskActionMenu.vue';
import { calculateVirtualWindow } from '../../../shared/utils/virtualWindow';

const ROW_STRIDE = 64;
const props = withDefaults(
  defineProps<{
    views: TransferTaskView[];
    selectedTaskId: string | null;
    selectedTaskIds: string[];
    loading?: boolean;
    mode?: 'transfer' | 'completed';
  }>(),
  { loading: false, mode: 'transfer' },
);
const emit = defineEmits<{
  inspectTask: [taskId: string];
  toggleTaskSelection: [taskId: string];
  toggleVisibleSelection: [taskIds: string[], selected: boolean];
  taskAction: [taskId: string, action: Exclude<TaskActionKind, 'none'>];
  openContextMenu: [taskId: string, event: MouseEvent];
  visibleTasks: [taskIds: string[]];
}>();
const visibleIds = computed(() => props.views.map((view) => view.id));
const selected = computed(() => new Set(props.selectedTaskIds));
const selection = computed<boolean | 'indeterminate'>(() => {
  const count = visibleIds.value.filter((id) => selected.value.has(id)).length;
  return count === 0 ? false : count === visibleIds.value.length ? true : 'indeterminate';
});
const scrollOffset = ref(0);
const viewportSize = ref(600);
const scrollElement = ref<HTMLElement | null>(null);
let resizeObserver: ResizeObserver | undefined;
onMounted(() => {
  if (!scrollElement.value || typeof ResizeObserver === 'undefined') return;
  resizeObserver = new ResizeObserver(() => {
    if (scrollElement.value) viewportSize.value = scrollElement.value.clientHeight;
  });
  resizeObserver.observe(scrollElement.value);
});
onBeforeUnmount(() => resizeObserver?.disconnect());
const taskWindow = computed(() =>
  calculateVirtualWindow(props.views.length, scrollOffset.value, viewportSize.value, ROW_STRIDE),
);
const renderedViews = computed(() =>
  props.views.slice(taskWindow.value.start, taskWindow.value.end).map((view, offset) => ({
    view,
    index: taskWindow.value.start + offset,
  })),
);
const updateViewport = (event: Event) => {
  const element = event.currentTarget as HTMLElement;
  scrollOffset.value = element.scrollTop;
  viewportSize.value = element.clientHeight;
};
watch(
  () => renderedViews.value.map(({ view }) => `${view.id}:${view.isCompleted}`).join('\n'),
  () =>
    emit(
      'visibleTasks',
      renderedViews.value.map(({ view }) => view.id),
    ),
  { immediate: true },
);
const canPlay = (view: TransferTaskView) =>
  view.primaryAction === 'open_file' || (!view.isCompleted && Boolean(view.sourceUrl));
const play = (view: TransferTaskView) => emit('taskAction', view.id, view.isCompleted ? 'open_file' : 'open_source');
const hasProgress = (view: TransferTaskView) =>
  ['downloading', 'paused'].includes(view.statusBadge) && view.stageProgressLabel !== '';
const openDirectory = (view: TransferTaskView) => view.secondaryActions.find((action) => action.kind === 'open_dir');
</script>

<template>
  <section class="transfer-table" aria-label="传输任务">
    <div class="transfer-columns transfer-selection-bar">
      <UiCheckbox
        :model-value="selection"
        label="选择当前筛选任务"
        compact
        :disabled="!views.length || loading"
        @update:model-value="emit('toggleVisibleSelection', visibleIds, selection !== true)"
      />
      <span>文件</span><span>进度</span><span>状态</span><span class="speed-column">速度</span
      ><span class="actions-label">操作</span>
    </div>
    <div ref="scrollElement" class="transfer-scroll" @scroll="updateViewport">
      <div class="virtual-task-list" role="list" aria-label="传输任务" :style="{ height: `${taskWindow.totalSize}px` }">
        <article
          v-for="row in renderedViews"
          :key="row.view.id"
          class="transfer-columns task-table-row"
          :class="{ inspected: selectedTaskId === row.view.id }"
          :style="{ transform: `translateY(${row.index * ROW_STRIDE}px)` }"
          :aria-posinset="row.index + 1"
          :aria-setsize="views.length"
          role="listitem"
          @contextmenu.prevent="emit('openContextMenu', row.view.id, $event)"
        >
          <UiCheckbox
            :model-value="selected.has(row.view.id)"
            :label="`选择 ${row.view.displayTitle}`"
            compact
            :disabled="loading"
            @update:model-value="emit('toggleTaskSelection', row.view.id)"
          />
          <div class="task-file">
            <UButton
              class="task-cover"
              variant="ghost"
              color="neutral"
              :ui="{ base: 'p-0', label: 'contents' }"
              :aria-label="
                row.view.isCompleted ? `播放本地文件 ${row.view.displayTitle}` : `播放 ${row.view.displayTitle}`
              "
              :disabled="!canPlay(row.view) || loading"
              @click="play(row.view)"
            >
              <img
                v-if="row.view.coverUrl"
                :src="coverUrl(row.view.coverUrl) ?? undefined"
                alt=""
                referrerpolicy="no-referrer"
                loading="lazy"
              /><UIcon v-else name="i-tabler-video" aria-hidden="true" />
            </UButton>
            <div class="task-file-copy">
              <UButton
                variant="link"
                color="neutral"
                class="task-file-title"
                :ui="{ base: 'p-0', label: 'truncate' }"
                :title="row.view.displayTitle"
                :aria-label="`选择 ${row.view.displayTitle}`"
                :aria-pressed="selected.has(row.view.id)"
                :disabled="loading"
                @click="emit('toggleTaskSelection', row.view.id)"
                >{{ row.view.displayTitle }}</UButton
              >
              <span class="task-subtitle" :title="row.view.isCompleted ? row.view.fullLocation : row.view.mediaLabel">{{
                row.view.isCompleted ? row.view.shortLocation : row.view.mediaLabel || row.view.subtitle
              }}</span>
            </div>
          </div>
          <div class="task-progress">
            <template v-if="hasProgress(row.view)"
              ><div class="progress-value">
                <UiProgressBar :value="row.view.progressValue" /><span>{{ row.view.progressLabel }}</span>
              </div>
              <small v-if="row.view.sizeLabel !== '--'">{{ row.view.sizeLabel }}</small></template
            >
            <span v-else>{{ row.view.isCompleted ? row.view.sizeLabel : '—' }}</span>
          </div>
          <div class="task-state">
            <UiStatusBadge :status="row.view.statusBadge">{{ row.view.statusLabel }}</UiStatusBadge
            ><span v-if="row.view.issueLabel !== '-'" class="task-issue" :title="row.view.issueLabel">{{
              row.view.issueLabel
            }}</span
            ><small class="compact-speed" v-if="row.view.speedLabel !== '--'">{{ row.view.speedLabel }}</small>
          </div>
          <span class="task-speed speed-column">{{ row.view.speedLabel === '--' ? '—' : row.view.speedLabel }}</span>
          <div class="task-actions">
            <UiIconButton
              v-if="row.view.primaryAction !== 'none' && row.view.primaryAction !== 'open_file'"
              :icon="row.view.primaryActionIcon"
              :label="row.view.primaryActionLabel"
              variant="ghost"
              size="compact"
              :disabled="loading"
              @click="emit('taskAction', row.view.id, row.view.primaryAction)"
            />
            <UiIconButton
              v-else-if="row.view.isCompleted && openDirectory(row.view)"
              icon="folder"
              label="打开所在目录"
              variant="ghost"
              size="compact"
              :disabled="loading"
              @click="emit('taskAction', row.view.id, 'open_dir')"
            />
            <TaskActionMenu
              :view="row.view"
              :disabled="loading"
              :show-primary="false"
              show-inspect
              @inspect="emit('inspectTask', row.view.id)"
              @action="(action) => emit('taskAction', row.view.id, action)"
            />
          </div>
        </article>
      </div>
    </div>
  </section>
</template>
<style scoped>
.transfer-table {
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.transfer-columns {
  display: grid;
  grid-template-columns: 1.25rem minmax(0, 2.5fr) minmax(7rem, 1.1fr) minmax(5.5rem, 0.8fr) 5.25rem 4rem;
  align-items: center;
  gap: 0.75rem;
}

.transfer-selection-bar {
  flex-shrink: 0;
  min-height: 2rem;
  padding: 0 0.25rem;
  color: var(--color-muted);
  background: var(--color-inset);
  font-size: var(--font-12);
}

.actions-label {
  text-align: center;
}

.transfer-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  scrollbar-gutter: stable;
}

.virtual-task-list {
  position: relative;
  min-width: 0;
}

.task-table-row {
  position: absolute;
  inset: 0 0 auto;
  height: 64px;
  padding: 0 0.25rem;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.task-table-row:hover,
.task-table-row.inspected {
  background: var(--color-hover-surface);
}

.task-file {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.task-cover {
  width: 3rem;
  height: 1.875rem;
  flex-shrink: 0;
  display: grid;
  place-items: center;
  overflow: hidden;
  border-radius: var(--radius-4);
  background: var(--color-inset);
  color: var(--color-muted);
}

.task-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.task-file-copy {
  min-width: 0;
  display: grid;
  gap: 0.125rem;
}

.task-file-title {
  min-width: 0;
  display: block;
  overflow: hidden;
  font-size: var(--font-13);
  font-weight: 400;
  line-height: 1.25rem;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text);
}

.task-subtitle {
  min-width: 0;
  overflow: hidden;
  font-size: var(--font-11);
  color: var(--color-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-progress {
  min-width: 0;
  display: grid;
  gap: 0.125rem;
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.progress-value {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.progress-value > span {
  min-width: 2.25rem;
  text-align: right;
}

.task-progress :deep(.progress-track) {
  min-width: 1rem;
  height: 0.3125rem;
  flex: 1;
}

.task-progress small {
  font-size: var(--font-11);
}

.task-state {
  min-width: 0;
  display: grid;
  justify-items: start;
  gap: 0.125rem;
}

.task-state :deep(.status-badge) {
  padding: 0;
  background: transparent;
  font-weight: 400;
}

.task-issue {
  max-width: 100%;
  overflow: hidden;
  font-size: var(--font-11);
  color: var(--color-danger);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-speed {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.task-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0;
}

.compact-speed {
  display: none;
}

@media (width <= 920px) {
  .transfer-columns {
    grid-template-columns: 1.25rem minmax(0, 2fr) minmax(5rem, 1fr) 5rem 4rem;
    gap: 0.5rem;
  }

  .speed-column {
    display: none;
  }

  .compact-speed {
    display: block;
    color: var(--color-muted);
    font-size: var(--font-11);
  }
}
</style>
