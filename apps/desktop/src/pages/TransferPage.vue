<script setup lang="ts">
import { computed, onMounted } from 'vue'

import type { DownloadTask, TaskStatus } from '../api/dto'
import { useQueueStore, type QueueFilter } from '../stores/queue'
import UiButton from '../ui/Button.vue'
import UiStatusBadge from '../ui/StatusBadge.vue'
import UiTabs from '../ui/Tabs.vue'
import UiTaskRow from '../ui/TaskRow.vue'

const queue = useQueueStore()

const tabs = computed(() => [
  { label: '正在下载', value: 'downloading', count: countByFilter('downloading') },
  { label: '队列中', value: 'queued', count: countByFilter('queued') },
  { label: '已暂停', value: 'paused', count: countByFilter('paused') },
  { label: '失败', value: 'failed', count: countByFilter('failed') },
  { label: '已完成', value: 'completed', count: countByFilter('completed') },
  { label: '全部', value: 'all', count: queue.tasks.length },
])

const selectedLogs = computed(() => (queue.selectedTaskId ? (queue.logsByTask[queue.selectedTaskId] ?? []) : []))

onMounted(() => {
  void queue.startEventListeners()
  void queue.list()
})

const countByFilter = (filter: QueueFilter): number =>
  queue.tasks.filter((task) => {
    if (filter === 'all') {
      return true
    }
    if (filter === 'downloading') {
      return task.status === 'downloading' || task.status === 'parsing' || task.status === 'muxing'
    }
    if (filter === 'queued') {
      return task.status === 'waiting'
    }
    if (filter === 'failed') {
      return task.status === 'failed' || task.status === 'cancelled'
    }
    return task.status === filter
  }).length

const resourceMeta = (task: DownloadTask): string => {
  const videoCount = task.resources.filter((resource) => resource.intent === 'video').length
  const audioCount = task.resources.filter((resource) => resource.intent === 'audio').length
  const assetCount = task.resources.filter((resource) => resource.kind === 'asset').length
  return `视频 ${videoCount} · 音频 ${audioCount} · 资源 ${assetCount}`
}

const stageText = (status: TaskStatus): string => {
  const labels: Record<TaskStatus, string> = {
    waiting: '等待开始',
    parsing: '解析中',
    downloading: '下载中',
    muxing: '合并中',
    completed: '已完成',
    failed: '失败',
    paused: '已暂停',
    cancelled: '已取消',
  }
  return labels[status]
}
</script>

<template>
  <section class="page-grid transfer-page">
    <section class="panel transfer-main">
      <div class="panel-heading">
        <div>
          <h2>任务</h2>
          <span class="muted-text">{{ queue.filteredTasks.length }} 个</span>
        </div>
        <UiButton variant="secondary" :disabled="queue.loading" @click="queue.list">刷新</UiButton>
      </div>

      <UiTabs v-model="queue.activeFilter" :tabs="tabs" />

      <div v-if="queue.filteredTasks.length" class="task-list">
        <UiTaskRow
          v-for="task in queue.filteredTasks"
          :key="task.id"
          :title="task.title"
          :status="task.status"
          :progress="queue.taskProgress(task)"
          quality="最佳"
          :audio="task.resources.some((resource) => resource.intent === 'audio') ? '音频' : '无音频'"
          codec="自动"
          speed="--"
          eta="--"
          :stage="stageText(task.status)"
          :path="task.output_path"
          :selected="queue.selectedTaskId === task.id"
          @click="queue.selectTask(task.id)"
          @pause="queue.pause(task.id)"
          @resume="queue.resume(task.id)"
          @cancel="queue.cancel(task.id)"
          @retry="queue.retry(task.id)"
          @remove="queue.remove(task.id)"
          @open-file="queue.openFile(task.id)"
          @open-dir="queue.openDir(task.id)"
        />
      </div>
      <div v-else class="empty-state">暂无任务</div>
    </section>

    <aside class="panel transfer-detail">
      <div class="panel-heading">
        <h2>详情</h2>
        <UiStatusBadge v-if="queue.selectedTask" status="ready">{{ stageText(queue.selectedTask.status) }}</UiStatusBadge>
      </div>

      <template v-if="queue.selectedTask">
        <div class="detail-block">
          <strong>{{ queue.selectedTask.title }}</strong>
          <span>{{ resourceMeta(queue.selectedTask) }}</span>
          <span>{{ queue.selectedTask.output_path }}</span>
        </div>

        <div class="log-list">
          <div v-for="log in selectedLogs" :key="`${log.created_at}-${log.message}`" class="log-row">
            <span>{{ log.level }}</span>
            <p>{{ log.message }}</p>
          </div>
          <div v-if="!selectedLogs.length" class="empty-state compact">暂无日志</div>
        </div>
      </template>
      <div v-else class="empty-state">未选择任务</div>
    </aside>
  </section>
</template>

<style scoped>
.transfer-page {
  grid-template-columns: minmax(0, 1fr) 320px;
}

.transfer-main,
.transfer-detail {
  min-height: 0;
}

.transfer-main {
  overflow: hidden;
}

.task-list {
  min-height: 0;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: var(--space-8);
}

.detail-block {
  display: grid;
  gap: var(--space-8);
}

.detail-block strong,
.detail-block span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail-block span {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.log-list {
  min-height: 0;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: var(--space-8);
}

.log-row {
  display: grid;
  gap: var(--space-4);
  padding: var(--space-8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-panel);
}

.log-row span {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.log-row p {
  margin: 0;
  font-size: var(--font-12);
  line-height: 1.5;
}

.empty-state {
  min-height: 120px;
  display: grid;
  place-items: center;
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
  color: var(--color-muted);
  font-size: var(--font-12);
}

.empty-state.compact {
  min-height: 72px;
}
</style>
