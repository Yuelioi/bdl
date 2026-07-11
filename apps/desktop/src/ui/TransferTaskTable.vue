<script setup lang="ts">
import { computed, ref } from 'vue'

import type { TaskActionKind, TransferTaskView } from '../stores/transferView'
import UiIconButton from './IconButton.vue'
import UiProgressBar from './ProgressBar.vue'
import UiStatusBadge from './StatusBadge.vue'
import UiCheckbox from './Checkbox.vue'
import TaskActionMenu from './TaskActionMenu.vue'
import { calculateVirtualWindow } from '../utils/virtualWindow'

const TASK_ROW_STRIDE = 62

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
const selectedSet = computed(() => new Set(selectedTaskIds))
const allVisibleSelected = computed(
  () => visibleIds.value.length > 0 && visibleIds.value.every((taskId) => selectedSet.value.has(taskId)),
)
const scrollOffset = ref(0)
const viewportSize = ref(600)
const taskWindow = computed(() =>
  calculateVirtualWindow(views.length, scrollOffset.value, viewportSize.value, TASK_ROW_STRIDE),
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
</script>

<template>
  <div
    class="transfer-table"
    :class="`${mode}-mode`"
    role="table"
    aria-label="传输任务"
    :aria-rowcount="views.length + 1"
    @scroll="updateViewport"
  >
    <div class="transfer-table-row table-head" role="row">
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

    <div class="virtual-task-list" :style="{ height: `${taskWindow.totalSize}px` }">
      <div
        v-for="row in renderedViews"
        :key="row.view.id"
        class="transfer-table-row task-table-row"
        :class="{ selected: selectedTaskId === row.view.id }"
        :style="{ transform: `translateY(${row.index * TASK_ROW_STRIDE}px)` }"
        role="row"
        :aria-rowindex="row.index + 2"
        :aria-selected="selectedTaskId === row.view.id"
        @contextmenu.prevent="emit('openContextMenu', row.view.id, $event)"
      >
        <template v-for="view in [row.view]" :key="view.id">
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
      </div>
    </div>
  </div>
</template>

<style scoped>
.transfer-table {
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: grid;
  align-content: start;
  gap: var(--space-4);
  overflow: hidden auto;
  padding-right: 2px;
}

.transfer-table-row {
  min-width: 0;
  display: grid;
  grid-template-columns:
    30px minmax(240px, 1fr) 72px minmax(112px, 0.34fr) 74px 56px
    96px;
  column-gap: var(--space-8);
  align-items: center;
}

.virtual-task-list {
  position: relative;
  min-width: 0;
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
  height: 58px;
  min-height: 58px;
  width: 100%;
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
</style>
