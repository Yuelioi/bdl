<script setup lang="ts">
import { computed } from 'vue'

import type { TaskActionKind, TransferTaskView } from '../stores/transferView'
import UiIconButton from './IconButton.vue'
import UiProgressBar from './ProgressBar.vue'
import UiStatusBadge from './StatusBadge.vue'
import TaskActionMenu from './TaskActionMenu.vue'

const { views, selectedTaskId, selectedTaskIds, loading = false } = defineProps<{
  views: TransferTaskView[]
  selectedTaskId: string | null
  selectedTaskIds: string[]
  loading?: boolean
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

const toggleVisible = () => {
  emit('toggleVisibleSelection', visibleIds.value, !allVisibleSelected.value)
}
</script>

<template>
  <div class="transfer-table" role="table" aria-label="传输任务">
    <div class="transfer-table-row table-head" role="row">
      <div class="select-cell" role="columnheader">
        <label class="row-check" title="选择当前筛选任务">
          <input type="checkbox" :checked="allVisibleSelected" :disabled="!views.length" @change="toggleVisible" />
          <span aria-hidden="true" />
        </label>
      </div>
      <div role="columnheader">名称</div>
      <div role="columnheader">状态</div>
      <div role="columnheader">进度</div>
      <div role="columnheader">速度</div>
      <div role="columnheader">剩余</div>
      <div class="action-head" role="columnheader">操作</div>
    </div>

    <div
      v-for="view in views"
      :key="view.id"
      class="transfer-table-row task-table-row"
      :class="{ selected: selectedTaskId === view.id }"
      role="row"
      :aria-selected="selectedTaskId === view.id"
      @contextmenu.prevent="emit('openContextMenu', view.id, $event)"
    >
      <span class="select-cell" role="cell" @click.stop @keydown.stop>
        <label class="row-check" :title="selectedSet.has(view.id) ? '取消选择' : '选择任务'">
          <input
            type="checkbox"
            :checked="selectedSet.has(view.id)"
            :disabled="loading"
            @change="emit('toggleTaskSelection', view.id)"
          />
          <span aria-hidden="true" />
        </label>
      </span>

      <span class="title-cell" role="cell">
        <strong :title="view.displayTitle">{{ view.displayTitle }}</strong>
        <small v-if="view.subtitle" :title="view.subtitle">{{ view.subtitle }}</small>
        <small v-if="view.issueLabel !== '-'" class="issue-line" :title="view.issueLabel">{{ view.issueLabel }}</small>
      </span>

      <span class="status-cell" role="cell">
        <UiStatusBadge :status="view.statusBadge">{{ view.statusLabel }}</UiStatusBadge>
      </span>

      <span class="progress-cell" role="cell">
        <strong>{{ view.progressLabel }}</strong>
        <small>{{ view.sizeLabel }}</small>
        <UiProgressBar :value="view.progressValue" />
      </span>

      <span class="metric-cell" role="cell">{{ view.speedLabel }}</span>
      <span class="metric-cell" role="cell">{{ view.etaLabel }}</span>
      <span class="action-cell" role="cell" @click.stop @keydown.stop>
        <UiIconButton
          icon="info"
          label="详情和诊断"
          variant="ghost"
          :disabled="loading"
          @click="emit('inspectTask', view.id)"
        />
        <TaskActionMenu
          :view
          :disabled="loading"
          @action="(action) => emit('taskAction', view.id, action)"
        />
      </span>
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
  overflow-x: hidden;
  overflow-y: auto;
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

.table-head {
  position: sticky;
  top: 0;
  z-index: 5;
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
  border-color: rgb(8 127 91 / 46%);
  background: #fbfdfc;
}

.task-table-row:focus-visible {
  outline: 2px solid rgb(8 127 91 / 30%);
  outline-offset: 2px;
}

.select-cell {
  display: inline-grid;
  place-items: center;
}

.row-check {
  width: 18px;
  height: 18px;
  display: inline-grid;
  place-items: center;
  cursor: pointer;
}

.row-check input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.row-check span {
  width: 16px;
  height: 16px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-4);
  background: var(--color-surface);
}

.row-check input:checked + span {
  border-color: var(--color-accent);
  background: var(--color-accent);
}

.row-check input:checked + span::after {
  content: "";
  display: block;
  width: 8px;
  height: 5px;
  margin: 3px 0 0 3px;
  border-left: 2px solid var(--color-on-accent);
  border-bottom: 2px solid var(--color-on-accent);
  transform: rotate(-45deg);
}

.row-check input:focus-visible + span {
  outline: 2px solid rgb(8 127 91 / 30%);
  outline-offset: 2px;
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

.action-cell :deep(.ui-icon-button) {
  width: 28px;
  height: 28px;
  min-width: 28px;
  min-height: 28px;
  padding: 0;
}

.action-cell :deep(.ui-icon-button svg) {
  width: 15px;
  height: 15px;
}
</style>
