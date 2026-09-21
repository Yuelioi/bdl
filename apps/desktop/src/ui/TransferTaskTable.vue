<script setup lang="ts">
import { computed, ref } from 'vue'

import type { TaskActionKind, TransferTaskView } from '../stores/transferView'
import { isMobilePlatform } from '../utils/platform'
import UiButton from './Button.vue'
import UiIconButton from './IconButton.vue'
import UiProgressBar from './ProgressBar.vue'
import UiStatusBadge from './StatusBadge.vue'
import UiCheckbox from './Checkbox.vue'
import TaskActionMenu from './TaskActionMenu.vue'
import { calculateVirtualWindow } from '../utils/virtualWindow'

const DESKTOP_TASK_ROW_STRIDE = 62
const MOBILE_COMPLETED_ROW_STRIDE = 150
const MOBILE_TRANSFER_ROW_STRIDE = 194

const {
  views,
  selectedTaskId,
  selectedTaskIds,
  loading = false,
  mode = 'transfer',
} = defineProps<{
  views: TransferTaskView[]
  selectedTaskId: string | null
  selectedTaskIds: string[]
  loading?: boolean
  mode?: 'transfer' | 'completed'
}>()

const emit = defineEmits<{
  inspectTask: [taskId: string]
  toggleTaskSelection: [taskId: string]
  toggleVisibleSelection: [taskIds: string[], selected: boolean]
  taskAction: [taskId: string, action: Exclude<TaskActionKind, 'none'>]
  openContextMenu: [taskId: string, event: MouseEvent]
}>()

const visibleIds = computed(() => views.map((view) => view.id))
const mobilePlatform = isMobilePlatform()
const taskRowStride = computed(() =>
  mobilePlatform
    ? mode === 'completed'
      ? MOBILE_COMPLETED_ROW_STRIDE
      : MOBILE_TRANSFER_ROW_STRIDE
    : DESKTOP_TASK_ROW_STRIDE,
)
const selectedSet = computed(() => new Set(selectedTaskIds))
const selectedVisibleCount = computed(() => visibleIds.value.filter((taskId) => selectedSet.value.has(taskId)).length)
const allVisibleSelected = computed(
  () => visibleIds.value.length > 0 && visibleIds.value.every((taskId) => selectedSet.value.has(taskId)),
)
const scrollOffset = ref(0)
const viewportSize = ref(600)
const taskWindow = computed(() =>
  calculateVirtualWindow(views.length, scrollOffset.value, viewportSize.value, taskRowStride.value),
)
const renderedViews = computed(() =>
  views.slice(taskWindow.value.start, taskWindow.value.end).map((view, offset) => ({
    view,
    index: taskWindow.value.start + offset,
  })),
)

const updateViewport = (event: Event) => {
  const element = event.currentTarget as HTMLElement
  scrollOffset.value = element.scrollTop
  viewportSize.value = element.clientHeight
}

const toggleVisible = () => {
  emit('toggleVisibleSelection', visibleIds.value, !allVisibleSelected.value)
}

const hasSecondaryAction = (view: TransferTaskView, action: Exclude<TaskActionKind, 'none'>) =>
  view.secondaryActions.some((candidate) => candidate.kind === action)
</script>

<template>
  <div
    class="transfer-table"
    :class="[`${mode}-mode`, { 'mobile-card-mode': mobilePlatform }]"
    :style="{ '--task-row-stride': `${taskRowStride}px` }"
    role="table"
    aria-label="传输任务"
    :aria-rowcount="views.length + 1"
    @scroll="updateViewport"
  >
    <div v-if="!mobilePlatform" class="transfer-table-row table-head" role="row">
      <div class="select-cell" role="columnheader">
        <UiCheckbox
          :model-value="allVisibleSelected"
          label="选择当前筛选任务"
          compact
          :disabled="!views.length"
          @update:model-value="toggleVisible"
        />
      </div>
      <div role="columnheader">名称</div>
      <div role="columnheader">状态</div>
      <div v-if="mode === 'completed'" role="columnheader">输出位置</div>
      <template v-else>
        <div role="columnheader">进度</div>
        <div role="columnheader">速度</div>
        <div role="columnheader">剩余</div>
      </template>
      <div class="action-head" role="columnheader">操作</div>
    </div>

    <div v-else class="mobile-selection-bar">
      <UiCheckbox
        :model-value="allVisibleSelected"
        label="全选"
        :disabled="!views.length || loading"
        @update:model-value="toggleVisible"
      />
      <span class="mobile-selection-count">已选 {{ selectedVisibleCount }} / {{ views.length }}</span>
    </div>

    <div class="virtual-task-list" :style="{ height: `${taskWindow.totalSize}px` }">
      <div
        v-for="row in renderedViews"
        :key="row.view.id"
        class="transfer-table-row task-table-row"
        :class="{ selected: selectedTaskId === row.view.id }"
        :style="{ transform: `translateY(${row.index * taskRowStride}px)` }"
        role="row"
        :aria-rowindex="row.index + 2"
        :aria-selected="selectedTaskId === row.view.id"
        @contextmenu.prevent="emit('openContextMenu', row.view.id, $event)"
      >
        <template v-for="view in [row.view]" :key="view.id">
          <template v-if="mobilePlatform">
            <div class="mobile-task-card">
              <div class="mobile-card-head">
                <span class="select-cell" @click.stop @keydown.stop>
                  <UiCheckbox
                    :model-value="selectedSet.has(view.id)"
                    :label="selectedSet.has(view.id) ? '取消选择' : '选择任务'"
                    compact
                    :disabled="loading"
                    @update:model-value="emit('toggleTaskSelection', view.id)"
                  />
                </span>
                <span class="mobile-card-title">
                  <strong :title="view.displayTitle">{{ view.displayTitle }}</strong>
                  <small v-if="view.subtitle" :title="view.subtitle">{{ view.subtitle }}</small>
                  <small v-if="view.issueLabel !== '-'" class="issue-line" :title="view.issueLabel">{{
                    view.issueLabel
                  }}</small>
                </span>
                <UiStatusBadge :status="view.statusBadge">{{ view.statusLabel }}</UiStatusBadge>
              </div>

              <div v-if="mode === 'completed'" class="mobile-card-location" :title="view.fullLocation">
                <UIcon name="i-tabler-folder" aria-hidden="true" />
                <span>{{ view.shortLocation }}</span>
              </div>
              <template v-else>
                <div class="mobile-card-progress">
                  <div>
                    <strong>{{ view.isCompleted ? '已完成' : view.progressLabel }}</strong>
                    <small v-if="!view.isCompleted">{{ view.sizeLabel }}</small>
                  </div>
                  <UiProgressBar v-if="!view.isCompleted" :value="view.progressValue" />
                </div>
                <div class="mobile-card-metrics">
                  <span>速度 {{ view.isCompleted ? '—' : view.speedLabel }}</span>
                  <span>剩余 {{ view.isCompleted ? '—' : view.etaLabel }}</span>
                </div>
              </template>

              <div class="mobile-card-actions" @click.stop @keydown.stop>
                <UiButton variant="secondary" size="compact" :disabled="loading" @click="emit('inspectTask', view.id)">
                  <UIcon name="i-tabler-info-circle" aria-hidden="true" />
                  详情
                </UiButton>
                <UiButton
                  v-if="view.primaryAction !== 'none'"
                  variant="secondary"
                  size="compact"
                  :disabled="loading"
                  @click="emit('taskAction', view.id, view.primaryAction)"
                >
                  {{ view.primaryActionLabel }}
                </UiButton>
                <UiButton
                  v-if="mode === 'completed' && hasSecondaryAction(view, 'open_dir')"
                  variant="secondary"
                  size="compact"
                  :disabled="loading"
                  @click="emit('taskAction', view.id, 'open_dir')"
                >
                  <UIcon name="i-tabler-folder-open" aria-hidden="true" />
                  文件夹
                </UiButton>
                <TaskActionMenu
                  :view
                  :disabled="loading"
                  :show-primary="false"
                  :exclude-actions="mode === 'completed' ? ['open_dir'] : []"
                  @action="(action) => emit('taskAction', view.id, action)"
                />
              </div>
            </div>
          </template>

          <template v-else>
          <span class="select-cell" role="cell" @click.stop @keydown.stop>
            <UiCheckbox
              :model-value="selectedSet.has(view.id)"
              :label="selectedSet.has(view.id) ? '取消选择' : '选择任务'"
              compact
              :disabled="loading"
              @update:model-value="emit('toggleTaskSelection', view.id)"
            />
          </span>

          <span class="title-cell" role="cell">
            <strong :title="view.displayTitle">{{ view.displayTitle }}</strong>
            <small v-if="view.subtitle" :title="view.subtitle">{{ view.subtitle }}</small>
            <small v-if="view.issueLabel !== '-'" class="issue-line" :title="view.issueLabel">{{
              view.issueLabel
            }}</small>
          </span>

          <span class="status-cell" role="cell">
            <UiStatusBadge :status="view.statusBadge">{{ view.statusLabel }}</UiStatusBadge>
          </span>

          <span v-if="mode === 'completed'" class="location-cell" role="cell" :title="view.fullLocation">
            <UIcon name="i-tabler-folder" aria-hidden="true" />
            {{ view.shortLocation }}
          </span>
          <template v-else>
            <span class="progress-cell" role="cell">
              <template v-if="!view.isCompleted">
                <strong>{{ view.progressLabel }}</strong>
                <small>{{ view.sizeLabel }}</small>
                <UiProgressBar :value="view.progressValue" />
              </template>
              <span v-else class="completed-result">已完成</span>
            </span>
            <span class="metric-cell" role="cell">{{ view.isCompleted ? '—' : view.speedLabel }}</span>
            <span class="metric-cell" role="cell">{{ view.isCompleted ? '—' : view.etaLabel }}</span>
          </template>
          <span class="action-cell" role="cell" @click.stop @keydown.stop>
            <UiIconButton
              icon="info"
              label="详情和诊断"
              variant="ghost"
              size="compact"
              :disabled="loading"
              @click="emit('inspectTask', view.id)"
            />
            <TaskActionMenu :view :disabled="loading" @action="(action) => emit('taskAction', view.id, action)" />
          </span>
          </template>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.transfer-table {
  --transfer-table-min-width: 760px;
  --task-row-stride: 62px;

  min-width: 0;
  min-height: 0;
  height: 100%;
  display: grid;
  align-content: start;
  gap: var(--space-4);
  overflow: auto;
  overscroll-behavior: contain;
  padding-right: 2px;
}

.transfer-table-row {
  width: max(100%, var(--transfer-table-min-width));
  min-width: var(--transfer-table-min-width);
  display: grid;
  grid-template-columns:
    30px minmax(240px, 1fr) 72px minmax(112px, 0.34fr) 74px 56px
    96px;
  column-gap: var(--space-8);
  align-items: center;
}

.virtual-task-list {
  position: relative;
  width: max(100%, var(--transfer-table-min-width));
  min-width: var(--transfer-table-min-width);
}

.completed-mode {
  --transfer-table-min-width: 680px;
}

.completed-mode .transfer-table-row {
  grid-template-columns: 30px minmax(260px, 1fr) 86px minmax(140px, 0.42fr) 96px;
}

.table-head {
  position: sticky;
  top: 0;
  z-index: var(--z-sticky);
  min-height: 34px;
  padding: 0 var(--space-8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.task-table-row {
  position: absolute;
  inset: 0 0 auto;
  height: calc(var(--task-row-stride) - 4px);
  min-height: calc(var(--task-row-stride) - 4px);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
  color: var(--color-text);
  padding: var(--space-8);
  text-align: left;
}

.task-table-row:hover {
  background: var(--color-panel);
}

.task-table-row.selected {
  border-color: color-mix(in oklab, var(--color-accent) 58%, var(--color-border));
  background: var(--color-selected-surface);
}

.task-table-row:focus-visible {
  outline: 2px solid var(--color-focus-outline);
  outline-offset: 2px;
}

.select-cell {
  display: inline-grid;
  place-items: center;
}

.title-cell,
.progress-cell {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.title-cell strong,
.title-cell small,
.metric-cell {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.title-cell strong {
  font-size: var(--font-13);
  line-height: 1.25;
}

.title-cell small {
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.2;
}

.status-cell {
  min-width: 0;
}

.progress-cell strong,
.progress-cell small,
.metric-cell {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.progress-cell strong {
  color: var(--color-text);
  font-weight: 700;
}

.completed-result {
  color: var(--color-success);
  font-size: var(--font-12);
  font-weight: 700;
}

.location-cell {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--space-4);
  overflow: hidden;
  color: var(--color-muted);
  font-size: var(--font-12);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.location-cell svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
}

.progress-cell small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.progress-cell :deep(.progress-track) {
  height: 6px;
}

.issue-line {
  color: var(--color-danger);
  font-weight: 650;
}

.action-head,
.action-cell {
  justify-self: end;
}

.action-cell {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 2px;
}

.mobile-card-mode {
  --transfer-table-min-width: 0px;

  gap: 0;
  overflow-x: hidden;
  padding-right: 0;
}

.mobile-card-mode .virtual-task-list,
.mobile-card-mode .task-table-row {
  width: 100%;
  min-width: 0;
}

.mobile-card-mode .virtual-task-list {
  margin-top: var(--space-8);
}

.mobile-card-mode .task-table-row {
  display: block;
  padding: var(--space-10) var(--space-12);
}

.mobile-selection-bar {
  position: sticky;
  top: 0;
  z-index: var(--z-sticky);
  min-height: 46px;
  display: flex;
  align-items: center;
  gap: var(--space-8);
  padding: 0 var(--space-10);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.mobile-selection-count {
  margin-left: auto;
  color: var(--color-muted);
  font-size: var(--font-11);
  font-variant-numeric: tabular-nums;
}

.mobile-task-card {
  min-width: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--space-8);
}

.mobile-card-head {
  min-width: 0;
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr) auto;
  align-items: start;
  gap: var(--space-8);
}

.mobile-card-head .select-cell {
  min-height: 28px;
  align-self: center;
}

.mobile-card-title {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.mobile-card-title strong,
.mobile-card-title small {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mobile-card-title strong {
  color: var(--color-text);
  font-size: var(--font-13);
  line-height: 1.35;
}

.mobile-card-title small {
  color: var(--color-muted);
  font-size: var(--font-11);
}

.mobile-card-location {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--space-6);
  color: var(--color-muted);
  font-size: var(--font-12);
}

.mobile-card-location svg {
  width: 16px;
  height: 16px;
  flex: 0 0 auto;
}

.mobile-card-location span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mobile-card-progress {
  min-width: 0;
  display: grid;
  gap: var(--space-6);
}

.mobile-card-progress > div {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-8);
}

.mobile-card-progress strong {
  color: var(--color-text);
  font-size: var(--font-12);
}

.mobile-card-progress small,
.mobile-card-metrics {
  color: var(--color-muted);
  font-size: var(--font-11);
}

.mobile-card-metrics {
  display: flex;
  align-items: center;
  gap: var(--space-12);
}

.mobile-card-actions {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--space-8);
  margin-top: auto;
  padding-top: var(--space-6);
  border-top: 1px solid var(--color-border);
}

.mobile-card-actions :deep(.ui-button) {
  min-height: 36px;
  padding-inline: var(--space-10);
}

.mobile-card-actions :deep(.ui-icon-button) {
  width: 36px;
  height: 36px;
  min-width: 36px;
  min-height: 36px;
}

.mobile-card-actions :deep(svg) {
  width: 16px;
  height: 16px;
}

@media (width <= 620px) {
  .transfer-table:not(.mobile-card-mode) {
    --transfer-table-min-width: 720px;

    scrollbar-width: thin;
    -webkit-overflow-scrolling: touch;
  }

  .completed-mode:not(.mobile-card-mode) {
    --transfer-table-min-width: 640px;
  }

  .transfer-table:not(.mobile-card-mode) .action-head,
  .transfer-table:not(.mobile-card-mode) .action-cell {
    position: sticky;
    right: 0;
    z-index: 2;
    padding-left: var(--space-xs);
    background: var(--color-panel);
    box-shadow: -10px 0 16px -16px var(--color-shadow-strong);
  }

  .transfer-table:not(.mobile-card-mode) .task-table-row .action-cell {
    background: var(--color-surface);
  }

  .transfer-table:not(.mobile-card-mode) .task-table-row:hover .action-cell {
    background: var(--color-panel);
  }

  .transfer-table:not(.mobile-card-mode) .task-table-row.selected .action-cell {
    background: var(--color-selected-surface);
  }
}
</style>
