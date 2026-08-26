<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref } from 'vue'

import type { DownloadMediaMode, DuplicateTaskMatch, DuplicateTaskPolicy, VideoCodecPreference } from '../../api/dto'
import { useParseStore } from '../../stores/parse'
import { useSettingsStore } from '../../stores/settings'
import { useUiStore } from '../../stores/ui'
import type { InlineNotice } from '../../stores/feedback'
import { statusBadge, statusLabel } from '../../stores/transferView'
import UiButton from '../../ui/Button.vue'
import UiDialog from '../../ui/Dialog.vue'
import UiDisclosure from '../../ui/Disclosure.vue'
import UiInlineNotice from '../../ui/InlineNotice.vue'
import UiSelect from '../../ui/Select.vue'
import UiStatusBadge from '../../ui/StatusBadge.vue'
import UiTextField from '../../ui/TextField.vue'
import { scheduledLocalError, toDateTimeLocalValue, toScheduledIso } from '../../utils/schedule'
import { formatSpeedLimit, speedLimitMibError, toBytesPerSecond } from '../../utils/speedLimit'

const parse = useParseStore()
const settings = useSettingsStore()
const ui = useUiStore()
const downloadDialogOpen = ref(false)
const duplicateDialogOpen = ref(false)
const duplicateMatches = ref<DuplicateTaskMatch[]>([])
const duplicatePendingSourceIds = ref<string[]>([])
const downloadNotice = ref<InlineNotice | null>(null)
const downloadDir = ref('')
const archiveMode = ref<'fast' | 'complete_archive' | 'custom'>('fast')
const outputExtension = ref<'mp4' | 'mkv'>('mp4')
const mediaMode = ref<DownloadMediaMode>('audio_video')
const videoQuality = ref('best')
const audioQuality = ref('best')
const videoCodec = ref<VideoCodecPreference>('auto')
const scheduledLocal = ref('')
const taskSpeedLimitMib = ref('')
const saveLayout = ref<'title_folder' | 'download_root'>('title_folder')
const scheduleMin = ref('')
const scheduleValidationNow = ref(Date.now())

const mediaModeOptions = [
  { label: '音视频', value: 'audio_video' },
  { label: '仅视频', value: 'video_only' },
  { label: '仅音频', value: 'audio_only' },
]
const videoQualityOptions = [
  { label: '最佳可用', value: 'best' }, { label: '8K / 127', value: '127' },
  { label: '4K / 120', value: '120' }, { label: '1080P60 / 116', value: '116' },
  { label: '1080P+ / 112', value: '112' }, { label: '1080P / 80', value: '80' },
  { label: '720P / 64', value: '64' }, { label: '480P / 32', value: '32' },
  { label: '360P / 16', value: '16' },
]
const audioQualityOptions = [
  { label: '最佳可用', value: 'best' }, { label: '高音质 / 30280', value: '30280' },
  { label: '中音质 / 30232', value: '30232' }, { label: '低音质 / 30216', value: '30216' },
]
const codecOptions = [
  { label: '自动', value: 'auto' }, { label: 'AVC / H.264', value: 'avc' },
  { label: 'HEVC / H.265', value: 'hevc' }, { label: 'AV1', value: 'av1' },
]
const outputExtensionOptions = [{ label: 'MP4', value: 'mp4' }, { label: 'MKV', value: 'mkv' }]
const archiveModeOptions = [
  { label: '仅下载最终媒体', value: 'fast' },
  { label: '下载全部附加内容', value: 'complete_archive' },
  { label: '使用设置中的附加内容', value: 'custom' },
]
const saveLayoutOptions = [
  { label: '按标题建立子文件夹', value: 'title_folder' },
  { label: '直接保存到下载目录', value: 'download_root' },
]
const directDownloadNamingTemplate = '{title} - P{part_index} - {part_title}.{ext}'

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
  outputExtension.value !== 'mkv' && (settings.draft.embed_cover || settings.draft.embed_subtitles)
    ? '嵌入封面和字幕仅支持 MKV，请改用 MKV 或在设置中关闭嵌入。'
    : null,
)
const duplicatePreview = computed(() => duplicateMatches.value.slice(0, 6))
const duplicateRemaining = computed(() => Math.max(duplicateMatches.value.length - duplicatePreview.value.length, 0))
const downloadAdvancedSummary = computed(() => {
  const codec = includesVideo.value ? optionLabel(codecOptions, videoCodec.value) : '无视频编码'
  return `${outputExtension.value.toUpperCase()} · ${codec} · ${optionLabel(saveLayoutOptions, saveLayout.value)} · ${optionLabel(archiveModeOptions, archiveMode.value)}`
})
const downloadSettingsSummary = computed(() => {
  const content = optionLabel(mediaModeOptions, mediaMode.value)
  const video = includesVideo.value ? optionLabel(videoQualityOptions, videoQuality.value) : '不下载视频'
  const audio = includesAudio.value ? optionLabel(audioQualityOptions, audioQuality.value) : '不下载音频'
  const schedule = scheduledLocal.value ? `定时 ${new Date(scheduledLocal.value).toLocaleString('zh-CN')}` : '立即开始'
  const taskLimit = toBytesPerSecond(taskSpeedLimitMib.value)
  return `${content} · ${video} · ${audio} · ${schedule} · ${taskLimit ? formatSpeedLimit(taskLimit) : '不限速'}`
})

const openDialog = async () => {
  if (!settings.environmentReady) {
    ui.openEnvironmentDialog()
    return
  }

  if (selectedSourceIds.value.length === 0 || selectedCount.value === 0) {
    parse.setNotice('请选择要下载的内容', 'warning')
    return
  }

  await settings.ensureLoaded()
  const defaults = settings.changed ? settings.draft : settings.saved
  downloadDir.value = defaults.download_dir ?? ''
  archiveMode.value = defaults.archive_mode
  outputExtension.value = defaults.output_extension
  mediaMode.value = 'audio_video'
  videoQuality.value = defaults.quality
  audioQuality.value = defaults.audio_quality
  videoCodec.value = defaults.codec
  saveLayout.value = 'title_folder'
  scheduledLocal.value = ''
  taskSpeedLimitMib.value = ''
  scheduleValidationNow.value = Date.now()
  scheduleMin.value = toDateTimeLocalValue(new Date(scheduleValidationNow.value + 60_000))
  duplicateMatches.value = []
  duplicatePendingSourceIds.value = []
  downloadNotice.value = null
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
  if (scheduleError.value || taskSpeedLimitError.value || embeddingFormatError.value) {
    downloadNotice.value = {
      message: scheduleError.value ?? taskSpeedLimitError.value ?? embeddingFormatError.value ?? '请检查下载设置',
      tone: 'warning',
    }
    return
  }
  if (!settings.environmentReady) {
    ui.openEnvironmentDialog()
    return
  }
  const result = await parse.createTasksForSources(sourceIds, {
    downloadDir: downloadDir.value,
    archiveMode: archiveMode.value,
    outputExtension: outputExtension.value,
    namingTemplate: saveLayout.value === 'download_root' ? directDownloadNamingTemplate : undefined,
    mediaMode: mediaMode.value,
    quality: videoQuality.value,
    audioQuality: audioQuality.value,
    codec: videoCodec.value,
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
    return
  }
  if (result?.requires_confirmation) {
    duplicateMatches.value = result.duplicates
    duplicatePendingSourceIds.value = result.pendingSourceIds
    downloadDialogOpen.value = false
    duplicateDialogOpen.value = true
    return
  }
  if (!result) return
  downloadDialogOpen.value = false
  duplicateDialogOpen.value = false
  duplicateMatches.value = []
  duplicatePendingSourceIds.value = []
}

const chooseDownloadDir = async () => {
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

function optionLabel(options: Array<{ label: string; value: string }>, value: string): string {
  return options.find((option) => option.value === value)?.label ?? value
}
</script>

<template>
  <UiDialog v-model="downloadDialogOpen" title="下载设置">
    <section class="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] items-center gap-3 rounded-lg border border-(--color-border) bg-(--color-accent-soft) p-3">
      <strong class="text-3xl leading-none text-(--color-accent-strong)">{{ selectedCount }}</strong>
      <div class="grid min-w-0 gap-1">
        <span class="text-xs font-bold text-(--color-muted)">{{ selectionUnit }}将加入传输</span>
        <p class="m-0 truncate text-[13px] font-bold text-(--color-text)">{{ selectionTitle }}</p>
      </div>
    </section>

    <UiInlineNotice v-if="downloadNotice" :tone="downloadNotice.tone">
      {{ downloadNotice.message }}
    </UiInlineNotice>

    <div class="grid gap-3">
      <div class="directory-row">
        <UiTextField :model-value="downloadDir" label="保存目录" placeholder="留空时使用 downloads" @update:model-value="updateDownloadDir" />
        <UiButton variant="secondary" :disabled="activeLoading" @click="chooseDownloadDir">选择</UiButton>
      </div>
      <UiSelect v-model="saveLayout" label="文件保存结构" :options="saveLayoutOptions" />

      <section class="grid gap-2">
        <h3 class="m-0 text-[13px] font-extrabold text-(--color-text)">下载内容</h3>
        <div class="grid grid-cols-2 gap-3 max-[840px]:grid-cols-1">
          <UiSelect v-model="mediaMode" label="下载内容" :options="mediaModeOptions" />
          <UiSelect v-if="includesVideo" v-model="videoQuality" label="视频清晰度" :options="videoQualityOptions" />
          <UiSelect v-if="includesAudio" v-model="audioQuality" label="音频质量" :options="audioQualityOptions" />
        </div>
      </section>

      <section class="grid gap-2">
        <h3 class="m-0 text-[13px] font-extrabold text-(--color-text)">开始方式</h3>
        <div class="grid grid-cols-2 gap-3 max-[840px]:grid-cols-1">
          <UiTextField v-model="scheduledLocal" type="datetime-local" label="开始时间（可选）" :min="scheduleMin" :error="scheduleError" helper="留空时立即加入下载队列" />
          <UiTextField v-model="taskSpeedLimitMib" label="单任务限速（MiB/s）" placeholder="留空时不单独限速" :error="taskSpeedLimitError ?? undefined" helper="留空时仅受全局限速影响" />
        </div>
      </section>

      <UiDisclosure title="更多选项" :description="downloadAdvancedSummary" variant="panel">
        <div class="grid grid-cols-2 gap-3 max-[840px]:grid-cols-1">
          <UiSelect v-if="includesVideo" v-model="videoCodec" label="视频编码偏好" :options="codecOptions" />
          <UiSelect v-model="outputExtension" label="封装格式" :options="outputExtensionOptions" />
          <UiSelect v-model="archiveMode" label="附加内容" :options="archiveModeOptions" />
        </div>
      </UiDisclosure>

      <UiInlineNotice v-if="embeddingFormatError" tone="danger">{{ embeddingFormatError }}</UiInlineNotice>

      <p class="m-0 wrap-anywhere rounded-md border border-(--color-border) bg-(--color-panel) px-3 py-2 text-xs leading-5 text-(--color-muted)">
        {{ downloadSettingsSummary }}
      </p>
    </div>

    <template #footer>
      <UiButton variant="secondary" :disabled="activeLoading" @click="downloadDialogOpen = false">取消</UiButton>
      <UiButton
        :disabled="selectedSourceIds.length === 0 || Boolean(scheduleError) || Boolean(taskSpeedLimitError) || Boolean(embeddingFormatError) || !settings.environmentReady"
        @click="createTasks()"
      >加入传输</UiButton>
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
