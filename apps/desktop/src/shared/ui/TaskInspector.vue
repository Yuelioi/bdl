<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import type { DownloadResource, DownloadTask, QueueLogEntry, ResourceStatus } from '../api/dto'
import { useDiagnosticExport } from '../features/diagnostics/useDiagnosticExport'
import { useUiStore } from '../stores/ui'
import {
  createTaskDiagnosticView,
  redactLogMessage,
  redactDiagnosticMessage,
  resourceIntentText,
  statusLabel,
} from '../stores/transferView'
import UiButton from './Button.vue'
import UiDefinitionList from './DefinitionList.vue'
import UiEmptyState from './EmptyState.vue'
import UiInlineNotice from './InlineNotice.vue'
import UiStatusBadge from './StatusBadge.vue'
import UiTabs from './Tabs.vue'
import { isMobilePlatform } from '../platform/environment'
import { captureDiagnosticEnvironment } from '../utils/diagnosticEnvironment'
import { writeClipboardText } from '../utils/clipboard'
import { classifyFailure, failureGuidance, latestFailure } from '../utils/failureDiagnostic'
import { audioCodecOptions, audioQualityOptions, codecOptions, videoQualityOptions } from '../features/settings/settingsCatalog'

const { task, progress, logs, logsLoading = false } = defineProps<{
  task: DownloadTask | null
  progress: number
  logs: QueueLogEntry[]
  logsLoading?: boolean
}>()

const emit = defineEmits<{
  refreshLogs: []
  recover: [taskId: string, action: 'retry' | 'refresh_retry']
  openSettings: []
}>()

const ui = useUiStore()
const selectedTab = ref('overview')
const copied = ref(false)
const {
  busy: exportingDiagnostics, folderReady, notice: exportNotice, noticeTone,
  run: exportDiagnostics, reset: resetExport,
} = useDiagnosticExport(() => task?.id)
const mobile = isMobilePlatform()
const transferCapabilities = { canOpenOutput: !mobile }

const tabs = computed(() => [
  { label: '概览', value: 'overview' },
  { label: '轨道', value: 'tracks', count: task?.resources.length ?? 0 },
  { label: '日志', value: 'raw_logs', count: logs.length },
])
const diagnostic = computed(() => (task ? createTaskDiagnosticView(task, logs, transferCapabilities) : null))
const recoveryAction = computed(() =>
  diagnostic.value?.recommendedAction === 'refresh_retry' ? 'refresh_retry' : 'retry',
)
const canRecover = computed(() => !logs.length || ['retry', 'refresh_retry'].some(action => action === diagnostic.value?.recommendedAction))
const guidance = computed(() => {
  const failure = latestFailure(logs) ?? ''
  const rule = classifyFailure(failure)
  const stage = failure.startsWith('[stage=refresh_urls ') ? 'parse' : 'download'
  return rule && failureGuidance(rule, stage, mobile)
})
const followGuidance = () => {
  if (!guidance.value) return
  const { section } = guidance.value
  if (section) {
    emit('openSettings')
    ui.openSettingsSection(section, section === 'settings-download')
  } else ui.openLoginDialog()
}
const recover = () => {
  if (task) emit('recover', task.id, recoveryAction.value)
}
const failedTrackCount = computed(
  () => task?.resources.filter((resource) => resource.status === 'failed' || resource.status === 'cancelled').length ?? 0,
)
const completedTrackCount = computed(
  () => task?.resources.filter((resource) => resource.status === 'completed').length ?? 0,
)
const selectionLabel = (options: readonly { value: string; label: string }[], value?: string | null) =>
  options.find((option) => option.value === value)?.label ?? (value && value !== 'unknown' ? value : '未记录')
const overviewFacts = computed(() => {
  const facts = [{ label: '输出文件', value: task?.output_path ?? '--' }]
  if (!task?.media_selection) return facts
  const selection = task.media_selection
  if (task.resources.some((resource) => resource.intent === 'video')) {
    facts.push(
      { label: '视频清晰度', value: selectionLabel(videoQualityOptions, selection.video_quality) },
      { label: '视频编码', value: selectionLabel(codecOptions, selection.video_codec) },
    )
  }
  if (task.resources.some((resource) => resource.intent === 'audio')) {
    const bandwidth = selection.audio_bandwidth
    facts.push(
      { label: '音频质量', value: selectionLabel(audioQualityOptions, selection.audio_quality) },
      {
        label: '源音频码率',
        value: bandwidth && bandwidth > 0
          ? `${(bandwidth / 1000).toLocaleString('zh-CN', { maximumFractionDigits: 1 })} kbps`
          : '未记录',
      },
      { label: '源音频编码', value: selectionLabel(audioCodecOptions, selection.audio_codec) },
    )
  }
  return facts
})
const redactedLogs = computed(() =>
  logs.map((log) => ({
    ...log,
    message: redactDiagnosticMessage(log.message),
  })),
)
const mediaSelectionText = computed(() => {
  const selection = task?.media_selection
  return `流选择: video_quality=${selection?.video_quality ?? '--'} audio_quality=${selection?.audio_quality ?? '--'} codec=${selection?.video_codec ?? '--'} container=${selection?.container ?? '--'}`
})
const diagnosticText = computed(() => {
  if (!task || !diagnostic.value) {
    return ''
  }

  const lines = [
    `任务: ${task.title}`,
    `任务 ID: ${task.id}`,
    `状态: ${statusLabel(task.status)}`,
    `诊断: ${diagnostic.value.summary}`,
    `原因: ${diagnostic.value.detail}`,
    `建议: ${diagnostic.value.recommendedActionLabel || '查看原始日志'}`,
    `影响: ${diagnostic.value.impact}`,
    `输出: ${task.output_path}`,
    mediaSelectionText.value,
    '',
    `原始日志（最近 ${Math.min(redactedLogs.value.length, 20)} 条；完整保留日志可导出任务诊断）:`,
    ...redactedLogs.value.slice(0, 20).map((log) => `- [${log.level}] ${log.created_at} ${log.message}`),
  ]

  return lines.join('\n')
})

watch(
  () => task?.id,
  () => {
    copied.value = false
    resetExport()
    selectedTab.value = 'overview'
  },
  { immediate: true },
)

const setTab = (value: string) => {
  selectedTab.value = value
}

const copyDiagnostics = async () => {
  const text = diagnosticText.value
  if (!text) {
    return
  }

  try {
    const environment = await captureDiagnosticEnvironment()
    await writeClipboardText(`${environment}\n\n${text}`)
    copied.value = true
    window.setTimeout(() => {
      copied.value = false
    }, 1600)
  } catch {
    copied.value = false
    ui.pushToast('复制失败，请重试', 'danger')
  }
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

</script>

<template>
  <div class="task-inspector">
    <template v-if="task && diagnostic">
      <UiTabs :model-value="selectedTab" :tabs="tabs" @update:model-value="setTab" />

      <section v-if="selectedTab === 'overview'" class="tab-panel overview-panel">
        <div v-if="diagnostic.tone === 'danger' || diagnostic.tone === 'warning'" class="diagnosis-block feedback-tone" :class="`tone-${diagnostic.tone}`">
          <h3>{{ diagnostic.summary }}</h3>
          <p>{{ task.status === 'failed' && !logsLoading && !logs.length && !diagnostic.recommendedAction ? '暂无可用日志。可以先重试任务。' : diagnostic.detail }}{{ task.status === 'failed' ? guidance?.tip : '' }}</p>
          <div v-if="task.status === 'failed' && (guidance || !mobile)" class="diagnosis-actions mt-3">
            <UiButton v-if="guidance" :class="{ 'mobile-recovery': mobile }" :size="mobile ? 'default' : 'compact'" @click="followGuidance">{{ guidance.label }}</UiButton>
            <UiButton v-if="canRecover && !mobile" size="compact" :variant="guidance ? 'secondary' : 'primary'" :disabled="logsLoading" @click="recover">
              <UIcon name="i-tabler-refresh" class="size-4" aria-hidden="true" />
              {{ recoveryAction === 'refresh_retry' ? '刷新链接并重试' : '重试任务' }}
            </UiButton>
            <UiButton size="compact" variant="ghost" @click="setTab('raw_logs')">查看日志</UiButton>
          </div>
        </div>
        <div v-if="task.status !== 'failed' || task.resources.length" class="overview-strip">
          <div>
            <strong>{{ progress }}%</strong>
            <span>{{ statusLabel(task.status) }}</span>
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

      <section v-else class="tab-panel raw-panel">
        <div class="log-toolbar">
          <span>最近 {{ redactedLogs.length }} 条日志</span>
          <div class="diagnosis-actions">
            <UiButton variant="secondary" :disabled="logsLoading || !redactedLogs.length" @click="copyDiagnostics">{{ copied ? '已复制' : '复制诊断信息' }}</UiButton>
            <UiButton variant="ghost" :disabled="exportingDiagnostics" @click="exportDiagnostics">
              {{ folderReady ? '打开诊断文件夹' : exportingDiagnostics ? '导出中' : '导出任务诊断' }}
            </UiButton>
            <UiButton variant="ghost" :disabled="logsLoading" @click="emit('refreshLogs')">刷新</UiButton>
          </div>
        </div>
        <div class="log-list">
          <UiInlineNotice v-if="exportNotice" :tone="noticeTone">{{ exportNotice }}</UiInlineNotice>
          <div v-for="log in redactedLogs" :key="`${log.created_at}-${log.message}`" class="log-row">
            <span :class="`level-${log.level}`">{{ log.level }} · {{ new Date(log.created_at).toLocaleString('zh-CN') }}</span>
            <p>{{ redactLogMessage(log.message) }}</p>
          </div>
          <UiEmptyState v-if="logsLoading && !redactedLogs.length" class="inspector-empty" title="日志加载中" layout="stacked" compact />
          <UiEmptyState v-else-if="!redactedLogs.length" class="inspector-empty" title="暂无日志" layout="stacked" compact embedded />
        </div>
      </section>
    </template>
    <UiEmptyState v-else title="未选择任务" layout="stacked" compact />
  </div>
</template>

<style scoped src="./TaskInspector.css"></style>
