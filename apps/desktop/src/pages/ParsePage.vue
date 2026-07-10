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
import UiIconButton from '../ui/IconButton.vue'
import UiInlineNotice from '../ui/InlineNotice.vue'
import UiSelect from '../ui/Select.vue'
import UiStatusBadge from '../ui/StatusBadge.vue'
import UiTextarea from '../ui/Textarea.vue'
import UiTextField from '../ui/TextField.vue'
import UiTree from '../ui/Tree.vue'
import { useParseStore } from '../stores/parse'
import { useSettingsStore } from '../stores/settings'
import { useUiStore } from '../stores/ui'
import { statusBadge, statusLabel } from '../stores/transferView'

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
  { label: '仅最终媒体', value: 'fast' },
  { label: '媒体 + 全部可用素材', value: 'complete_archive' },
  { label: '使用设置页自定义素材', value: 'custom' },
]
const resultSortOptions = [
  { label: '原始顺序', value: 'source' },
  { label: '标题 A-Z', value: 'title_asc' },
  { label: '时长优先', value: 'duration_desc' },
]

const activeSource = computed(() => parse.activeSource)
const selectedIds = computed(() => parse.activeSelection)
const selectedCount = computed(() => selectedIds.value.length)
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
const canLoadMore = computed(() => Boolean(activeSource.value?.source.has_more && !activeLoading.value))
const hasResultQuery = computed(() => resultQuery.value.trim().length > 0)
const canSelectVisible = computed(() => Boolean(activeSource.value && visiblePartCount.value > 0 && !activeLoading.value))
const canSelectRange = computed(() => Boolean(activeSource.value && rangeExpression.value.trim() && visiblePartCount.value > 0 && !activeLoading.value))
const createTaskLabel = computed(() => {
  if (activeLoading.value) {
    return '处理中'
  }
  return selectedCount.value > 0 ? `下载已选择 (${selectedCount.value})` : '先选择分集'
})
const loadedLabel = computed(() => {
  if (!activeSource.value) {
    return '--'
  }

  const total = activeSource.value.source.total_count ?? activeSource.value.source.loaded_count
  return `${activeSource.value.source.loaded_count} / ${total}`
})
const activeSourceKindLabel = computed(() => (activeSource.value ? sourceKindLabels[activeSource.value.source.kind] : '--'))
const activeSourceTitle = computed(() => activeSource.value?.source.title ?? '')
const canCloseActiveSource = computed(() => Boolean(activeSource.value && parse.sourceOrder.length > 1))
const sourceMenuLabel = computed(() => (parse.sourceOrder.length > 0 ? `已解析 ${parse.sourceOrder.length}` : '无记录'))
const visibleResultLabel = computed(() => (hasResultQuery.value ? `匹配 ${visiblePartCount.value}` : `可见 ${visiblePartCount.value}`))
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
      ? '视频 + 全部可用素材'
      : archiveMode.value === 'custom'
        ? '使用设置页自定义素材'
        : '仅最终媒体'
  return `${content} · ${video} · ${audio} · ${codec} · ${outputExtension.value.toUpperCase()} · ${dir} · ${archive}`
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
  downloadDialogOpen.value = true
}

const createTasks = async (duplicatePolicy: DuplicateTaskPolicy = 'ask') => {
  if (activeSource.value) {
    const result = await parse.createTasksForSelection(activeSource.value.source.id, {
      downloadDir: downloadDir.value,
      archiveMode: archiveMode.value,
      outputExtension: outputExtension.value,
      mediaMode: mediaMode.value,
      quality: videoQuality.value,
      audioQuality: audioQuality.value,
      codec: videoCodec.value,
      duplicatePolicy,
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
    }
  } catch (error) {
    parse.setNotice(errorMessage(error), 'warning')
  }
}

const loadMore = () => {
  if (activeSource.value) {
    void parse.loadMore(activeSource.value.source.id)
  }
}

const parseAll = () => {
  if (activeSource.value) {
    const source = activeSource.value.source
    const total = source.total_count ?? '未知'
    const confirmed = window.confirm(
      `解析全部会继续加载这个来源的远端列表，当前最多解析到 100 项。\n\n当前已加载 ${source.loaded_count} / ${total}。是否继续？`,
    )
    if (confirmed) {
      void parse.parseAll(source.id, 100)
    }
  }
}

const selectAllLoaded = () => {
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
    description: `${sourceLoadedLabel(tree)} · 已选 ${sourceSelectedCount(tree.source.id)}`,
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
    parse.setNotice(`已选中 ${partIds.length} 个可见分集`, 'success')
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
  durationSeconds: item.duration_seconds,
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
      throw new Error(`范围超出当前可见数量：${token}`)
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
): string => item.duration_seconds ? formatDuration(item.duration_seconds) : streamSummary(part.streams.length)

const sameTitle = (left: string, right: string): boolean => left.trim() !== '' && left.trim() === right.trim()

const formatDuration = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60)
  const rest = seconds % 60
  return `${minutes}:${rest.toString().padStart(2, '0')}`
}

const streamSummary = (count: number): string => (count > 0 ? `${count} 条流` : '未拉流')

const sourceLoadedLabel = (tree: NormalizedSourceTree): string =>
  `${tree.source.loaded_count} / ${tree.source.total_count ?? tree.source.loaded_count}`

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
    <section class="panel parse-input-panel command-panel">
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
          <UiStatusBadge :status="createLoading ? 'downloading' : 'ready'">
            {{ createLoading ? "解析中" : "就绪" }}
          </UiStatusBadge>
        </div>
      </div>
      <form class="parse-form" @submit.prevent="submitInput">
        <div class="parse-input-stack">
          <UiTextarea v-model="parse.input" label="链接、BV / AV 或多行列表" placeholder="BV1xx411c7mD&#10;https://www.bilibili.com/video/..." :rows="3" />
          <div class="input-capabilities" aria-label="支持的输入">
            <span>视频</span>
            <span>合集 / 收藏夹</span>
            <span>番剧 / 课程</span>
            <kbd>Ctrl L 聚焦</kbd>
          </div>
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
          <strong :title="activeSourceTitle">{{ activeSourceTitle }}</strong>
          <span>{{ activeSourceKindLabel }} · 已选 {{ selectedCount }} 个 · 已解析 {{ loadedLabel }} · {{ visibleResultLabel }}</span>
        </div>
        <div class="result-actions">
          <UiButton variant="secondary" :disabled="!canSelectVisible" @click="selectAllLoaded">全选可见</UiButton>
          <UiButton variant="secondary" :disabled="activeLoading || selectedCount === 0" @click="clearSelection">
            清空
          </UiButton>
          <UiButton variant="secondary" :disabled="!canLoadMore" @click="loadMore">解析更多</UiButton>
          <UiButton variant="secondary" :disabled="!canLoadMore" @click="parseAll">解析全部</UiButton>
          <UiButton :disabled="!canCreateTasks" @click="openDownloadSettings">{{ createTaskLabel }}</UiButton>
          <UiIconButton icon="refresh" label="刷新解析结果" variant="ghost" :disabled="activeLoading" @click="refreshSource" />
          <UiIconButton
            v-if="canCloseActiveSource"
            icon="x"
            label="关闭当前结果"
            variant="ghost"
            @click="closeSource(activeSource.source.id)"
          />
        </div>
        <div class="result-tools">
          <UiTextField v-model="resultQuery" label="搜索结果" placeholder="标题 / UP 主 / BV" :disabled="activeLoading" />
          <UiSelect v-model="resultSort" label="排序" :options="resultSortOptions" :disabled="activeLoading" />
          <div class="range-control">
            <UiTextField
              v-model="rangeExpression"
              label="范围"
              placeholder="1-5,7,9-12"
              :disabled="activeLoading || visiblePartCount === 0"
            />
            <UiButton type="button" variant="secondary" :disabled="!canSelectRange" @click="selectRange">选中</UiButton>
          </div>
        </div>
        <p v-if="rangeError" class="range-error">{{ rangeError }}</p>
      </div>

      <UiTree v-if="activeSource && treeNodes.length" :nodes="treeNodes" :selected-ids="selectedIds" @toggle="toggleNode" />
      <div v-else-if="activeSource" class="empty-state parse-empty-state compact-empty">
        <UIcon name="i-tabler-filter-off" aria-hidden="true" />
        <strong>没有匹配结果</strong>
        <p>换一个关键词，或清除搜索与范围条件。</p>
      </div>
      <div v-else class="empty-state parse-empty-state">
        <div class="empty-signal" aria-hidden="true">
          <span><UIcon name="i-tabler-link" /></span>
          <i></i>
          <span><UIcon name="i-tabler-list-tree" /></span>
          <i></i>
          <span><UIcon name="i-tabler-download" /></span>
        </div>
        <strong>从一个来源开始</strong>
        <p>输入视频、合集、收藏夹、UP 主空间、番剧或课程链接。</p>
        <small>解析结果只保留在当前会话；确认选择后才会创建下载任务。</small>
      </div>
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
          <UiTextField v-model="downloadDir" label="保存目录" placeholder="留空时使用 downloads" />
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
        <details class="download-settings-more">
          <summary>
            <div>
              <strong>更多选项</strong>
              <span>{{ downloadAdvancedSummary }}</span>
            </div>
          </summary>
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
                label="保存内容"
                :options="archiveModeOptions"
              />
            </div>
          </section>
        </details>
        <p class="download-settings-note">{{ downloadSettingsSummary }}</p>
      </div>

      <template #footer>
        <UiButton variant="secondary" :disabled="activeLoading" @click="downloadDialogOpen = false">取消</UiButton>
        <UiButton :disabled="!canCreateTasks" @click="createTasks()">加入传输</UiButton>
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

<style scoped>
.parse-page {
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: auto minmax(0, 1fr);
}

.parse-input-panel {
  grid-column: 1 / -1;
  z-index: 10;
  overflow: visible;
}

.command-panel {
  position: relative;
  background: var(--color-surface);
}

.command-panel::after {
  content: none;
}

.duplicate-summary {
  display: flex;
  align-items: flex-start;
  gap: var(--space-12);
}

.duplicate-summary p,
.duplicate-remaining {
  margin: 4px 0 0;
  color: var(--color-text-muted);
}

.duplicate-summary-icon {
  width: 36px;
  height: 36px;
  flex: 0 0 36px;
  display: grid;
  place-items: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
}

.duplicate-list {
  max-height: 240px;
  margin: 0;
  padding: 0;
  overflow: auto;
  list-style: none;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.duplicate-list li {
  min-height: 44px;
  padding: 8px 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
  border-bottom: 1px solid var(--color-border);
}

.duplicate-list li:last-child {
  border-bottom: 0;
}

.duplicate-list li > span:first-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.command-heading {
  min-width: 0;
  display: flex;
  align-items: flex-start;
  gap: var(--space-sm);
}

.command-index {
  margin-top: 1px;
  color: var(--color-accent-strong);
  font-family: var(--font-display);
  font-size: var(--font-11);
  font-weight: 760;
  letter-spacing: 0.08em;
}

.command-heading > div {
  min-width: 0;
  display: grid;
  gap: 3px;
}

.command-heading h2,
.command-heading p {
  margin: 0;
}

.command-heading p {
  max-width: 62ch;
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.5;
}

.parse-heading-tools {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-8);
}

.source-menu {
  position: relative;
  min-width: 0;
}

.source-menu-button {
  max-width: 280px;
  height: 28px;
  display: inline-grid;
  grid-template-columns: auto minmax(0, 1fr) 14px;
  align-items: center;
  gap: var(--space-8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-text);
  padding: 0 var(--space-10, 10px);
  font-size: var(--font-12);
  font-weight: 700;
}

.source-menu-button > svg {
  width: 14px;
  height: 14px;
  color: var(--color-muted);
}

.source-menu-button:disabled {
  color: var(--color-muted);
  background: var(--color-panel);
}

.source-menu-button small {
  min-width: 0;
  overflow: hidden;
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.parse-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 104px;
  align-items: start;
  gap: var(--space-12);
}

.parse-input-stack {
  min-width: 0;
  display: grid;
  gap: var(--space-xs);
}

.input-capabilities {
  min-width: 0;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
}

.input-capabilities span,
.input-capabilities kbd {
  min-height: 22px;
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-4);
  background: color-mix(in oklab, var(--color-surface) 72%, transparent);
  color: var(--color-muted);
  padding: 0 var(--space-xs);
  font-size: var(--font-11);
  font-weight: 620;
}

.input-capabilities kbd {
  margin-left: auto;
  border-color: transparent;
  background: transparent;
  font-family: var(--font-display);
}

.parse-actions {
  display: grid;
  gap: var(--space-8);
  padding-top: 24px;
}

.parse-actions :deep(.ui-button) {
  width: 100%;
}

.visually-hidden-file {
  position: fixed;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}

.result-panel {
  min-height: 0;
  overflow: hidden;
}

.result-toolbar {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-16);
  padding: var(--space-10, 10px) var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

.result-current {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.result-current strong,
.result-current span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-current strong {
  color: var(--color-text);
  font-size: var(--font-13);
}

.result-current span {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.result-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-4);
}

.result-toolbar :deep(.ui-button) {
  min-width: 64px;
  height: 28px;
  font-size: var(--font-12);
}

.result-actions :deep(.ui-icon-button) {
  width: 28px;
  height: 28px;
}

.result-tools {
  grid-column: 1 / -1;
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(180px, 1fr) minmax(150px, 180px) minmax(220px, 280px);
  align-items: end;
  gap: var(--space-10, 10px);
}

.range-control {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
}

.range-control :deep(.ui-button) {
  height: var(--height-input);
}

.range-error {
  grid-column: 1 / -1;
  margin: calc(var(--space-8) * -1) 0 0;
  color: var(--color-danger);
  font-size: var(--font-12);
  font-weight: 700;
}

.empty-state {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.parse-empty-state {
  min-height: 240px;
  align-content: center;
  justify-items: center;
  gap: var(--space-xs);
  padding: var(--space-xl);
  border-style: solid;
  background: var(--color-panel);
  text-align: center;
}

.parse-empty-state > strong {
  color: var(--color-text-strong);
  font-size: var(--font-16);
  font-weight: 760;
}

.parse-empty-state > p,
.parse-empty-state > small {
  max-width: 60ch;
  margin: 0;
  line-height: 1.55;
}

.parse-empty-state > p {
  color: var(--color-muted);
  font-size: var(--font-13);
}

.parse-empty-state > small {
  color: var(--color-dimmed);
  font-size: var(--font-11);
}

.parse-empty-state > svg {
  width: 24px;
  height: 24px;
  color: var(--color-muted);
}

.parse-empty-state.compact-empty {
  min-height: 160px;
}

.empty-signal {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  margin-bottom: var(--space-sm);
}

.empty-signal span {
  width: 38px;
  height: 38px;
  display: grid;
  place-items: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-accent-strong);
  box-shadow: var(--shadow-panel);
}

.empty-signal svg {
  width: 18px;
  height: 18px;
}

.empty-signal i {
  width: 30px;
  height: 1px;
  background: repeating-linear-gradient(90deg, var(--color-border-strong) 0 4px, transparent 4px 7px);
}

.empty-state {
  min-height: 120px;
  display: grid;
  place-items: center;
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

.inline-alert {
  margin: 0;
  padding: var(--space-8) var(--space-12);
  border: 1px solid rgb(201 42 42 / 20%);
  border-radius: var(--radius-6);
  background: #fffafa;
  color: var(--color-danger);
  font-size: var(--font-12);
  line-height: 1.5;
  overflow-wrap: anywhere;
}

.download-dialog-summary {
  min-width: 0;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: var(--space-12);
  padding: var(--space-12);
  border: 1px solid rgb(8 127 91 / 20%);
  border-radius: var(--radius-8);
  background: #eef8f3;
}

.download-dialog-summary > strong {
  color: var(--color-accent-strong);
  font-size: 34px;
  line-height: 1;
}

.download-dialog-summary div {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.download-dialog-summary span,
.download-dialog-summary p {
  min-width: 0;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.download-dialog-summary span {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.download-dialog-summary p {
  color: var(--color-text);
  font-size: var(--font-13);
  font-weight: 700;
}

.download-settings-form {
  display: grid;
  gap: var(--space-12);
}

.download-settings-section {
  display: grid;
  gap: var(--space-8);
}

.download-settings-more {
  min-width: 0;
  display: grid;
  gap: var(--space-10, 10px);
  padding: var(--space-10, 10px) var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-panel);
}

.download-settings-more summary {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
  cursor: pointer;
  list-style: none;
}

.download-settings-more summary::-webkit-details-marker {
  display: none;
}

.download-settings-more summary::after {
  content: "展开";
  flex: 0 0 auto;
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
}

.download-settings-more[open] summary::after {
  content: "收起";
}

.download-settings-more summary div {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.download-settings-more summary strong {
  color: var(--color-text);
  font-size: var(--font-13);
}

.download-settings-more summary span {
  min-width: 0;
  overflow: hidden;
  color: var(--color-muted);
  font-size: var(--font-12);
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.download-settings-section h3 {
  margin: 0;
  color: var(--color-text);
  font-size: var(--font-13);
  font-weight: 800;
}

.download-settings-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-12);
}

.download-settings-note {
  margin: 0;
  padding: var(--space-8) var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-panel);
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.5;
  overflow-wrap: anywhere;
}

@media (max-width: 840px) {
  .parse-form,
  .result-toolbar,
  .result-tools,
  .download-settings-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .range-control {
    grid-template-columns: minmax(0, 1fr) 88px;
  }

  .parse-actions,
  .result-actions {
    justify-content: flex-start;
  }
}
</style>
