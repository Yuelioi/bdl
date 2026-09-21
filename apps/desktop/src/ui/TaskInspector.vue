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
import UiDefinitionList from './DefinitionList.vue'
import UiEmptyState from './EmptyState.vue'
import UiInlineNotice from './InlineNotice.vue'
import UiSectionToolbar from './SectionToolbar.vue'
import UiStatusBadge from './StatusBadge.vue'
import UiTabs from './Tabs.vue'
import ExternalLinkButton from './ExternalLinkButton.vue'
import { bilibiliTaskUrl } from '../utils/bilibiliLinks'
import { isMobilePlatform } from '../utils/platform'

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
const transferCapabilities = { canOpenOutput: !isMobilePlatform() }

const tabs = computed(() => [
  { label: '诊断', value: 'diagnosis' },
  { label: '概览', value: 'overview' },
  { label: '轨道', value: 'tracks', count: task?.resources.length ?? 0 },
  { label: '事件', value: 'events', count: timeline.value.length },
  { label: '原始日志', value: 'raw_logs', count: logs.length },
])
const diagnostic = computed(() => (task ? createTaskDiagnosticView(task, logs, transferCapabilities) : null))
const timeline = computed(() => (task ? createTaskTimeline(task, logs) : []))
const failedTrackCount = computed(
  () => task?.resources.filter((resource) => resource.status === 'failed' || resource.status === 'cancelled').length ?? 0,
)
const completedTrackCount = computed(
  () => task?.resources.filter((resource) => resource.status === 'completed').length ?? 0,
)
const outputDirectory = computed(() => outputDir(task?.output_path ?? null))
const diagnosticFacts = computed(() => [
  { label: '影响范围', value: diagnostic.value?.impact ?? '--' },
  { label: '输出位置', value: outputDirectory.value },
])
const overviewFacts = computed(() => [
  { label: '保存目录', value: outputDirectory.value },
  { label: '输出文件', value: task?.output_path ?? '--' },
  { label: '任务 ID', value: task?.id ?? '--' },
])
const taskPageUrl = computed(() => (task ? bilibiliTaskUrl(task) : null))
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
        <div class="diagnosis-block feedback-tone" :class="`tone-${diagnostic.tone}`">
          <span>{{ statusLabel(task.status) }}</span>
          <h3>{{ diagnostic.summary }}</h3>
          <p>{{ diagnostic.detail }}</p>
        </div>
        <UiDefinitionList :items="diagnosticFacts" />
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
        <UiDefinitionList :items="overviewFacts" />
        <ExternalLinkButton v-if="taskPageUrl" :href="taskPageUrl" label="在 Bilibili 打开来源页面">
          在 Bilibili 查看来源
        </ExternalLinkButton>
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
        <UiSectionToolbar :label="`${timeline.length} 条事件`">
          <template #actions>
            <UiButton variant="ghost" :disabled="logsLoading" @click="emit('refreshLogs')">刷新</UiButton>
          </template>
        </UiSectionToolbar>
        <div class="event-list">
          <div v-for="event in timeline" :key="event.id" class="event-row feedback-tone" :class="`tone-${event.tone}`">
            <time>{{ formatEventTime(event.time) }}</time>
            <div>
              <strong>{{ event.title }}</strong>
              <p>{{ event.detail }}</p>
            </div>
          </div>
        </div>
      </section>

      <section v-else class="tab-panel raw-panel">
        <UiSectionToolbar :label="`${redactedLogs.length} 条原始日志`">
          <template #actions>
            <UiButton variant="ghost" @click="copyDiagnostics">{{ copied ? '已复制' : '复制诊断信息' }}</UiButton>
            <UiButton variant="ghost" :disabled="exportingDiagnostics" @click="exportDiagnostics">
              {{ exportingDiagnostics ? '导出中' : '导出诊断' }}
            </UiButton>
            <UiButton variant="ghost" :disabled="logsLoading" @click="emit('refreshLogs')">刷新</UiButton>
          </template>
        </UiSectionToolbar>
        <div class="log-list">
          <UiInlineNotice v-if="exportNotice" tone="success">{{ exportNotice }}</UiInlineNotice>
          <div v-for="log in redactedLogs" :key="`${log.created_at}-${log.message}`" class="log-row">
            <span :class="`level-${log.level}`">{{ log.level }}</span>
            <p>{{ log.message }}</p>
          </div>
          <UiEmptyState v-if="logsLoading && !redactedLogs.length" class="inspector-empty" title="日志加载中" layout="stacked" compact />
          <UiEmptyState v-else-if="!redactedLogs.length" class="inspector-empty" title="暂无日志" layout="stacked" compact />
        </div>
      </section>
    </template>
    <UiEmptyState v-else title="未选择任务" layout="stacked" compact />
  </div>
</template>

<style scoped src="./TaskInspector.css"></style>
