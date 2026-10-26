<script setup lang="ts">
import { commandDiagnostic } from '../../utils/commandDiagnostic'
import ErrorDetails from '../../ui/ErrorDetails.vue'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref, watch } from 'vue'

import type { SettingsSnapshot, NamingPreset, DuplicateTaskMatch, DuplicateTaskPolicy, DocumentTreeDirectory } from '../../api/dto'
import { mobilePickExportDirectory } from '../../api/tauri'
import { useParseStore } from '../../stores/parse'
import { validateNamingTemplate, useSettingsStore } from '../../stores/settings'
import type { InlineNotice } from '../../stores/feedback'
import { statusBadge, statusLabel } from '../../stores/transferView'
import { duplicateNamingOptions } from '../settings/settingsCatalog'
import NamingTemplatePicker from '../../ui/NamingTemplatePicker.vue'
import UiTabs from '../../ui/Tabs.vue'
import { isAndroidPlatform, isMobilePlatform } from '../../platform/environment'
import UiCheckbox from '../../ui/Checkbox.vue'
import UiButton from '../../ui/Button.vue'
import UiDialog from '../../ui/Dialog.vue'
import UiIconButton from '../../ui/IconButton.vue'
import UiInlineNotice from '../../ui/InlineNotice.vue'
import UiSelect from '../../ui/Select.vue'
import UiStatusBadge from '../../ui/StatusBadge.vue'
import UiTextField from '../../ui/TextField.vue'
import { scheduledLocalError, toDateTimeLocalValue, toScheduledIso } from '../../utils/schedule'
import { speedLimitMbError, toBytesPerSecond } from '../../utils/speedLimit'
import { workflowError, workflowExtension, workflowOutputs } from '../../utils/downloadWorkflow'

const parse = useParseStore()
const settings = useSettingsStore()
const downloadDialogOpen = ref(false)
const duplicateDialogOpen = ref(false)
const duplicateMatches = ref<DuplicateTaskMatch[]>([])
const duplicatePendingSourceIds = ref<string[]>([])
const downloadNotice = ref<InlineNotice | null>(null)
const downloadDir = ref('')
const documentTreeOutput = ref<DocumentTreeDirectory | null>(null)
const rememberDocumentTreeOutput = ref(false)
const downloadPresetId = ref('')
const selectedPreset = computed(() => settings.saved.download_presets.find((preset) => preset.id === downloadPresetId.value))
const effectiveOutputExtension = computed(() => selectedPreset.value ? workflowExtension(selectedPreset.value.workflow) : 'mp4')
const outputPreview = computed(() => selectedPreset.value ? workflowOutputs(selectedPreset.value.workflow) : [])
const scheduledLocal = ref('')
const taskSpeedLimitMb = ref('')
const activeTab = ref('general')
const defaultsRevision = ref(0)
const tabs = [{ label: '下载', value: 'general' }, { label: '调度', value: 'schedule' }]
const namingTemplate = ref('')
const namingPresets = ref<NamingPreset[]>([])
const duplicateNamingStrategy = ref<SettingsSnapshot['duplicate_naming_strategy']>('skip_existing')
const namingError = computed(() => validateNamingTemplate(namingTemplate.value))
const scheduleMin = ref('')
const scheduleValidationNow = ref(Date.now())
const sizeEstimate = ref<{ estimated_bytes: number; estimated_parts: number; unknown_streams: number } | null>(null)
const sizeEstimateLoading = ref(false)
const sizeEstimateError = ref('')
let sizeEstimateRevision = 0

const androidPlatform = isAndroidPlatform()
const activeSource = computed(() => parse.activeSource)
const selectedSourceIds = computed(() => parse.isBatch
  ? parse.selectedSourceIds
  : activeSource.value && parse.activeSelection.length > 0 ? [activeSource.value.source.id] : [])
const selectedCount = computed(() => parse.isBatch ? parse.selectedBatchEntryIds.length : parse.activeSelection.length)
const selectionUnit = computed(() => parse.isBatch ? '个视频' : '个分集')
const selectionTitle = computed(() => parse.isBatch ? '批量链接' : activeSource.value?.source.title ?? '')
const activeLoading = computed(() => selectedSourceIds.value.some((sourceId) => parse.loadingBySource[sourceId]))
watch(downloadPresetId, (id) => {
  if (downloadDialogOpen.value && id && id !== settings.saved.selected_download_preset) void settings.saveAppPreferences({ selected_download_preset: id })
})
const scheduleError = computed(() => scheduledLocalError(scheduledLocal.value, scheduleValidationNow.value))
const taskSpeedLimitError = computed(() => speedLimitMbError(taskSpeedLimitMb.value))
const presetError = computed(() => {
  const flow = selectedPreset.value?.workflow
  if (!flow) return '请选择已保存的下载预设'
  if (androidPlatform && ((flow.cover.enabled && flow.cover.embed) || (flow.subtitles.enabled && flow.subtitles.embed))) return 'Android 暂不支持此预设中的嵌入，请到设置中改为保存独立文件'
  return workflowError(flow)
})
const duplicatePreview = computed(() => duplicateMatches.value.slice(0, 6))
const duplicateRemaining = computed(() => Math.max(duplicateMatches.value.length - duplicatePreview.value.length, 0))
const formatEstimatedBytes = (bytes: number) => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let value = Math.max(0, bytes)
  let unitIndex = 0
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }
  const digits = unitIndex === 0 || value >= 100 ? 0 : value >= 10 ? 1 : 2
  return `${value.toFixed(digits)} ${units[unitIndex]}`
}
const sizeEstimateLabel = computed(() => {
  if (sizeEstimateLoading.value && !sizeEstimate.value) return '正在估算下载大小…'
  if (!sizeEstimate.value) return '预计大小暂不可用'
  if (sizeEstimate.value.estimated_bytes === 0 && sizeEstimate.value.unknown_streams > 0) {
    return '预计大小暂不可用'
  }
  const unknown = sizeEstimate.value.unknown_streams > 0 ? ' + 部分未知' : ''
  return `预计下载 ${formatEstimatedBytes(sizeEstimate.value.estimated_bytes)}${unknown}`
})
const refreshSizeEstimate = async () => {
  if (!downloadDialogOpen.value || selectedSourceIds.value.length === 0) return
  const revision = ++sizeEstimateRevision
  sizeEstimateLoading.value = true
  sizeEstimateError.value = ''
  try {
    const estimate = await parse.estimateDownloadSizeForSources(selectedSourceIds.value, {
      downloadPresetId: downloadPresetId.value,
    })
    if (revision === sizeEstimateRevision) sizeEstimate.value = estimate
  } catch (error) {
    if (revision === sizeEstimateRevision) {
      sizeEstimate.value = null
      sizeEstimateError.value = commandDiagnostic(error, `操作: 估算下载大小\n来源: ${selectionTitle.value}`)
    }
  } finally {
    if (revision === sizeEstimateRevision) sizeEstimateLoading.value = false
  }
}
watch(
  [downloadDialogOpen, downloadPresetId],
  () => {
    if (downloadDialogOpen.value) void refreshSizeEstimate()
  },
  { deep: true },
)
const restoreDefaults = () => {
  defaultsRevision.value += 1
  const defaults = settings.saved
  downloadDir.value = defaults.download_dir ?? ''
  documentTreeOutput.value = defaults.document_tree_output ? { ...defaults.document_tree_output } : null
  rememberDocumentTreeOutput.value = false
  downloadPresetId.value = defaults.selected_download_preset || defaults.download_presets[0]?.id || ''
  namingTemplate.value = defaults.naming_template
  namingPresets.value = defaults.naming_presets.map((preset) => ({ ...preset }))
  duplicateNamingStrategy.value = defaults.duplicate_naming_strategy
  scheduledLocal.value = ''
  taskSpeedLimitMb.value = ''
}

const openDialog = async () => {

  if (selectedSourceIds.value.length === 0 || selectedCount.value === 0) {
    parse.setNotice('请选择要下载的内容', 'warning')
    return
  }

  await settings.ensureLoaded()
  restoreDefaults()
  activeTab.value = 'general'
  scheduleValidationNow.value = Date.now()
  scheduleMin.value = toDateTimeLocalValue(new Date(scheduleValidationNow.value + 60_000))
  duplicateMatches.value = []
  duplicatePendingSourceIds.value = []
  downloadNotice.value = null
  sizeEstimate.value = null
  sizeEstimateError.value = ''
  downloadDialogOpen.value = true
}

defineExpose({ openDialog })

const updateDownloadDir = (value: string) => {
  downloadDir.value = value
}

const createTasks = async (duplicatePolicy: DuplicateTaskPolicy = 'ask') => {
  downloadNotice.value = null
  const sourceIds = duplicatePolicy === 'ask' ? selectedSourceIds.value : duplicatePendingSourceIds.value
  if (sourceIds.length === 0) return
  scheduleValidationNow.value = Date.now()
  if (namingError.value || scheduleError.value || taskSpeedLimitError.value || presetError.value) {
    downloadNotice.value = {
      message: namingError.value ?? scheduleError.value ?? taskSpeedLimitError.value ?? presetError.value ?? '请检查下载设置',
      tone: 'warning',
    }
    return
  }
  if (isMobilePlatform() && !documentTreeOutput.value) {
    downloadNotice.value = {
      message: '请选择 Android 导出目录。',
      tone: 'warning',
    }
    return
  }
  downloadDialogOpen.value = false
  parse.setNotice('正在加入传输队列…', 'info')
  if (isMobilePlatform() && rememberDocumentTreeOutput.value && documentTreeOutput.value) {
    void settings.saveDefaultDocumentTreeOutput({ ...documentTreeOutput.value })
  }
  const result = await parse.createTasksForSources(sourceIds, {
    downloadDir: isMobilePlatform() ? null : downloadDir.value.trim() || 'downloads',
    documentTreeOutput: isMobilePlatform() ? documentTreeOutput.value : null,
    downloadPresetId: downloadPresetId.value,
    namingTemplate: namingTemplate.value,
    duplicateNamingStrategy: duplicateNamingStrategy.value,
    duplicatePolicy,
    scheduledAt: scheduledLocal.value ? toScheduledIso(scheduledLocal.value) : undefined,
    speedLimitBytesPerSecond: toBytesPerSecond(taskSpeedLimitMb.value),
  })
  if (result?.failures.length) {
    const firstFailure = result.failures[0]
    downloadNotice.value = {
      message:
        result.failures.length === 1
          ? firstFailure.message
          : `${result.failures.length} 个来源创建任务失败：${firstFailure.message}`,
      tone: result.created.length > 0 ? 'warning' : 'danger',
      detail: result.failures.map((failure) => failure.detail ?? failure.message).join('\n\n'),
    }
    downloadDialogOpen.value = true
    return
  }
  if (result?.requires_confirmation) {
    duplicateMatches.value = result.duplicates
    duplicatePendingSourceIds.value = result.pendingSourceIds
    downloadDialogOpen.value = false
    duplicateDialogOpen.value = true
    return
  }
  if (!result) {
    downloadDialogOpen.value = true
    return
  }
  duplicateDialogOpen.value = false
  duplicateMatches.value = []
  duplicatePendingSourceIds.value = []
}

const chooseDownloadDir = async () => {
  if (isMobilePlatform()) return
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择保存目录',
      defaultPath: downloadDir.value || settings.saved.download_dir || undefined,
    })
    if (typeof selected === 'string') {
      downloadDir.value = selected
    }
  } catch (error) {
    downloadNotice.value = {
      message: error instanceof Error ? error.message : String(error),
      detail: commandDiagnostic(error, '操作: 选择下载目录'),
      tone: 'warning',
    }
  }
}

const chooseDocumentTreeOutput = async () => {
  try {
    documentTreeOutput.value = await mobilePickExportDirectory()
    if (!settings.saved.document_tree_output) rememberDocumentTreeOutput.value = true
  } catch (error) {
    downloadNotice.value = {
      message: error instanceof Error ? error.message : String(error),
      detail: commandDiagnostic(error, '操作: 选择下载目录'),
      tone: 'warning',
    }
  }
}

</script>

<template>
  <UiDialog v-model="downloadDialogOpen" title="下载设置" :description="`${selectionTitle} · ${selectedCount} ${selectionUnit}`" fixed-height>
    <UiTabs v-model="activeTab" :tabs="tabs" />
    <div :key="activeTab" class="download-options-scroll">
      <UiInlineNotice v-if="downloadNotice" :tone="downloadNotice.tone" :detail="downloadNotice.detail">{{ downloadNotice.message }}</UiInlineNotice>
      <ErrorDetails v-if="sizeEstimateError && !downloadNotice" :detail="sizeEstimateError" />
      <div class="grid content-start gap-4">
        <div v-show="activeTab === 'general'" class="grid gap-4">
          <UiSelect v-model="downloadPresetId" label="下载预设" :options="settings.saved.download_presets.map((preset) => ({ label: preset.name, value: preset.id }))" :disabled="activeLoading" helper="在设置 → 下载预设中新增或修改，自动记住上次选择" />
          <div class="grid gap-2 rounded-lg border border-(--color-border) p-3" aria-label="本次输出预览">
            <strong class="text-sm">将生成</strong>
            <ul class="m-0 grid list-none gap-1 p-0 text-sm"><li v-for="item in outputPreview" :key="item">{{ item }}</li></ul>
          </div>
          <div v-if="isMobilePlatform()" class="directory-row">
            <UiTextField :model-value="documentTreeOutput?.display_name ?? ''" label="导出目录" placeholder="请选择目录" disabled />
            <UiIconButton
              icon="folder-open"
              :label="documentTreeOutput ? '更换导出目录' : '选择导出目录'"
              :disabled="activeLoading"
              @click="chooseDocumentTreeOutput"
            />
          </div>
          <div v-if="isMobilePlatform() && documentTreeOutput" class="grid gap-1">
            <UiCheckbox v-model="rememberDocumentTreeOutput" label="设为默认导出目录" />
            <p v-if="settings.saved.document_tree_output && !rememberDocumentTreeOutput" class="m-0 text-xs text-(--color-muted)">
              已使用设置中的默认目录，本次无需再次选择；更换目录后可勾选保存为新的默认目录。
            </p>
          </div>
          <div v-if="!isMobilePlatform()" class="directory-row">
            <UiTextField :model-value="downloadDir" label="保存目录" placeholder="留空时使用 downloads" @update:model-value="updateDownloadDir" />
            <UiButton variant="secondary" :disabled="activeLoading" @click="chooseDownloadDir">选择</UiButton>
          </div>
          <NamingTemplatePicker :key="defaultsRevision" v-model="namingTemplate" :presets="namingPresets" :extension="effectiveOutputExtension" />
          <UiSelect v-model="duplicateNamingStrategy" label="重名处理" :options="duplicateNamingOptions" helper="目标文件已存在时使用此策略；传输记录中的重复任务会另外提示。" />
        </div>
        <div v-show="activeTab === 'schedule'" class="grid grid-cols-2 gap-3 max-[37.5rem]:grid-cols-1">
          <UiTextField v-model="scheduledLocal" type="datetime-local" label="开始时间（可选）" :min="scheduleMin" :error="scheduleError" helper="留空时立即加入下载队列" />
          <UiTextField v-model="taskSpeedLimitMb" label="单任务限速（MB/s）" placeholder="留空时不单独限速" :error="taskSpeedLimitError ?? undefined" helper="留空时仅受全局限速影响" />
        </div>
      </div>
      <UiInlineNotice v-if="presetError || namingError || scheduleError || taskSpeedLimitError" tone="danger">{{ presetError ?? namingError ?? scheduleError ?? taskSpeedLimitError }}</UiInlineNotice>
    </div>
    <template #footer>
      <div class="planner-dialog-footer">
        <UiButton size="compact" variant="secondary" :disabled="activeLoading" @click="downloadDialogOpen = false"
          >取消</UiButton
        >
        <UiButton class="planner-restore" size="compact" variant="ghost" :disabled="activeLoading" @click="restoreDefaults"
          ><span class="restore-wide">恢复默认偏好</span><span class="restore-short">恢复</span></UiButton
        >
        <span class="planner-estimate" title="按媒体轨道码率与时长估算，附加文件大小可能未知。">{{ sizeEstimateLabel }}</span>
        <UiButton
          size="compact"
          :disabled="activeLoading || sizeEstimateLoading || selectedSourceIds.length === 0 || Boolean(namingError || scheduleError || taskSpeedLimitError || presetError)"
          @click="createTasks()"
          >开始下载</UiButton
        >
      </div>
    </template>
  </UiDialog>

  <UiDialog v-model="duplicateDialogOpen" title="发现重复任务">
    <section class="flex items-start gap-3">
      <span class="grid size-9 shrink-0 place-items-center rounded-lg border border-(--color-border) bg-(--color-accent-soft) text-(--color-accent-strong)" aria-hidden="true">
        <UIcon class="size-4" name="i-tabler-copy" />
      </span>
      <div>
        <strong>{{ duplicateMatches.length }} 个分集已在传输记录中</strong>
        <p class="mt-1 mb-0 text-(--color-muted)">可以跳过这些分集，或创建使用独立文件名的新任务。</p>
      </div>
    </section>
    <ul class="m-0 max-h-60 list-none overflow-auto rounded-lg border border-(--color-border) p-0" aria-label="重复任务">
      <li v-for="match in duplicatePreview" :key="match.proposed_task_id" class="flex min-h-11 items-center justify-between gap-3 border-b border-(--color-border) px-2.5 py-2 last:border-b-0">
        <span class="truncate">{{ match.title }}</span>
        <UiStatusBadge :status="statusBadge(match.existing_status)">{{ statusLabel(match.existing_status) }}</UiStatusBadge>
      </li>
    </ul>
    <p v-if="duplicateRemaining > 0" class="mt-1 mb-0 text-(--color-muted)">另有 {{ duplicateRemaining }} 项未展开</p>
    <template #footer>
      <UiButton variant="ghost" :disabled="activeLoading" @click="duplicateDialogOpen = false">取消</UiButton>
      <UiButton variant="secondary" :disabled="activeLoading" @click="createTasks('skip')">跳过重复项</UiButton>
      <UiButton :disabled="activeLoading" @click="createTasks('create')">仍然创建</UiButton>
    </template>
  </UiDialog>
</template>


<style scoped>
.download-options-scroll {
  min-height: 0;
  flex: 1 1 0%;
  display: grid;
  align-content: start;
  gap: var(--space-16);
  overflow: auto;
  overscroll-behavior: contain;
  scrollbar-gutter: stable;
  scrollbar-width: thin;
  scrollbar-color: color-mix(in oklab, var(--color-muted) 35%, transparent) transparent;
}

.download-options-scroll::-webkit-scrollbar {
  width: 0.5rem;
}

.download-options-scroll::-webkit-scrollbar-thumb {
  border: 0.125rem solid transparent;
  border-radius: 62.4375rem;
  background: color-mix(in oklab, var(--color-muted) 35%, transparent);
  background-clip: padding-box;
}

.download-options-scroll::-webkit-scrollbar-thumb:hover {
  background-color: color-mix(in oklab, var(--color-muted) 60%, transparent);
}

.directory-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
}

.planner-dialog-footer {
  width: 100%;
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.375rem;
}

.planner-estimate {
  min-width: 0;
  overflow: hidden;
  color: var(--color-muted);
  font-size: 0.6875rem;
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.restore-short {
  display: none;
}

@media (width <= 600px) {
  .restore-wide {
    display: none;
  }

  .restore-short {
    display: inline;
  }
}
</style>
