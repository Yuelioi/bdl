<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import type { DownloadResource, DownloadTask, QueueLogEntry, ResourceStatus } from '../api/dto'
import { diagnosticsExport } from '../api/tauri'
import { useUiStore } from '../stores/ui'
import {
  createTaskDiagnosticView,
  createTaskTimeline,
  redactLogMessage,
  resourceIntentText,
  statusLabel,
} from '../stores/transferView'
import UiButton from './Button.vue'
import UiInlineNotice from './InlineNotice.vue'
import UiStatusBadge from './StatusBadge.vue'
import UiTabs from './Tabs.vue'

const { task, progress, logs, logsLoading = false } = defineProps<{
  task: DownloadTask | null
  progress: number
  logs: QueueLogEntry[]
  logsLoading?: boolean
}>()

const emit = defineEmits<{
  refreshLogs: []
}>()

const ui = useUiStore()
const selectedTab = ref('overview')
const tabTouched = ref(false)
const copied = ref(false)
const exportingDiagnostics = ref(false)
const exportNotice = ref('')

const tabs = computed(() => [
  { label: '诊断', value: 'diagnosis' },
  { label: '概览', value: 'overview' },
  { label: '轨道', value: 'tracks', count: task?.resources.length ?? 0 },
  { label: '事件', value: 'events', count: timeline.value.length },
  { label: '原始日志', value: 'raw_logs', count: logs.length },
])
const diagnostic = computed(() => (task ? createTaskDiagnosticView(task, logs) : null))
const timeline = computed(() => (task ? createTaskTimeline(task, logs) : []))
const failedTrackCount = computed(
  () => task?.resources.filter((resource) => resource.status === 'failed' || resource.status === 'cancelled').length ?? 0,
)
const completedTrackCount = computed(
  () => task?.resources.filter((resource) => resource.status === 'completed').length ?? 0,
)
const outputDirectory = computed(() => outputDir(task?.output_path ?? null))
const redactedLogs = computed(() =>
  logs.map((log) => ({
    ...log,
    message: redactLogMessage(log.message),
  })),
)
const diagnosticText = computed(() => {
  if (!task || !diagnostic.value) {
    return ''
  }

  const lines = [
    `任务: ${task.title}`,
    `状态: ${statusLabel(task.status)}`,
    `诊断: ${diagnostic.value.summary}`,
    `建议: ${diagnostic.value.recommendedActionLabel || '查看原始日志'}`,
    `影响: ${diagnostic.value.impact}`,
    `输出: ${task.output_path}`,
    '',
    '事件:',
    ...timeline.value.map((event) => `- ${formatEventTime(event.time)} ${event.title}: ${event.detail}`),
    '',
    '原始日志:',
    ...redactedLogs.value.map((log) => `- [${log.level}] ${log.created_at} ${log.message}`),
  ]

  return lines.join('\n')
})

const defaultTab = (): string => (task?.status === 'failed' || task?.status === 'cancelled' ? 'diagnosis' : 'overview')

watch(
  () => task?.id,
  () => {
    tabTouched.value = false
    selectedTab.value = defaultTab()
  },
  { immediate: true },
)

watch(
  () => task?.status,
  () => {
    if (!tabTouched.value) {
      selectedTab.value = defaultTab()
    }
  },
)

const setTab = (value: string) => {
  tabTouched.value = true
  selectedTab.value = value
}

const copyDiagnostics = async () => {
  if (!diagnosticText.value) {
    return
  }

  try {
    await navigator.clipboard.writeText(diagnosticText.value)
    copied.value = true
    window.setTimeout(() => {
      copied.value = false
    }, 1600)
  } catch {
    copied.value = false
  }
}

const exportDiagnostics = async () => {
  exportingDiagnostics.value = true
  try {
    const result = await diagnosticsExport()
    exportNotice.value = `诊断已导出：${result.path}`
    window.setTimeout(() => {
      exportNotice.value = ''
    }, 3000)
  } catch (error) {
    ui.pushToast(errorMessage(error), 'danger')
  } finally {
    exportingDiagnostics.value = false
  }
}

const formatEventTime = (value: string): string => {
  if (!value) {
    return '--:--'
  }

  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return value.slice(11, 16) || '--:--'
  }

  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
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

const trackTitle = (resource: DownloadResource): string => resourceIntentText(resource.intent)

const outputDir = (path: string | null): string => {
  if (!path) {
    return '--'
  }

  const separatorIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  return separatorIndex >= 0 ? path.slice(0, separatorIndex) : '.'
}

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
</script>

<template>
  <div class="task-inspector">
    <template v-if="task && diagnostic">
      <UiTabs :model-value="selectedTab" :tabs="tabs" @update:model-value="setTab" />

      <section v-if="selectedTab === 'diagnosis'" class="tab-panel diagnosis-panel">
        <div class="diagnosis-block" :class="`tone-${diagnostic.tone}`">
          <span>{{ statusLabel(task.status) }}</span>
          <h3>{{ diagnostic.summary }}</h3>
          <p>{{ diagnostic.detail }}</p>
        </div>
        <dl class="diagnosis-facts">
          <div>
            <dt>影响范围</dt>
            <dd>{{ diagnostic.impact }}</dd>
          </div>
          <div>
            <dt>输出位置</dt>
            <dd>{{ outputDirectory }}</dd>
          </div>
        </dl>
        <div class="diagnosis-actions">
          <UiButton variant="secondary" @click="copyDiagnostics">
            {{ copied ? '已复制' : '复制诊断信息' }}
          </UiButton>
          <UiButton variant="secondary" :disabled="exportingDiagnostics" @click="exportDiagnostics">
            {{ exportingDiagnostics ? '导出中' : '导出诊断' }}
          </UiButton>
        </div>
        <UiInlineNotice v-if="exportNotice" tone="success">{{ exportNotice }}</UiInlineNotice>
      </section>

      <section v-else-if="selectedTab === 'overview'" class="tab-panel overview-panel">
        <div class="overview-strip">
          <div>
            <strong>{{ progress }}%</strong>
            <span>进度</span>
          </div>
          <div>
            <strong>{{ completedTrackCount }}</strong>
            <span>完成轨道</span>
          </div>
          <div :class="{ danger: failedTrackCount > 0 }">
            <strong>{{ failedTrackCount }}</strong>
            <span>失败轨道</span>
          </div>
        </div>
        <dl class="overview-list">
          <div>
            <dt>保存目录</dt>
            <dd>{{ outputDirectory }}</dd>
          </div>
          <div>
            <dt>输出文件</dt>
            <dd>{{ task.output_path }}</dd>
          </div>
          <div>
            <dt>任务 ID</dt>
            <dd>{{ task.id }}</dd>
          </div>
        </dl>
      </section>

      <section v-else-if="selectedTab === 'tracks'" class="tab-panel tracks-panel">
        <div class="track-list">
          <div
            v-for="resource in task.resources"
            :key="resource.id"
            class="track-row"
            :class="{ failed: resource.status === 'failed' || resource.status === 'cancelled' }"
          >
            <div class="track-copy">
              <strong>{{ trackTitle(resource) }}</strong>
              <p>{{ resource.target_path }}</p>
            </div>
            <UiStatusBadge :status="resourceStatusBadge(resource.status)">
              {{ resourceStatusText(resource.status) }}
            </UiStatusBadge>
          </div>
        </div>
      </section>

      <section v-else-if="selectedTab === 'events'" class="tab-panel events-panel">
        <div class="events-toolbar">
          <span>{{ timeline.length }} 条事件</span>
          <UiButton variant="ghost" :disabled="logsLoading" @click="emit('refreshLogs')">刷新</UiButton>
        </div>
        <div class="event-list">
          <div v-for="event in timeline" :key="event.id" class="event-row" :class="`tone-${event.tone}`">
            <time>{{ formatEventTime(event.time) }}</time>
            <div>
              <strong>{{ event.title }}</strong>
              <p>{{ event.detail }}</p>
            </div>
          </div>
        </div>
      </section>

      <section v-else class="tab-panel raw-panel">
        <div class="logs-toolbar">
          <span>{{ redactedLogs.length }} 条原始日志</span>
          <div>
            <UiButton variant="ghost" @click="copyDiagnostics">{{ copied ? '已复制' : '复制诊断信息' }}</UiButton>
            <UiButton variant="ghost" :disabled="exportingDiagnostics" @click="exportDiagnostics">
              {{ exportingDiagnostics ? '导出中' : '导出诊断' }}
            </UiButton>
            <UiButton variant="ghost" :disabled="logsLoading" @click="emit('refreshLogs')">刷新</UiButton>
          </div>
        </div>
        <div class="log-list">
          <UiInlineNotice v-if="exportNotice" tone="success">{{ exportNotice }}</UiInlineNotice>
          <div v-for="log in redactedLogs" :key="`${log.created_at}-${log.message}`" class="log-row">
            <span :class="`level-${log.level}`">{{ log.level }}</span>
            <p>{{ log.message }}</p>
          </div>
          <div v-if="logsLoading && !redactedLogs.length" class="empty-state compact">日志加载中</div>
          <div v-else-if="!redactedLogs.length" class="empty-state compact">暂无日志</div>
        </div>
      </section>
    </template>
    <div v-else class="empty-state">未选择任务</div>
  </div>
</template>

<style scoped>
.task-inspector {
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: var(--space-12);
  overflow: hidden;
}

.tab-panel {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.diagnosis-panel,
.overview-panel,
.events-panel,
.raw-panel {
  display: grid;
  align-content: start;
  gap: var(--space-12);
}

.diagnosis-block {
  min-width: 0;
  display: grid;
  gap: var(--space-6, 6px);
  padding: var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

.diagnosis-block span {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.diagnosis-block h3,
.diagnosis-block p {
  margin: 0;
}

.diagnosis-block h3 {
  color: var(--color-text);
  font-size: var(--font-14);
  line-height: 1.45;
}

.diagnosis-block p {
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.55;
}

.diagnosis-block.tone-danger {
  border-color: rgb(201 42 42 / 24%);
  background: #fffafa;
}

.diagnosis-block.tone-danger span {
  color: var(--color-danger);
}

.diagnosis-block.tone-success {
  border-color: rgb(26 127 55 / 20%);
  background: #f8fff9;
}

.diagnosis-facts,
.overview-list {
  display: grid;
  gap: var(--space-12);
}

.diagnosis-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-8);
}

.diagnosis-facts div,
.overview-list div {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.diagnosis-facts dt,
.overview-list dt,
.events-toolbar span,
.logs-toolbar span {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.diagnosis-facts dd,
.overview-list dd {
  margin: 0;
  min-width: 0;
  overflow-wrap: anywhere;
  color: var(--color-text);
  font-size: var(--font-12);
  line-height: 1.55;
}

.overview-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  padding: var(--space-8) 0;
  border-top: 1px solid var(--color-border);
  border-bottom: 1px solid var(--color-border);
}

.overview-strip div {
  min-width: 0;
  display: grid;
  gap: 2px;
  padding: 0 var(--space-8);
}

.overview-strip div + div {
  border-left: 1px solid var(--color-border);
}

.overview-strip strong {
  font-size: var(--font-16);
  line-height: 1.15;
}

.overview-strip span {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.overview-strip .danger strong,
.overview-strip .danger span {
  color: var(--color-danger);
}

.tracks-panel,
.event-list,
.log-list {
  min-height: 0;
}

.track-list,
.event-list,
.log-list {
  min-width: 0;
  min-height: 0;
  display: grid;
  align-content: start;
  gap: var(--space-8);
  overflow-x: hidden;
  overflow-y: auto;
  padding-right: 2px;
}

.track-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: start;
  gap: var(--space-8);
  padding: var(--space-12) var(--space-8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-panel);
}

.track-row.failed {
  border-color: rgb(201 42 42 / 26%);
  background: #fffafa;
}

.track-copy {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.track-copy strong {
  overflow-wrap: anywhere;
  font-size: var(--font-12);
  font-weight: 700;
}

.track-copy p {
  margin: 0;
  overflow-wrap: anywhere;
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.45;
}

.events-panel,
.raw-panel {
  grid-template-rows: auto minmax(0, 1fr);
}

.events-toolbar,
.logs-toolbar {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-8);
}

.logs-toolbar > div {
  display: inline-flex;
  align-items: center;
  gap: var(--space-4);
}

.event-row {
  min-width: 0;
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr);
  gap: var(--space-8);
  padding: var(--space-8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-panel);
}

.event-row.tone-danger {
  border-color: color-mix(in oklab, var(--color-danger) 34%, var(--color-border));
  background: var(--color-danger-soft);
}

.event-row.tone-warning {
  border-color: color-mix(in oklab, var(--color-warning) 38%, var(--color-border));
  background: var(--color-warning-soft);
}

.event-row time {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.event-row strong,
.event-row p {
  margin: 0;
}

.event-row strong {
  color: var(--color-text);
  font-size: var(--font-12);
}

.event-row p {
  margin-top: var(--space-4);
  overflow-wrap: anywhere;
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.5;
}

.log-row {
  min-width: 0;
  max-width: 100%;
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
  min-width: 0;
  max-width: 100%;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
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
