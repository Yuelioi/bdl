<script setup lang="ts">
const {
  selectedCount,
  visibleCount,
  completedCount,
  canPause,
  canResume,
  canRetry,
  canRefreshRetry,
  canRemove,
  loading = false,
} = defineProps<{
  selectedCount: number
  visibleCount: number
  completedCount: number
  canPause: boolean
  canResume: boolean
  canRetry: boolean
  canRefreshRetry: boolean
  canRemove: boolean
  loading?: boolean
}>()

const emit = defineEmits<{
  pause: []
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
    <div class="bulk-copy">
      <strong v-if="selectedCount > 0">已选择 {{ selectedCount }} 个</strong>
      <strong v-else>当前 {{ visibleCount }} 个</strong>
      <span v-if="selectedCount > 0">批量操作只作用于可执行的任务状态。</span>
      <span v-else>可暂停、继续、清理已完成或刷新列表。</span>
    </div>

    <div v-if="selectedCount > 0" class="bulk-actions">
      <button type="button" :disabled="loading || !canPause" @click="emit('pause')">暂停</button>
      <button type="button" :disabled="loading || !canResume" @click="emit('resume')">继续</button>
      <button type="button" :disabled="loading || !canRetry" @click="emit('retry')">重试</button>
      <button type="button" :disabled="loading || !canRefreshRetry" @click="emit('refreshRetry')">
        刷新链接并重试
      </button>
      <button type="button" class="danger" :disabled="loading || !canRemove" @click="emit('remove')">移除</button>
    </div>

    <div v-else class="bulk-actions">
      <button type="button" :disabled="loading || !canPause" @click="emit('pause')">暂停全部</button>
      <button type="button" :disabled="loading || !canResume" @click="emit('resume')">继续全部</button>
      <button type="button" :disabled="loading || completedCount === 0" @click="emit('clearCompleted')">
        清理已完成
      </button>
      <button type="button" :disabled="loading" @click="emit('refresh')">刷新</button>
    </div>
  </div>
</template>

<style scoped>
.bulk-action-bar {
  min-width: 0;
  min-height: 40px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-12);
  padding: var(--space-8) var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

.bulk-action-bar.selected {
  border-color: rgb(8 127 91 / 28%);
  background: #f8fcfa;
}

.bulk-copy {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.bulk-copy strong,
.bulk-copy span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bulk-copy strong {
  color: var(--color-text);
  font-size: var(--font-13);
}

.bulk-copy span {
  color: var(--color-muted);
  font-size: var(--font-12);
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
  .bulk-action-bar {
    grid-template-columns: minmax(0, 1fr);
  }

  .bulk-actions {
    justify-content: flex-start;
  }
}
</style>
