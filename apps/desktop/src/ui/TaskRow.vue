<script setup lang="ts">
import UiProgressBar from './ProgressBar.vue'
import UiStatusBadge from './StatusBadge.vue'

const statusLabel: Record<string, string> = {
  downloading: '正在下载',
  queued: '队列中',
  done: '已完成',
  warning: '需处理',
  error: '失败',
}

defineProps<{
  title: string
  status: 'downloading' | 'queued' | 'done' | 'warning' | 'error'
  progress: number
  meta: string
  path: string
}>()
</script>

<template>
  <article class="task-row">
    <div class="task-main">
      <div class="title-line">
        <h3>{{ title }}</h3>
        <UiStatusBadge :status="status">{{ statusLabel[status] }}</UiStatusBadge>
      </div>
      <p>{{ meta }}</p>
      <UiProgressBar :value="progress" />
    </div>
    <div class="task-path">{{ path }}</div>
  </article>
</template>

<style scoped>
.task-row {
  min-height: var(--height-task-row);
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(260px, 32%);
  align-items: center;
  gap: var(--space-16);
  padding: var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
}

.task-main {
  min-width: 0;
  display: grid;
  gap: var(--space-8);
}

.title-line {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--space-8);
}

h3,
p {
  margin: 0;
}

h3 {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-14);
}

p,
.task-path {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.task-path {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
}
</style>
