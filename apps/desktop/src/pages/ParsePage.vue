<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref, useTemplateRef } from 'vue'

import type {
  DownloadMediaMode,
  DuplicateTaskMatch,
  DuplicateTaskPolicy,
  NormalizedSourceTree,
  SourceKind,
  VideoCodecPreference,
} from '../api/dto'
import UiButton from '../ui/Button.vue'
import UiDialog from '../ui/Dialog.vue'
import UiDisclosure from '../ui/Disclosure.vue'
import UiEmptyState from '../ui/EmptyState.vue'
import UiIconButton from '../ui/IconButton.vue'
import UiInlineNotice from '../ui/InlineNotice.vue'
import UiSelect from '../ui/Select.vue'
import UiStatusBadge from '../ui/StatusBadge.vue'
import UiTextarea from '../ui/Textarea.vue'
import UiTextField from '../ui/TextField.vue'
import UiTree from '../ui/Tree.vue'
import UiEnvironmentHealthPanel from '../ui/EnvironmentHealthPanel.vue'
import { useParseStore } from '../stores/parse'
import { useSettingsStore } from '../stores/settings'
import { useUiStore } from '../stores/ui'
import { statusBadge, statusLabel } from '../stores/transferView'
import { scheduledLocalError, toDateTimeLocalValue, toScheduledIso } from '../utils/schedule'
import { formatSpeedLimit, speedLimitMibError, toBytesPerSecond } from '../utils/speedLimit'
import { displayPartDuration, formatDuration } from '../utils/duration'

interface PageTreeNode {
  id: string
  label: string
  meta?: string
  partIds: string[]
  searchText: string
  sortTitle: string
  durationSeconds: number | null
  sourceOrder: number
  leafPartId?: string
  children?: PageTreeNode[]
}

interface VisiblePartEntry {
  id: string
  label: string
}

type ResultSortMode = 'source' | 'title_asc' | 'duration_desc'

const sourceKindLabels: Record<SourceKind, string> = {
  video: '视频',
  bangumi: '番剧',
  cheese: '课程',
  favorite: '收藏夹',
  collection: '合集',
  series: '系列',
  uploader: 'UP 主',
  unknown: '未知',
}

const parse = useParseStore()
const settings = useSettingsStore()
const ui = useUiStore()
const inputFile = useTemplateRef<HTMLInputElement>('input-file')
const downloadDialogOpen = ref(false)
const duplicateDialogOpen = ref(false)
const duplicateMatches = ref<DuplicateTaskMatch[]>([])
const duplicatePreview = computed(() => duplicateMatches.value.slice(0, 6))
const duplicateRemaining = computed(() => Math.max(duplicateMatches.value.length - duplicatePreview.value.length, 0))
const downloadDir = ref('')
const archiveMode = ref<'fast' | 'complete_archive' | 'custom'>('fast')
const outputExtension = ref<'mp4' | 'mkv'>('mp4')
const mediaMode = ref<DownloadMediaMode>('audio_video')
const videoQuality = ref('best')
const audioQuality = ref('best')
const videoCodec = ref<VideoCodecPreference>('auto')
const scheduledLocal = ref('')
const taskSpeedLimitMib = ref('')
const scheduleMin = ref('')
const scheduleValidationNow = ref(Date.now())
const resultQuery = ref('')
const resultSort = ref<ResultSortMode>('source')
const rangeExpression = ref('')
const rangeError = ref('')

const mediaModeOptions = [
  { label: '音视频', value: 'audio_video' },
  { label: '仅视频', value: 'video_only' },
  { label: '仅音频', value: 'audio_only' },
]
const videoQualityOptions = [
  { label: '最佳可用', value: 'best' },
  { label: '8K / 127', value: '127' },
  { label: '4K / 120', value: '120' },
  { label: '1080P60 / 116', value: '116' },
  { label: '1080P+ / 112', value: '112' },
  { label: '1080P / 80', value: '80' },
  { label: '720P / 64', value: '64' },
  { label: '480P / 32', value: '32' },
  { label: '360P / 16', value: '16' },
]
const audioQualityOptions = [
  { label: '最佳可用', value: 'best' },
  { label: '高音质 / 30280', value: '30280' },
  { label: '中音质 / 30232', value: '30232' },
  { label: '低音质 / 30216', value: '30216' },
]
const codecOptions = [
  { label: '自动', value: 'auto' },
  { label: 'AVC / H.264', value: 'avc' },
  { label: 'HEVC / H.265', value: 'hevc' },
  { label: 'AV1', value: 'av1' },
]
const outputExtensionOptions = [
  { label: 'MP4', value: 'mp4' },
  { label: 'MKV', value: 'mkv' },
]
const archiveModeOptions = [
  { label: '仅下载最终媒体', value: 'fast' },
  { label: '下载全部附加内容', value: 'complete_archive' },
  { label: '使用设置中的附加内容', value: 'custom' },
]
const resultSortOptions = [
  { label: '原始顺序', value: 'source' },
  { label: '标题 A-Z', value: 'title_asc' },
  { label: '时长优先', value: 'duration_desc' },
]

const activeSource = computed(() => parse.activeSource)
const selectedIds = computed(() => parse.activeSelection)
const selectedCount = computed(() => selectedIds.value.length)
const totalPartCount = computed(() => (activeSource.value ? sourcePartCount(activeSource.value) : 0))
const treeNodes = computed(() => {
  if (!activeSource.value) {
    return []
  }

  const filtered = filterTreeNodes(toTreeNodes(activeSource.value), resultQuery.value)
  return numberVisibleParts(sortTreeNodes(filtered, resultSort.value))
})
const visiblePartEntries = computed(() => flattenVisibleParts(treeNodes.value))
const visiblePartCount = computed(() => visiblePartEntries.value.length)
const createLoading = computed(() => Boolean(parse.loadingBySource.__create__))
const activeLoading = computed(() => Boolean(activeSource.value && parse.loadingBySource[activeSource.value.source.id]))
const activeError = computed(() => (activeSource.value ? parse.errorsBySource[activeSource.value.source.id] : null))
const canCreateTasks = computed(() => Boolean(activeSource.value && selectedCount.value > 0 && !activeLoading.value))
const hasResultQuery = computed(() => resultQuery.value.trim().length > 0)
const canSelectResults = computed(() => Boolean(activeSource.value && visiblePartCount.value > 0 && !activeLoading.value))
const canSelectRange = computed(() => Boolean(activeSource.value && rangeExpression.value.trim() && visiblePartCount.value > 0 && !activeLoading.value))
const scheduleError = computed(() => {
  return scheduledLocalError(scheduledLocal.value, scheduleValidationNow.value)
})
const taskSpeedLimitError = computed(() => speedLimitMibError(taskSpeedLimitMib.value))
const createTaskLabel = computed(() => {
  if (activeLoading.value) {
    return '处理中'
  }
  return selectedCount.value > 0 ? `下载已选择 (${selectedCount.value})` : '先选择分集'
})
const activeSourceKindLabel = computed(() => (activeSource.value ? sourceKindLabels[activeSource.value.source.kind] : '--'))
const activeSourceTitle = computed(() => activeSource.value?.source.title ?? '')
const canCloseActiveSource = computed(() => Boolean(activeSource.value && parse.sourceOrder.length > 1))
const sourceMenuLabel = computed(() => (parse.sourceOrder.length > 0 ? `${parse.sourceOrder.length} 个来源` : '无来源'))
const selectAllLabel = computed(() => (hasResultQuery.value ? `全选搜索结果 (${visiblePartCount.value})` : '全选全部'))
const includesVideo = computed(() => mediaMode.value !== 'audio_only')
const includesAudio = computed(() => mediaMode.value !== 'video_only')
const downloadSettingsSummary = computed(() => {
  const dir = downloadDir.value.trim() || 'downloads'
  const content = optionLabel(mediaModeOptions, mediaMode.value)
  const video = includesVideo.value ? optionLabel(videoQualityOptions, videoQuality.value) : '不下载视频'
  const audio = includesAudio.value ? optionLabel(audioQualityOptions, audioQuality.value) : '不下载音频'
  const codec = includesVideo.value ? optionLabel(codecOptions, videoCodec.value) : '无视频编码'
  const archive =
    archiveMode.value === 'complete_archive'
      ? '全部附加内容'
      : archiveMode.value === 'custom'
        ? '使用设置中的附加内容'
        : '仅最终媒体文件'
  const schedule = scheduledLocal.value
    ? `定时 ${new Date(scheduledLocal.value).toLocaleString('zh-CN', { dateStyle: 'short', timeStyle: 'short' })}`
    : '立即开始'
  const taskLimit = toBytesPerSecond(taskSpeedLimitMib.value)
  const speedLimit = taskLimit ? `任务限速 ${formatSpeedLimit(taskLimit)}` : '仅受全局限速影响'
  return `${content} · ${video} · ${audio} · ${codec} · ${outputExtension.value.toUpperCase()} · ${dir} · ${archive} · ${schedule} · ${speedLimit}`
})
const downloadAdvancedSummary = computed(() => {
  const codec = includesVideo.value ? optionLabel(codecOptions, videoCodec.value) : '无视频编码'
  return `${outputExtension.value.toUpperCase()} · ${codec} · ${optionLabel(archiveModeOptions, archiveMode.value)}`
})

const submitInput = () => {
  void parse.createSource()
}

const importTextFile = () => {
  inputFile.value?.click()
}

const handleTextFile = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) {
    return
  }

  parse.appendInput(await file.text())
  target.value = ''
}

const openDownloadSettings = async () => {
  if (!activeSource.value || selectedCount.value === 0) {
    parse.setNotice('请选择要下载的分集', 'warning')
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
  scheduledLocal.value = ''
  taskSpeedLimitMib.value = ''
  scheduleValidationNow.value = Date.now()
  scheduleMin.value = toDateTimeLocalValue(new Date(scheduleValidationNow.value + 60_000))
  downloadDialogOpen.value = true
  await checkDownloadEnvironment()
}

const checkDownloadEnvironment = () => {
  return settings.checkEnvironment({
    downloadDir: downloadDir.value.trim() || null,
    ffmpegPath: settings.saved.ffmpeg_path,
  })
}

const updateDownloadDir = (value: string) => {
  downloadDir.value = value
  settings.invalidateEnvironmentHealth()
}

const createDownloadDirectory = () => {
  return settings.createDownloadDirectory({
    downloadDir: downloadDir.value.trim() || null,
    ffmpegPath: settings.saved.ffmpeg_path,
  })
}

const createTasks = async (duplicatePolicy: DuplicateTaskPolicy = 'ask') => {
  if (activeSource.value) {
    scheduleValidationNow.value = Date.now()
    if (scheduleError.value || taskSpeedLimitError.value) {
      parse.setNotice(scheduleError.value ?? taskSpeedLimitError.value ?? '请检查下载设置', 'warning')
      return
    }
    const health = await checkDownloadEnvironment()
    if (!health?.ready) {
      parse.setNotice('请先修复保存目录或 FFmpeg 环境', 'warning')
      return
    }
    const result = await parse.createTasksForSelection(activeSource.value.source.id, {
      downloadDir: downloadDir.value,
      archiveMode: archiveMode.value,
      outputExtension: outputExtension.value,
      mediaMode: mediaMode.value,
      quality: videoQuality.value,
      audioQuality: audioQuality.value,
      codec: videoCodec.value,
      duplicatePolicy,
      scheduledAt: scheduledLocal.value ? toScheduledIso(scheduledLocal.value) : undefined,
      speedLimitBytesPerSecond: toBytesPerSecond(taskSpeedLimitMib.value),
    })
    if (result?.requires_confirmation) {
      duplicateMatches.value = result.duplicates
      downloadDialogOpen.value = false
      duplicateDialogOpen.value = true
      return
    }
    if (!result) {
      return
    }
    downloadDialogOpen.value = false
    duplicateDialogOpen.value = false
    duplicateMatches.value = []
  }
}

const resolveDuplicates = (policy: Exclude<DuplicateTaskPolicy, 'ask'>) => {
  void createTasks(policy)
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
      await checkDownloadEnvironment()
    }
  } catch (error) {
    parse.setNotice(errorMessage(error), 'warning')
  }
}

const openEnvironmentSettings = () => {
  downloadDialogOpen.value = false
  ui.setTab('settings')
}

const selectAllResults = () => {
  if (activeSource.value) {
    parse.selectPartIds(
      activeSource.value.source.id,
      visiblePartEntries.value.map((entry) => entry.id),
    )
  }
}

const clearSelection = () => {
  if (activeSource.value) {
    parse.clearSelection(activeSource.value.source.id)
  }
}

const refreshSource = () => {
  if (activeSource.value) {
    void parse.refreshSource(activeSource.value.source.id)
  }
}

const runNoticeAction = () => {
  if (parse.notice?.actionLabel === '查看传输') {
    ui.setTab('transfer')
    parse.clearNotice()
  }
}

const closeSource = (sourceId: string) => {
  void parse.closeSource(sourceId)
}

const selectSource = (sourceId: string) => {
  parse.setActiveSource(sourceId)
}

const sourceMenuItems = computed(() => [
  parse.orderedSources.map((tree) => ({
    label: tree.source.title,
    description: `${sourcePartCount(tree)} 项 · 已选 ${sourceSelectedCount(tree.source.id)}`,
    icon: parse.activeSourceId === tree.source.id ? 'i-tabler-check' : 'i-tabler-link',
    onSelect: () => selectSource(tree.source.id),
  })),
])

const toggleNode = (nodeId: string) => {
  if (activeSource.value) {
    parse.toggleNode(activeSource.value.source.id, nodeId)
  }
}

const selectRange = () => {
  if (!activeSource.value) {
    return
  }

  rangeError.value = ''
  try {
    const indexes = parseRangeExpression(rangeExpression.value, visiblePartEntries.value.length)
    const partIds = indexes.map((index) => visiblePartEntries.value[index].id)
    parse.selectPartIds(activeSource.value.source.id, partIds)
    parse.setNotice(`已选中 ${partIds.length} 个结果`, 'success')
  } catch (error) {
    rangeError.value = errorMessage(error)
  }
}

const toTreeNodes = (tree: NormalizedSourceTree): PageTreeNode[] =>
  tree.groups.flatMap((group, groupIndex) => {
    const itemNodes = group.items.map((item, itemIndex) => itemNode(item, tree.source.kind, itemIndex)).flat()

    if (tree.source.kind === 'video' && group.items.length === 1) {
      return videoItemNodes(group.items[0])
    }

    if (tree.groups.length === 1 || sameTitle(group.title, tree.source.title)) {
      return itemNodes
    }

    return [
      {
        id: group.id,
        label: group.title,
        meta: `${group.items.length} 项`,
        partIds: group.items.flatMap((item) => item.parts.map((part) => part.id)),
        searchText: searchableText(group.title, `${group.items.length} 项`),
        sortTitle: group.title,
        durationSeconds: maxDuration(group.items.map((item) => item.duration_seconds)),
        sourceOrder: groupIndex,
        children: itemNodes,
      },
    ]
  })

const itemNode = (
  item: NormalizedSourceTree['groups'][number]['items'][number],
  sourceKind: SourceKind,
  itemIndex: number,
): PageTreeNode[] => {
  if (item.parts.length === 1) {
    const part = item.parts[0]
    return [
      {
        id: item.id,
        label: item.title || part.title,
        meta: item.owner_name ?? partMeta(item, part),
        partIds: [part.id],
        searchText: searchableText(item.title, item.owner_name, part.title, part.bvid, part.cid),
        sortTitle: item.title || part.title,
        durationSeconds: item.duration_seconds,
        sourceOrder: itemIndex,
        leafPartId: part.id,
      },
    ]
  }

  if (sourceKind === 'video') {
    return videoItemNodes(item)
  }

  return [
    {
      id: item.id,
      label: item.title,
      meta: `${item.parts.length} P`,
      partIds: item.parts.map((part) => part.id),
      searchText: searchableText(item.title, item.owner_name, `${item.parts.length} P`),
      sortTitle: item.title,
      durationSeconds: item.duration_seconds,
      sourceOrder: itemIndex,
      children: item.parts.map((part, index) => partNode(item, part, index)),
    },
  ]
}

const videoItemNodes = (item: NormalizedSourceTree['groups'][number]['items'][number]): PageTreeNode[] =>
  item.parts.map((part, index) => partNode(item, part, index))

const partNode = (
  item: NormalizedSourceTree['groups'][number]['items'][number],
  part: NormalizedSourceTree['groups'][number]['items'][number]['parts'][number],
  index: number,
): PageTreeNode => ({
  id: part.id,
  label: part.title || item.title || `P${index + 1}`,
  meta: partMeta(item, part),
  partIds: [part.id],
  searchText: searchableText(item.title, item.owner_name, part.title, part.bvid, part.cid),
  sortTitle: part.title || item.title || `P${index + 1}`,
  durationSeconds: displayPartDuration(part.duration_seconds, item.duration_seconds, item.parts.length),
  sourceOrder: index,
  leafPartId: part.id,
})

const filterTreeNodes = (nodes: PageTreeNode[], query: string): PageTreeNode[] => {
  const normalized = normalizeSearch(query)
  if (!normalized) {
    return nodes.map(cloneTreeNode)
  }

  return nodes
    .map((node) => filterTreeNode(node, normalized))
    .filter((node): node is PageTreeNode => Boolean(node))
}

const filterTreeNode = (node: PageTreeNode, query: string): PageTreeNode | null => {
  const children = node.children
    ?.map((child) => filterTreeNode(child, query))
    .filter((child): child is PageTreeNode => Boolean(child))

  if (node.searchText.includes(query)) {
    return cloneTreeNode(node)
  }

  if (children?.length) {
    return {
      ...node,
      partIds: children.flatMap((child) => child.partIds),
      children,
    }
  }

  return null
}

const sortTreeNodes = (nodes: PageTreeNode[], mode: ResultSortMode): PageTreeNode[] => {
  const sorted = nodes.map((node) => ({
    ...node,
    children: node.children ? sortTreeNodes(node.children, mode) : undefined,
  }))

  if (mode === 'source') {
    return sorted
  }

  return sorted.sort((left, right) => compareTreeNodes(left, right, mode))
}

const compareTreeNodes = (left: PageTreeNode, right: PageTreeNode, mode: ResultSortMode): number => {
  if (mode === 'duration_desc') {
    const byDuration = (right.durationSeconds ?? -1) - (left.durationSeconds ?? -1)
    if (byDuration !== 0) {
      return byDuration
    }
  }

  if (mode === 'title_asc' || mode === 'duration_desc') {
    const byTitle = left.sortTitle.localeCompare(right.sortTitle, 'zh-Hans-CN', { numeric: true, sensitivity: 'base' })
    if (byTitle !== 0) {
      return byTitle
    }
  }

  return left.sourceOrder - right.sourceOrder
}

const numberVisibleParts = (nodes: PageTreeNode[]): PageTreeNode[] => {
  let index = 0
  const visit = (node: PageTreeNode): PageTreeNode => {
    const children = node.children?.map(visit)
    if (!children?.length && node.leafPartId) {
      index += 1
      return {
        ...node,
        label: `${String(index).padStart(2, '0')}  ${node.label}`,
      }
    }

    return { ...node, children }
  }

  return nodes.map(visit)
}

const flattenVisibleParts = (nodes: PageTreeNode[]): VisiblePartEntry[] =>
  nodes.flatMap((node) => {
    if (node.children?.length) {
      return flattenVisibleParts(node.children)
    }

    return node.leafPartId ? [{ id: node.leafPartId, label: node.label }] : []
  })

const cloneTreeNode = (node: PageTreeNode): PageTreeNode => ({
  ...node,
  children: node.children?.map(cloneTreeNode),
})

const searchableText = (...values: Array<string | number | null | undefined>): string =>
  values
    .filter((value) => value !== null && value !== undefined && value !== '')
    .map((value) => normalizeSearch(String(value)))
    .join(' ')

const normalizeSearch = (value: string): string => value.trim().toLocaleLowerCase()

const maxDuration = (values: Array<number | null>): number | null => {
  const durations = values.filter((value): value is number => typeof value === 'number')
  return durations.length ? Math.max(...durations) : null
}

const parseRangeExpression = (value: string, total: number): number[] => {
  const expression = value.trim()
  if (!expression) {
    throw new Error('请输入范围')
  }

  const selected = new Set<number>()
  for (const token of expression.split(/[,，\s]+/).filter(Boolean)) {
    const match = token.match(/^(\d+)(?:-(\d+))?$/)
    if (!match) {
      throw new Error(`范围格式无效：${token}`)
    }

    const start = Number(match[1])
    const end = Number(match[2] ?? match[1])
    const min = Math.min(start, end)
    const max = Math.max(start, end)
    if (min < 1 || max > total) {
      throw new Error(`范围超出当前结果数量：${token}`)
    }

    for (let index = min; index <= max; index += 1) {
      selected.add(index - 1)
    }
  }

  return [...selected].sort((left, right) => left - right)
}

const partMeta = (
  item: NormalizedSourceTree['groups'][number]['items'][number],
  part: NormalizedSourceTree['groups'][number]['items'][number]['parts'][number],
): string => {
  const duration = displayPartDuration(part.duration_seconds, item.duration_seconds, item.parts.length)
  return duration !== null ? formatDuration(duration) : ''
}

const sameTitle = (left: string, right: string): boolean => left.trim() !== '' && left.trim() === right.trim()

const sourcePartCount = (tree: NormalizedSourceTree): number =>
  tree.groups.reduce(
    (groupTotal, group) => groupTotal + group.items.reduce((itemTotal, item) => itemTotal + item.parts.length, 0),
    0,
  )

const sourceSelectedCount = (sourceId: string): number => parse.selectionBySource[sourceId]?.length ?? 0

function optionLabel(options: Array<{ label: string; value: string }>, value: string): string {
  return options.find((option) => option.value === value)?.label ?? value
}

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
</script>

<template>
  <section class="page-grid parse-page">
    <section class="panel parse-input-panel command-panel" :class="{ 'has-results': activeSource }">
      <div class="panel-heading">
        <div class="command-heading">
          <span class="command-index">01</span>
          <div>
            <h2>添加来源</h2>
            <p>粘贴链接或编号，BDL 会识别类型并整理成可选择的分集。</p>
          </div>
        </div>
        <div class="parse-heading-tools">
          <div class="source-menu">
            <UDropdownMenu
              :items="sourceMenuItems"
              :disabled="parse.orderedSources.length === 0"
              :content="{ align: 'end', sideOffset: 4, collisionPadding: 12 }"
              :ui="{ content: 'w-96 max-w-[calc(100vw-4rem)]', itemDescription: 'truncate' }"
            >
              <button
                class="source-menu-button"
                type="button"
                :disabled="parse.orderedSources.length === 0"
              >
                <span>{{ sourceMenuLabel }}</span>
                <small v-if="activeSource" :title="activeSourceTitle">{{ activeSourceTitle }}</small>
                <UIcon name="i-tabler-chevron-down" aria-hidden="true" />
              </button>
            </UDropdownMenu>
          </div>
          <UiStatusBadge v-if="createLoading" status="downloading">解析中</UiStatusBadge>
        </div>
      </div>
      <form class="parse-form" @submit.prevent="submitInput">
        <div class="parse-input-stack">
          <UiTextarea v-model="parse.input" label="链接、BV / AV 或多行列表" placeholder="粘贴视频、合集、收藏夹、番剧或课程链接" :rows="2" />
        </div>
        <div class="parse-actions">
          <UiButton type="submit" :disabled="createLoading">{{ createLoading ? '识别中' : '开始解析' }}</UiButton>
          <UiButton type="button" variant="secondary" :disabled="createLoading" @click="importTextFile">导入文本</UiButton>
        </div>
        <input ref="input-file" class="visually-hidden-file" type="file" accept=".txt,.list,.csv,text/plain" @change="handleTextFile" />
      </form>
      <UiInlineNotice
        v-if="parse.notice"
        :tone="parse.notice.tone"
        :action-label="parse.notice.actionLabel"
        @action="runNoticeAction"
      >
        {{ parse.notice.message }}
      </UiInlineNotice>
    </section>

    <section class="panel result-panel">
      <div v-if="activeSource" class="result-toolbar">
        <div class="result-current">
          <div class="result-title-line">
            <strong :title="activeSourceTitle">{{ activeSourceTitle }}</strong>
            <span class="result-kind">{{ activeSourceKindLabel }}</span>
          </div>
          <div class="result-counts" aria-label="内容选择统计">
            <span>共 <b>{{ totalPartCount }}</b> 项</span>
            <span>已选 <b>{{ selectedCount }}</b> 项</span>
            <span v-if="hasResultQuery">搜索找到 <b>{{ visiblePartCount }}</b> 项</span>
          </div>
        </div>
        <div class="result-actions">
          <UiButton variant="ghost" :disabled="!canSelectResults" @click="selectAllResults">{{ selectAllLabel }}</UiButton>
          <UiButton variant="ghost" :disabled="activeLoading || selectedCount === 0" @click="clearSelection">清空选择</UiButton>
          <UiIconButton icon="refresh" label="刷新内容" variant="ghost" size="compact" :disabled="activeLoading" @click="refreshSource" />
          <UiIconButton
            v-if="canCloseActiveSource"
            icon="x"
            label="关闭当前结果"
            variant="ghost"
            size="compact"
            @click="closeSource(activeSource.source.id)"
          />
          <UiButton class="result-primary-action" :disabled="!canCreateTasks" @click="openDownloadSettings">{{ createTaskLabel }}</UiButton>
        </div>
        <p class="result-hydration-note">
          <UIcon name="i-tabler-bolt" aria-hidden="true" />
          清晰度、编码与下载地址会在创建任务时，仅为所选内容获取。
        </p>
        <div class="result-tools">
          <UiTextField v-model="resultQuery" label="搜索内容" placeholder="标题 / UP 主 / BV" :disabled="activeLoading" />
          <UiSelect v-model="resultSort" label="排序" :options="resultSortOptions" :disabled="activeLoading" />
          <div class="range-control">
            <UiTextField
              v-model="rangeExpression"
              label="序号范围"
              placeholder="1-5,7,9-12"
              :disabled="activeLoading || visiblePartCount === 0"
            />
            <UiButton type="button" variant="secondary" :disabled="!canSelectRange" @click="selectRange">选中</UiButton>
          </div>
        </div>
        <p v-if="rangeError" class="range-error">{{ rangeError }}</p>
      </div>

      <UiTree v-if="activeSource && treeNodes.length" :nodes="treeNodes" :selected-ids="selectedIds" @toggle="toggleNode" />
      <UiEmptyState
        v-else-if="activeSource"
        title="没有匹配结果"
        description="换一个关键词，或清除搜索与范围条件。"
        icon="i-tabler-filter-off"
        layout="stacked"
        compact
      />
      <UiEmptyState
        v-else
        title="从一个来源开始"
        description="输入视频、合集、收藏夹、UP 主空间、番剧或课程链接。"
        icon="i-tabler-link"
        layout="stacked"
      >
        <template #detail>解析结果只保留在当前会话；确认选择后才会创建下载任务。</template>
      </UiEmptyState>
      <p v-if="activeError" class="inline-alert">{{ activeError }}</p>
    </section>

    <UiDialog v-model="downloadDialogOpen" title="下载设置">
      <section class="download-dialog-summary">
        <strong>{{ selectedCount }}</strong>
        <div>
          <span>个分集将加入传输</span>
          <p>{{ activeSourceTitle }}</p>
        </div>
      </section>

      <div class="download-settings-form">
        <div class="directory-row">
          <UiTextField
            :model-value="downloadDir"
            label="保存目录"
            placeholder="留空时使用 downloads"
            @update:model-value="updateDownloadDir"
          />
          <UiButton variant="secondary" :disabled="activeLoading" @click="chooseDownloadDir">选择</UiButton>
        </div>
        <section class="download-settings-section">
          <h3>下载内容</h3>
          <div class="download-settings-grid">
            <UiSelect v-model="mediaMode" label="下载内容" :options="mediaModeOptions" />
            <UiSelect
              v-if="includesVideo"
              v-model="videoQuality"
              label="视频清晰度"
              :options="videoQualityOptions"
            />
            <UiSelect v-if="includesAudio" v-model="audioQuality" label="音频质量" :options="audioQualityOptions" />
          </div>
        </section>
        <section class="download-settings-section">
          <h3>开始方式</h3>
          <div class="download-settings-grid">
            <UiTextField
              v-model="scheduledLocal"
              type="datetime-local"
              label="开始时间（可选）"
              :min="scheduleMin"
              :error="scheduleError"
              helper="留空时立即加入下载队列"
            />
            <UiTextField
              v-model="taskSpeedLimitMib"
              label="单任务限速（MiB/s）"
              placeholder="留空时不单独限速"
              :error="taskSpeedLimitError ?? undefined"
              helper="留空时仅受全局限速影响；多分段共同使用此额度"
            />
          </div>
        </section>
        <UiDisclosure title="更多选项" :description="downloadAdvancedSummary" variant="panel">
          <section class="download-settings-section">
            <div class="download-settings-grid">
              <UiSelect v-if="includesVideo" v-model="videoCodec" label="视频编码偏好" :options="codecOptions" />
              <UiSelect
                v-model="outputExtension"
                label="封装格式"
                :options="outputExtensionOptions"
              />
              <UiSelect
                v-model="archiveMode"
                label="附加内容"
                :options="archiveModeOptions"
              />
            </div>
          </section>
        </UiDisclosure>
        <p class="download-settings-note">{{ downloadSettingsSummary }}</p>
        <UiEnvironmentHealthPanel
          compact
          :health="settings.environmentHealth"
          :checking="settings.environmentChecking"
          @check="checkDownloadEnvironment"
          @create-directory="createDownloadDirectory"
          @choose-directory="chooseDownloadDir"
          @choose-ffmpeg="openEnvironmentSettings"
          @use-system-ffmpeg="openEnvironmentSettings"
        />
      </div>

      <template #footer>
        <UiButton variant="secondary" :disabled="activeLoading" @click="downloadDialogOpen = false">取消</UiButton>
        <UiButton
          :disabled="!canCreateTasks || Boolean(scheduleError) || Boolean(taskSpeedLimitError) || settings.environmentChecking || settings.environmentHealth?.ready === false"
          @click="createTasks()"
        >
          加入传输
        </UiButton>
      </template>
    </UiDialog>

    <UiDialog v-model="duplicateDialogOpen" title="发现重复任务">
      <section class="duplicate-summary">
        <span class="duplicate-summary-icon" aria-hidden="true">
          <UIcon name="i-tabler-copy" />
        </span>
        <div>
          <strong>{{ duplicateMatches.length }} 个分集已在传输记录中</strong>
          <p>可以跳过这些分集，或创建使用独立文件名的新任务。</p>
        </div>
      </section>

      <ul class="duplicate-list" aria-label="重复任务">
        <li v-for="match in duplicatePreview" :key="match.proposed_task_id">
          <span>{{ match.title }}</span>
          <UiStatusBadge :status="statusBadge(match.existing_status)">
            {{ statusLabel(match.existing_status) }}
          </UiStatusBadge>
        </li>
      </ul>
      <p v-if="duplicateRemaining > 0" class="duplicate-remaining">另有 {{ duplicateRemaining }} 项未展开</p>

      <template #footer>
        <UiButton variant="ghost" :disabled="activeLoading" @click="duplicateDialogOpen = false">取消</UiButton>
        <UiButton variant="secondary" :disabled="activeLoading" @click="resolveDuplicates('skip')">跳过重复项</UiButton>
        <UiButton :disabled="activeLoading" @click="resolveDuplicates('create')">仍然创建</UiButton>
      </template>
    </UiDialog>
  </section>
</template>

<style scoped src="./ParsePage.css"></style>
