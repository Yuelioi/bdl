<script setup lang="ts">
const {
  selectedCount,
  completedCount,
  canPause,
  canCancel,
  canResume,
  canRetry,
  canRefreshRetry,
  canRemove,
  loading = false,
} = defineProps<{
  selectedCount: number
  completedCount: number
  canPause: boolean
  canCancel: boolean
  canResume: boolean
  canRetry: boolean
  canRefreshRetry: boolean
  canRemove: boolean
  loading?: boolean
}>()

const emit = defineEmits<{
  pause: []
  cancel: []
  resume: []
  retry: []
  refreshRetry: []
  remove: []
  clearCompleted: []
  refresh: []
}>()
</script>

<template>
  <div class="bulk-action-bar" :class="{ selected: selectedCount > 0 }">
    <div v-if="selectedCount > 0" class="bulk-copy">
      <strong>已选择 {{ selectedCount }} 个</strong>
    </div>

    <div v-if="selectedCount > 0" class="bulk-actions">
      <button type="button" :disabled="loading || !canPause" @click="emit('pause')">暂停</button>
      <button type="button" :disabled="loading || !canCancel" @click="emit('cancel')">取消</button>
      <button type="button" :disabled="loading || !canResume" @click="emit('resume')">继续</button>
      <button type="button" :disabled="loading || !canRetry" @click="emit('retry')">重试</button>
      <button type="button" :disabled="loading || !canRefreshRetry" @click="emit('refreshRetry')">
        刷新链接并重试
      </button>
      <button type="button" class="danger" :disabled="loading || !canRemove" @click="emit('remove')">移除</button>
    </div>

    <div v-else class="bulk-actions">
      <button type="button" :disabled="loading || completedCount === 0" @click="emit('clearCompleted')">
        清理已完成
      </button>
      <button type="button" :disabled="loading" @click="emit('refresh')">刷新</button>
      <details class="bulk-more">
        <summary>更多</summary>
        <div>
          <button type="button" :disabled="loading || !canPause" @click="emit('pause')">暂停全部</button>
          <button type="button" :disabled="loading || !canCancel" @click="emit('cancel')">取消全部</button>
          <button type="button" :disabled="loading || !canResume" @click="emit('resume')">继续全部</button>
        </div>
      </details>
    </div>
  </div>
</template>

<style scoped>
.bulk-action-bar {
  min-width: 0;
  min-height: 32px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-12);
}

.bulk-action-bar.selected {
  min-height: 40px;
  justify-content: space-between;
  padding: var(--space-8) var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  border-color: rgb(8 127 91 / 28%);
  background: #f8fcfa;
}

.bulk-copy {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.bulk-copy strong {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text);
  font-size: var(--font-13);
}

.bulk-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-4);
}

.bulk-actions button {
  height: 28px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-text);
  padding: 0 var(--space-8);
  font-size: var(--font-12);
  font-weight: 650;
  white-space: nowrap;
}

.bulk-more {
  position: relative;
}

.bulk-more summary {
  height: 28px;
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-muted);
  padding: 0 var(--space-8);
  font-size: var(--font-12);
  font-weight: 650;
  white-space: nowrap;
  cursor: pointer;
  list-style: none;
}

.bulk-more summary::-webkit-details-marker {
  display: none;
}

.bulk-more[open] summary {
  background: var(--color-panel);
}

.bulk-more div {
  position: absolute;
  top: calc(100% + var(--space-4));
  right: 0;
  z-index: 20;
  min-width: 120px;
  display: grid;
  gap: var(--space-4);
  padding: var(--space-4);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
  box-shadow: 0 12px 28px rgb(23 33 29 / 12%);
}

.bulk-more div button {
  width: 100%;
  justify-content: flex-start;
  text-align: left;
}

.bulk-actions button:hover:not(:disabled) {
  background: var(--color-panel);
}

.bulk-actions button:focus-visible {
  outline: 2px solid rgb(8 127 91 / 30%);
  outline-offset: 2px;
}

.bulk-actions button:disabled {
  opacity: 0.48;
}

.bulk-actions button.danger {
  border-color: rgb(201 42 42 / 28%);
  color: var(--color-danger);
}

@media (max-width: 1180px) {
  .bulk-action-bar.selected {
    align-items: flex-start;
    flex-direction: column;
  }

  .bulk-actions {
    justify-content: flex-start;
  }
}
</style>
