<script setup lang="ts">
import { computed } from 'vue'

import UiCheckbox from '../../ui/Checkbox.vue'
import type { ParseResultRow } from './parseResultTree'

const props = defineProps<{
  rows: ParseResultRow[]
  selectedIds: string[]
  disabled?: boolean
}>()
const emit = defineEmits<{ toggle: [id: string]; toggleAll: [] }>()

const selectedSet = computed(() => new Set(props.selectedIds))
const allPartIds = computed(() => props.rows.flatMap((row) => row.partIds))
const selectedPartCount = computed(() => allPartIds.value.filter((id) => selectedSet.value.has(id)).length)
const allSelected = computed(
  () => allPartIds.value.length > 0 && selectedPartCount.value === allPartIds.value.length,
)
const headerSelection = computed<boolean | 'indeterminate'>(() =>
  allSelected.value ? true : selectedPartCount.value > 0 ? 'indeterminate' : false,
)

const rowSelected = (row: ParseResultRow): boolean =>
  row.partIds.length > 0 && row.partIds.every((id) => selectedSet.value.has(id))

const handleRowKeydown = (event: KeyboardEvent, rowId: string) => {
  if (event.key !== 'Enter' && event.key !== ' ') return
  event.preventDefault()
  emit('toggle', rowId)
}
</script>

<template>
  <div class="parse-result-table" role="table" aria-label="解析结果">
    <div class="parse-result-table-header" role="row">
      <div class="selection-cell" role="columnheader">
        <UiCheckbox
          :model-value="headerSelection"
          label="全选已加载"
          :disabled="disabled || rows.length === 0"
          compact
          @update:model-value="emit('toggleAll')"
        />
      </div>
      <div role="columnheader">序号</div>
      <div role="columnheader">标题</div>
      <div class="meta-column" role="columnheader">UP 主</div>
    </div>

    <div class="parse-result-table-body" role="rowgroup">
      <div
        v-for="(row, index) in rows"
        :key="row.id"
        class="parse-result-table-row"
        :class="{ selected: rowSelected(row) }"
        role="row"
        tabindex="0"
        @click="emit('toggle', row.id)"
        @keydown="handleRowKeydown($event, row.id)"
      >
        <div class="selection-cell" role="cell">
          <UiCheckbox
            :model-value="rowSelected(row)"
            :label="`选择 ${row.title}`"
            :disabled
            compact
            @click.stop
            @update:model-value="emit('toggle', row.id)"
          />
        </div>
        <div class="sequence-cell" role="cell">{{ String(index + 1).padStart(2, '0') }}</div>
        <div class="title-cell" role="cell" :title="row.title">{{ row.title }}</div>
        <div class="meta-cell" role="cell" :title="row.meta">{{ row.meta || '—' }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.parse-result-table {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: 34px minmax(0, 1fr);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  overflow: hidden;
}

.parse-result-table-header,
.parse-result-table-row {
  min-width: 0;
  display: grid;
  grid-template-columns: 36px 56px minmax(0, 1fr) minmax(100px, 160px);
  align-items: center;
  column-gap: var(--space-xs);
  padding-inline: var(--space-xs);
}

.parse-result-table-header {
  color: var(--color-muted);
  background: var(--color-panel);
  font-size: var(--font-12);
  font-weight: 650;
}

.parse-result-table-body {
  min-height: 0;
  overflow-y: auto;
  padding-inline-end: var(--space-xs);
  scrollbar-gutter: stable;
}

.parse-result-table-row {
  min-height: 40px;
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text);
  cursor: pointer;
  font-size: var(--font-13);
  transition: background var(--duration-fast) var(--ease-out);
}

.parse-result-table-row:last-child {
  border-bottom: 0;
}

.parse-result-table-row:hover {
  background: var(--color-hover-surface);
}

.parse-result-table-row.selected {
  background: var(--color-accent-faint);
}

.parse-result-table-row:focus-visible {
  outline: 2px solid var(--color-focus-outline);
  outline-offset: -2px;
}

.selection-cell {
  display: flex;
  align-items: center;
  justify-content: center;
}

.sequence-cell {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.title-cell,
.meta-cell {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.title-cell {
  font-weight: 650;
}

.meta-column,
.meta-cell {
  text-align: right;
}

.meta-cell {
  color: var(--color-muted);
  font-size: var(--font-12);
}

@media (width <= 700px) {
  .parse-result-table-header,
  .parse-result-table-row {
    grid-template-columns: 32px 44px minmax(0, 1fr) 96px;
    column-gap: var(--space-2xs);
  }
}

@media (prefers-reduced-motion: reduce) {
  .parse-result-table-row {
    transition: none;
  }
}
</style>
