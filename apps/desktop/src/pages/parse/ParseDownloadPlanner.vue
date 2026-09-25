<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref, watch } from 'vue'

import type { SettingsSnapshot, NamingPreset, DownloadMediaMode, DuplicateTaskMatch, DuplicateTaskPolicy, VideoCodecPreference, DocumentTreeDirectory } from '../../api/dto'
import { mobilePickExportDirectory } from '../../api/tauri'
import { useParseStore } from '../../stores/parse'
import { validateNamingTemplate, cloneMediaPreferences, useSettingsStore } from '../../stores/settings'
import type { InlineNotice } from '../../stores/feedback'
import { statusBadge, statusLabel } from '../../stores/transferView'
import { archiveModeOptions, duplicateNamingOptions, missingQualityOptions, videoQualityOptions, audioQualityOptions, codecOptions } from '../settings/settingsCatalog'
import NamingTemplatePicker from '../../ui/NamingTemplatePicker.vue'
import MediaPreferenceEditor from '../settings/MediaPreferenceEditor.vue'
import UiTabs from '../../ui/Tabs.vue'
import { isAndroidPlatform, isMobilePlatform } from '../../utils/platform'
import UiCheckbox from '../../ui/Checkbox.vue'
import UiButton from '../../ui/Button.vue'
import UiDialog from '../../ui/Dialog.vue'
import UiDisclosure from '../../ui/Disclosure.vue'
import UiIconButton from '../../ui/IconButton.vue'
import UiInlineNotice from '../../ui/InlineNotice.vue'
import UiSelect from '../../ui/Select.vue'
import UiStatusBadge from '../../ui/StatusBadge.vue'
import UiTextField from '../../ui/TextField.vue'
import { scheduledLocalError, toDateTimeLocalValue, toScheduledIso } from '../../utils/schedule'
import { speedLimitMibError, toBytesPerSecond } from '../../utils/speedLimit'

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
const archiveMode = ref<'fast' | 'complete_archive' | 'custom'>('fast')
const outputExtension = ref<'mp4' | 'mkv'>('mp4')
const mediaMode = ref<DownloadMediaMode>('audio_video')
const videoQuality = ref('best')
const mediaPreferences = ref(cloneMediaPreferences())
const audioQuality = ref('best')
const videoCodec = ref<VideoCodecPreference>('auto')
const scheduledLocal = ref('')
const taskSpeedLimitMib = ref('')
const activeTab = ref('general')
const defaultsRevision = ref(0)
const tabs = [{ label: '常规', value: 'general' }, { label: '画质与音频', value: 'media' }, { label: '附加内容', value: 'assets' }, { label: '调度', value: 'schedule' }]
const namingTemplate = ref('')
const namingPresets = ref<NamingPreset[]>([])
const duplicateNamingStrategy = ref<SettingsSnapshot['duplicate_naming_strategy']>('skip_existing')
const missingQualityPolicy = ref<SettingsSnapshot['missing_quality_policy']>('lower')
const retainRawStreams = ref(false)
const embedCover = ref(false)
const embedSubtitles = ref(false)
const archiveAssets = ref({ ...settings.saved.archive_assets })
const namingError = computed(() => validateNamingTemplate(namingTemplate.value))
const scheduleMin = ref('')
const scheduleValidationNow = ref(Date.now())
const sizeEstimate = ref<{ estimated_bytes: number; estimated_parts: number; unknown_streams: number } | null>(null)
const sizeEstimateLoading = ref(false)
let sizeEstimateRevision = 0

const mediaModeOptions = [
  { label: '音视频', value: 'audio_video' },
  { label: '仅视频', value: 'video_only' },
  { label: '仅音频', value: 'audio_only' },
]
const androidPlatform = isAndroidPlatform()
const outputExtensionOptions = [{ label: 'MP4', value: 'mp4' }, { label: 'MKV', value: 'mkv' }]
const activeSource = computed(() => parse.activeSource)
const selectedSourceIds = computed(() => parse.isBatch
  ? parse.selectedSourceIds
  : activeSource.value && parse.activeSelection.length > 0 ? [activeSource.value.source.id] : [])
const selectedCount = computed(() => parse.isBatch ? parse.selectedBatchEntryIds.length : parse.activeSelection.length)
const selectionUnit = computed(() => parse.isBatch ? '个视频' : '个分集')
const selectionTitle = computed(() => parse.isBatch ? '批量链接' : activeSource.value?.source.title ?? '')
const activeLoading = computed(() => selectedSourceIds.value.some((sourceId) => parse.loadingBySource[sourceId]))
const includesVideo = computed(() => mediaMode.value !== 'audio_only')
const includesAudio = computed(() => mediaMode.value !== 'video_only')
const scheduleError = computed(() => scheduledLocalError(scheduledLocal.value, scheduleValidationNow.value))
const taskSpeedLimitError = computed(() => speedLimitMibError(taskSpeedLimitMib.value))
const embeddingFormatError = computed(() =>
  outputExtension.value !== 'mkv' && (embedCover.value || embedSubtitles.value)
    ? '嵌入封面和字幕仅支持 MKV，请改用 MKV 或关闭本次嵌入。'
    : null,
)
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
  try {
    const estimate = await parse.estimateDownloadSizeForSources(selectedSourceIds.value, {
      mediaMode: mediaMode.value,
      quality: videoQuality.value,
      audioQuality: audioQuality.value,
      codec: videoCodec.value,
      mediaPreferences: {
        ...mediaPreferences.value,
        video: mediaPreferences.value.video.map((rule) => ({ ...rule })),
        audio: [...mediaPreferences.value.audio],
      },
      missingQualityPolicy: missingQualityPolicy.value,
    })
    if (revision === sizeEstimateRevision) sizeEstimate.value = estimate
  } catch {
    if (revision === sizeEstimateRevision) sizeEstimate.value = null
  } finally {
    if (revision === sizeEstimateRevision) sizeEstimateLoading.value = false
  }
}
watch(
  [downloadDialogOpen, mediaMode, videoQuality, audioQuality, videoCodec, mediaPreferences, missingQualityPolicy],
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
  archiveMode.value = defaults.archive_mode
  outputExtension.value = defaults.output_extension
  mediaMode.value = 'audio_video'
  mediaPreferences.value = cloneMediaPreferences(defaults.media_preferences)
  videoQuality.value = mediaPreferences.value.video.length ? 'best' : defaults.quality
  audioQuality.value = mediaPreferences.value.audio.length ? 'best' : defaults.audio_quality
  videoCodec.value = mediaPreferences.value.video.length ? 'auto' : defaults.codec
  namingTemplate.value = defaults.naming_template
  namingPresets.value = defaults.naming_presets.map((preset) => ({ ...preset }))
  duplicateNamingStrategy.value = defaults.duplicate_naming_strategy
  missingQualityPolicy.value = defaults.missing_quality_policy
  archiveAssets.value = { ...defaults.archive_assets }
  retainRawStreams.value = defaults.retain_raw_streams
  embedCover.value = androidPlatform ? false : defaults.embed_cover
  embedSubtitles.value = androidPlatform ? false : defaults.embed_subtitles
  scheduledLocal.value = ''
  taskSpeedLimitMib.value = ''
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
  if (namingError.value || scheduleError.value || taskSpeedLimitError.value || embeddingFormatError.value) {
    downloadNotice.value = {
      message: namingError.value ?? scheduleError.value ?? taskSpeedLimitError.value ?? embeddingFormatError.value ?? '请检查下载设置',
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
    archiveMode: archiveMode.value,
    outputExtension: outputExtension.value,
    namingTemplate: namingTemplate.value,
    duplicateNamingStrategy: duplicateNamingStrategy.value,
    archiveAssets: { ...archiveAssets.value },
    retainRawStreams: retainRawStreams.value,
    embedCover: embedCover.value,
    embedSubtitles: embedSubtitles.value,
    missingQualityPolicy: missingQualityPolicy.value,
    mediaMode: mediaMode.value,
    quality: videoQuality.value,
    audioQuality: audioQuality.value,
    codec: videoCodec.value,
    mediaPreferences: {
      ...mediaPreferences.value,
      video: videoQuality.value === 'best' && videoCodec.value === 'auto' ? mediaPreferences.value.video : [],
      audio: audioQuality.value === 'best' ? mediaPreferences.value.audio : [],
    },
    duplicatePolicy,
    scheduledAt: scheduledLocal.value ? toScheduledIso(scheduledLocal.value) : undefined,
    speedLimitBytesPerSecond: toBytesPerSecond(taskSpeedLimitMib.value),
  })
  if (result?.failures.length) {
    const firstFailure = result.failures[0]
    downloadNotice.value = {
      message:
        result.failures.length === 1
          ? firstFailure.message
          : `${result.failures.length} 个来源创建任务失败：${firstFailure.message}`,
      tone: result.created.length > 0 ? 'warning' : 'danger',
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
      tone: 'warning',
    }
  }
}

</script>

<template>
  <UiDialog v-model="downloadDialogOpen" title="下载设置" :description="`${selectionTitle} · ${selectedCount} ${selectionUnit}`" fixed-height>
    <UiTabs v-model="activeTab" :tabs="tabs" />
    <div :key="activeTab" class="download-options-scroll">
      <UiInlineNotice v-if="downloadNotice" :tone="downloadNotice.tone">{{ downloadNotice.message }}</UiInlineNotice>
      <div class="grid content-start gap-4">
        <div v-show="activeTab === 'general'" class="grid gap-4">
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
          <div class="grid grid-cols-2 gap-3 max-[600px]:grid-cols-1">
            <UiSelect v-model="mediaMode" label="下载内容" :options="mediaModeOptions" />
            <UiSelect v-model="outputExtension" label="封装格式" :options="outputExtensionOptions" />
          </div>
          <NamingTemplatePicker :key="defaultsRevision" v-model="namingTemplate" :presets="namingPresets" :extension="outputExtension" />
          <UiSelect v-model="duplicateNamingStrategy" label="重名处理" :options="duplicateNamingOptions" helper="目标文件已存在时使用此策略；传输记录中的重复任务会另外提示。" />
        </div>
        <div v-show="activeTab === 'media'" class="grid gap-4">
          <div class="grid grid-cols-2 gap-3 max-[600px]:grid-cols-1">
            <UiSelect v-if="includesVideo" v-model="videoQuality" label="视频清晰度" :options="videoQualityOptions" :helper="mediaPreferences.video.length && videoQuality === 'best' && videoCodec === 'auto' ? '按本次优先顺序选择' : undefined" />
            <UiSelect v-if="includesAudio" v-model="audioQuality" label="音频质量" :options="audioQualityOptions" />
            <UiSelect v-if="includesVideo" v-model="videoCodec" label="视频编码偏好" :options="codecOptions" />
            <UiSelect v-model="missingQualityPolicy" label="指定质量不可用时" :options="missingQualityOptions" />
          </div>
          <UiDisclosure title="本次优先顺序" description="最优画质 + 自动编码时使用视频排序；最佳可用音频时使用音频排序。" variant="panel">
            <MediaPreferenceEditor v-model="mediaPreferences" />
          </UiDisclosure>
        </div>
        <div v-show="activeTab === 'assets'" class="grid gap-4">
          <UiSelect v-model="archiveMode" label="下载范围" :options="archiveModeOptions" />
          <div v-if="archiveMode === 'custom'" class="grid grid-cols-2 gap-3 max-[600px]:grid-cols-1">
            <UiCheckbox v-model="archiveAssets.cover" label="保存封面" />
            <UiCheckbox v-model="archiveAssets.subtitles" label="保存字幕" />
            <UiCheckbox v-model="archiveAssets.danmaku" label="保存弹幕" />
            <UiCheckbox v-model="archiveAssets.nfo" label="生成 NFO" />
          </div>
          <UiCheckbox v-model="retainRawStreams" label="保留原始视频/音频轨道" />
          <div class="grid grid-cols-2 gap-3 max-[600px]:grid-cols-1">
            <UiCheckbox v-model="embedCover" label="嵌入封面（仅 MKV）" :disabled="androidPlatform" />
            <UiCheckbox v-model="embedSubtitles" label="嵌入字幕（仅 MKV）" :disabled="androidPlatform" />
          </div>
          <UiInlineNotice v-if="androidPlatform" tone="info">
            Android 内置 FFmpeg 支持 MP4 与 MKV；封面和字幕可保存为独立文件，暂不嵌入成品。
          </UiInlineNotice>
        </div>
        <div v-show="activeTab === 'schedule'" class="grid grid-cols-2 gap-3 max-[600px]:grid-cols-1">
          <UiTextField v-model="scheduledLocal" type="datetime-local" label="开始时间（可选）" :min="scheduleMin" :error="scheduleError" helper="留空时立即加入下载队列" />
          <UiTextField v-model="taskSpeedLimitMib" label="单任务限速（MiB/s）" placeholder="留空时不单独限速" :error="taskSpeedLimitError ?? undefined" helper="留空时仅受全局限速影响" />
        </div>
      </div>
      <UiInlineNotice v-if="embeddingFormatError || namingError || scheduleError || taskSpeedLimitError" tone="danger">{{ embeddingFormatError ?? namingError ?? scheduleError ?? taskSpeedLimitError }}</UiInlineNotice>
    </div>
    <template #footer>
      <div class="planner-dialog-footer">
        <UiButton size="compact" variant="secondary" :disabled="activeLoading" @click="downloadDialogOpen = false"
          >取消</UiButton
        >
        <UiButton class="planner-restore" size="compact" variant="ghost" :disabled="activeLoading" @click="restoreDefaults"
          ><span class="restore-wide">恢复默认偏好</span><span class="restore-short">恢复</span></UiButton
        >
        <span class="planner-estimate" title="按所选媒体轨道的码率与时长估算，不含封面、字幕等附加文件。">{{ sizeEstimateLabel }}</span>
        <UiButton
          size="compact"
          :disabled="activeLoading || selectedSourceIds.length === 0 || Boolean(namingError || scheduleError || taskSpeedLimitError || embeddingFormatError)"
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
  width: 8px;
}

.download-options-scroll::-webkit-scrollbar-thumb {
  border: 2px solid transparent;
  border-radius: 999px;
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
  gap: 6px;
}

.planner-estimate {
  min-width: 0;
  overflow: hidden;
  color: var(--color-muted);
  font-size: 11px;
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
