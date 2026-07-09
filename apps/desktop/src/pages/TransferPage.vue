<script setup lang="ts">
import { computed, onMounted } from 'vue'

import type { DownloadResourceIntent, DownloadTask, ResourceStatus, TaskStatus } from '../api/dto'
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
const selectedLogsLoading = computed(() =>
  queue.selectedTaskId ? Boolean(queue.logsLoadingByTask[queue.selectedTaskId]) : false,
)
const selectedOutputDir = computed(() => outputDir(queue.selectedTask?.output_path ?? null))
const selectedCanRetry = computed(
  () => queue.selectedTask?.status === 'failed' || queue.selectedTask?.status === 'cancelled',
)

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

const statusBadge = (status: TaskStatus): 'ready' | 'downloading' | 'queued' | 'done' | 'error' | 'paused' => {
  if (status === 'completed') {
    return 'done'
  }
  if (status === 'failed' || status === 'cancelled') {
    return 'error'
  }
  if (status === 'paused') {
    return 'paused'
  }
  if (status === 'waiting') {
    return 'queued'
  }
  if (status === 'downloading' || status === 'parsing' || status === 'muxing') {
    return 'downloading'
  }
  return 'ready'
}

const resourceStatusBadge = (status: ResourceStatus): 'ready' | 'downloading' | 'queued' | 'done' | 'error' | 'paused' => {
  if (status === 'completed') {
    return 'done'
  }
  if (status === 'failed' || status === 'cancelled') {
    return 'error'
  }
  if (status === 'paused') {
    return 'paused'
  }
  if (status === 'pending') {
    return 'queued'
  }
  return 'downloading'
}

const resourceStatusText = (status: ResourceStatus): string => {
  const labels: Record<ResourceStatus, string> = {
    pending: '等待',
    downloading: '下载中',
    completed: '已完成',
    failed: '失败',
    paused: '已暂停',
    cancelled: '已取消',
  }
  return labels[status]
}

const resourceIntentText = (intent: DownloadResourceIntent): string => {
  const labels: Record<DownloadResourceIntent, string> = {
    video: '视频',
    audio: '音频',
    cover: '封面',
    subtitle: '字幕',
    danmaku: '弹幕',
    nfo: 'NFO',
  }
  return labels[intent]
}

const outputDir = (path: string | null): string => {
  if (!path) {
    return '--'
  }

  const separatorIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  return separatorIndex >= 0 ? path.slice(0, separatorIndex) : '.'
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
        <UiStatusBadge v-if="queue.selectedTask" :status="statusBadge(queue.selectedTask.status)">
          {{ stageText(queue.selectedTask.status) }}
        </UiStatusBadge>
      </div>

      <template v-if="queue.selectedTask">
        <div class="detail-actions">
          <UiButton
            v-if="selectedCanRetry"
            variant="primary"
            @click="queue.retry(queue.selectedTask.id)"
          >
            重试
          </UiButton>
          <UiButton variant="secondary" @click="queue.openDir(queue.selectedTask.id)">打开文件夹</UiButton>
          <UiButton variant="secondary" @click="queue.openFile(queue.selectedTask.id)">打开文件</UiButton>
        </div>

        <div class="detail-block">
          <strong>{{ queue.selectedTask.title }}</strong>
          <div class="detail-field">
            <span>资源</span>
            <p>{{ resourceMeta(queue.selectedTask) }}</p>
          </div>
          <div class="detail-field">
            <span>保存目录</span>
            <p>{{ selectedOutputDir }}</p>
          </div>
          <div class="detail-field">
            <span>输出文件</span>
            <p>{{ queue.selectedTask.output_path }}</p>
          </div>
        </div>

        <div class="detail-section">
          <div class="section-title">
            <h3>资源</h3>
          </div>
          <div class="resource-list">
            <div
              v-for="resource in queue.selectedTask.resources"
              :key="resource.id"
              class="resource-row"
              :class="{ failed: resource.status === 'failed' || resource.status === 'cancelled' }"
            >
              <span>{{ resourceIntentText(resource.intent) }}</span>
              <UiStatusBadge :status="resourceStatusBadge(resource.status)">
                {{ resourceStatusText(resource.status) }}
              </UiStatusBadge>
            </div>
          </div>
        </div>

        <div class="detail-section logs-section">
          <div class="section-title">
            <h3>日志</h3>
            <UiButton
              variant="ghost"
              :disabled="selectedLogsLoading"
              @click="queue.loadLogs(queue.selectedTask.id)"
            >
              刷新
            </UiButton>
          </div>
          <div class="log-list">
            <div v-for="log in selectedLogs" :key="`${log.created_at}-${log.message}`" class="log-row">
              <span :class="`level-${log.level}`">{{ log.level }}</span>
              <p>{{ log.message }}</p>
            </div>
            <div v-if="selectedLogsLoading && !selectedLogs.length" class="empty-state compact">日志加载中</div>
            <div v-else-if="!selectedLogs.length" class="empty-state compact">暂无日志</div>
          </div>
        </div>
      </template>
      <div v-else class="empty-state">未选择任务</div>
    </aside>
  </section>
</template>

<style scoped>
.transfer-page {
  grid-template-columns: minmax(0, 1fr) minmax(280px, 320px);
}

.transfer-main,
.transfer-detail {
  min-width: 0;
  min-height: 0;
}

.transfer-main {
  overflow: hidden;
}

.transfer-detail {
  overflow: auto;
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

.detail-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-8);
}

.detail-block strong {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail-field {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.detail-field span,
.section-title h3 {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.detail-field p {
  margin: 0;
  min-width: 0;
  overflow-wrap: anywhere;
  color: var(--color-text);
  font-size: var(--font-12);
  line-height: 1.5;
}

.detail-section {
  min-height: 0;
  display: grid;
  gap: var(--space-8);
}

.logs-section {
  min-height: 0;
}

.section-title {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-8);
}

.section-title h3 {
  margin: 0;
}

.resource-list,
.log-list {
  min-height: 0;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: var(--space-8);
}

.resource-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-8);
  padding: var(--space-8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-panel);
}

.resource-row.failed {
  border-color: rgb(201 42 42 / 26%);
  background: #fffafa;
}

.resource-row > span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-12);
  font-weight: 700;
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

.log-row .level-error {
  color: var(--color-danger);
}

.log-row .level-warning {
  color: var(--color-warning);
}

.log-row .level-info {
  color: var(--color-accent-strong);
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

@media (max-width: 1040px) {
  .transfer-page {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr) minmax(180px, 32%);
  }
}
</style>
