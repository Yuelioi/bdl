<script setup lang="ts">
import { computed } from 'vue'

import UiIconButton from './IconButton.vue'
import UiProgressBar from './ProgressBar.vue'
import UiStatusBadge from './StatusBadge.vue'

type TaskStatus = 'waiting' | 'parsing' | 'downloading' | 'muxing' | 'completed' | 'failed' | 'paused' | 'cancelled'

const statusLabel: Record<TaskStatus, string> = {
  waiting: '队列中',
  parsing: '解析中',
  downloading: '正在下载',
  muxing: '合并中',
  completed: '已完成',
  failed: '失败',
  paused: '已暂停',
  cancelled: '已取消',
}

const props = defineProps<{
  title: string
  status: TaskStatus
  progress: number
  quality: string
  audio: string
  codec: string
  speed: string
  eta: string
  stage: string
  path: string
  selected?: boolean
}>()

const emit = defineEmits<{
  pause: []
  resume: []
  cancel: []
  retry: []
  remove: []
  openFile: []
  openDir: []
}>()

const badgeStatus = computed(() => {
  if (props.status === 'completed') {
    return 'done'
  }
  if (props.status === 'failed' || props.status === 'cancelled') {
    return 'error'
  }
  if (props.status === 'paused') {
    return 'paused'
  }
  if (props.status === 'waiting') {
    return 'queued'
  }
  return 'downloading'
})

const canPause = computed(() => props.status === 'waiting' || props.status === 'parsing' || props.status === 'downloading')
const canResume = computed(() => props.status === 'paused')
const canRetry = computed(() => props.status === 'failed' || props.status === 'cancelled')
</script>

<template>
  <article class="task-row" :class="{ selected }">
    <div class="task-main">
      <div class="title-line">
        <h3>{{ title }}</h3>
        <UiStatusBadge :status="badgeStatus">{{ statusLabel[status] }}</UiStatusBadge>
        <span class="progress-text">{{ progress }}%</span>
        <div class="task-actions">
          <UiIconButton v-if="canPause" icon="pause" label="暂停" variant="ghost" @click.stop="emit('pause')" />
          <UiIconButton v-if="canResume" icon="play" label="恢复" variant="ghost" @click.stop="emit('resume')" />
          <UiIconButton v-if="canRetry" icon="refresh" label="重试" variant="ghost" @click.stop="emit('retry')" />
          <UiIconButton icon="x" label="取消" variant="ghost" @click.stop="emit('cancel')" />
          <UiIconButton icon="trash" label="移除" variant="ghost" @click.stop="emit('remove')" />
        </div>
      </div>
      <p>{{ quality }} · {{ audio }} · {{ codec }} · {{ speed }} · ETA {{ eta }} · {{ stage }}</p>
      <UiProgressBar :value="progress" />
    </div>
    <div class="task-side">
      <div class="task-path">{{ path }}</div>
      <div class="file-actions">
        <UiIconButton icon="file" label="打开文件" variant="ghost" @click.stop="emit('openFile')" />
        <UiIconButton icon="folder" label="打开目录" variant="ghost" @click.stop="emit('openDir')" />
      </div>
    </div>
  </article>
</template>

<style scoped>
.task-row {
  min-height: var(--height-task-row);
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(176px, 24%);
  align-items: center;
  gap: var(--space-16);
  padding: var(--space-8) var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
}

.task-row.selected {
  border-color: rgb(8 127 91 / 46%);
  background: #fbfdfc;
}

.task-main {
  min-width: 0;
  display: grid;
  gap: var(--space-6, 6px);
}

.title-line {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--space-8);
}

.task-actions,
.file-actions {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.task-actions {
  margin-left: auto;
  flex-shrink: 0;
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
.task-path,
.progress-text {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.progress-text {
  font-weight: 700;
}

.task-side {
  min-width: 0;
  display: grid;
  justify-items: end;
  gap: var(--space-4);
}

.task-actions :deep(.ui-icon-button),
.file-actions :deep(.ui-icon-button) {
  width: 28px;
  height: 28px;
}

.task-path {
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
}
</style>
