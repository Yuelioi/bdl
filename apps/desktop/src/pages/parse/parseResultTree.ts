import type { NormalizedSourceTree, SourceKind } from '../../api/dto'
import { displayPartDuration, formatDuration } from '../../utils/duration'

export interface PageTreeNode {
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

export interface VisiblePartEntry {
  id: string
  label: string
}

export type ResultSortMode = 'source' | 'title_asc' | 'duration_desc'

export const sourceKindLabels: Record<SourceKind, string> = {
  video: '视频',
  bangumi: '番剧',
  cheese: '课程',
  favorite: '收藏夹',
  collection: '合集',
  series: '系列',
  uploader: 'UP 主',
  unknown: '未知',
}

export const toTreeNodes = (tree: NormalizedSourceTree): PageTreeNode[] =>
  tree.groups.flatMap((group, groupIndex) => {
    const itemNodes = group.items.map((item, itemIndex) => itemNode(item, tree.source.kind, itemIndex)).flat()

    if (tree.source.kind === 'video' && group.items.length === 1) return videoItemNodes(group.items[0])
    if (tree.groups.length === 1 || sameTitle(group.title, tree.source.title)) return itemNodes

    return [{
      id: group.id,
      label: group.title,
      meta: `${group.items.length} 项`,
      partIds: group.items.flatMap((item) => item.parts.map((part) => part.id)),
      searchText: searchableText(group.title, `${group.items.length} 项`),
      sortTitle: group.title,
      durationSeconds: maxDuration(group.items.map((item) => item.duration_seconds)),
      sourceOrder: groupIndex,
      children: itemNodes,
    }]
  })

const itemNode = (
  item: NormalizedSourceTree['groups'][number]['items'][number],
  sourceKind: SourceKind,
  itemIndex: number,
): PageTreeNode[] => {
  if (item.parts.length === 1) {
    const part = item.parts[0]
    return [{
      id: item.id,
      label: item.title || part.title,
      meta: item.owner_name ?? partMeta(item, part),
      partIds: [part.id],
      searchText: searchableText(item.title, item.owner_name, part.title, part.bvid, part.cid),
      sortTitle: item.title || part.title,
      durationSeconds: item.duration_seconds,
      sourceOrder: itemIndex,
      leafPartId: part.id,
    }]
  }

  if (sourceKind === 'video') return videoItemNodes(item)

  return [{
    id: item.id,
    label: item.title,
    meta: `${item.parts.length} P`,
    partIds: item.parts.map((part) => part.id),
    searchText: searchableText(item.title, item.owner_name, `${item.parts.length} P`),
    sortTitle: item.title,
    durationSeconds: item.duration_seconds,
    sourceOrder: itemIndex,
    children: item.parts.map((part, index) => partNode(item, part, index)),
  }]
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

export const filterTreeNodes = (nodes: PageTreeNode[], query: string): PageTreeNode[] => {
  const normalized = normalizeSearch(query)
  if (!normalized) return nodes.map(cloneTreeNode)
  return nodes.map((node) => filterTreeNode(node, normalized)).filter((node): node is PageTreeNode => Boolean(node))
}

const filterTreeNode = (node: PageTreeNode, query: string): PageTreeNode | null => {
  const children = node.children
    ?.map((child) => filterTreeNode(child, query))
    .filter((child): child is PageTreeNode => Boolean(child))

  if (node.searchText.includes(query)) return cloneTreeNode(node)
  if (children?.length) return { ...node, partIds: children.flatMap((child) => child.partIds), children }
  return null
}

export const sortTreeNodes = (nodes: PageTreeNode[], mode: ResultSortMode): PageTreeNode[] => {
  const sorted = nodes.map((node) => ({
    ...node,
    children: node.children ? sortTreeNodes(node.children, mode) : undefined,
  }))
  return mode === 'source' ? sorted : sorted.sort((left, right) => compareTreeNodes(left, right, mode))
}

const compareTreeNodes = (left: PageTreeNode, right: PageTreeNode, mode: ResultSortMode): number => {
  if (mode === 'duration_desc') {
    const byDuration = (right.durationSeconds ?? -1) - (left.durationSeconds ?? -1)
    if (byDuration !== 0) return byDuration
  }
  const byTitle = left.sortTitle.localeCompare(right.sortTitle, 'zh-Hans-CN', { numeric: true, sensitivity: 'base' })
  return byTitle || left.sourceOrder - right.sourceOrder
}

export const numberVisibleParts = (nodes: PageTreeNode[]): PageTreeNode[] => {
  let index = 0
  const visit = (node: PageTreeNode): PageTreeNode => {
    const children = node.children?.map(visit)
    if (!children?.length && node.leafPartId) {
      index += 1
      return { ...node, label: `${String(index).padStart(2, '0')}  ${node.label}` }
    }
    return { ...node, children }
  }
  return nodes.map(visit)
}

export const flattenVisibleParts = (nodes: PageTreeNode[]): VisiblePartEntry[] =>
  nodes.flatMap((node) => node.children?.length
    ? flattenVisibleParts(node.children)
    : node.leafPartId ? [{ id: node.leafPartId, label: node.label }] : [])

export const parseRangeExpression = (value: string, total: number): number[] => {
  const expression = value.trim()
  if (!expression) throw new Error('请输入范围')

  const selected = new Set<number>()
  for (const token of expression.split(/[,，\s]+/).filter(Boolean)) {
    const match = token.match(/^(\d+)(?:-(\d+))?$/)
    if (!match) throw new Error(`范围格式无效：${token}`)
    const start = Number(match[1])
    const end = Number(match[2] ?? match[1])
    const min = Math.min(start, end)
    const max = Math.max(start, end)
    if (min < 1 || max > total) throw new Error(`范围超出当前结果数量：${token}`)
    for (let index = min; index <= max; index += 1) selected.add(index - 1)
  }
  return [...selected].sort((left, right) => left - right)
}

export const sourcePartCount = (tree: NormalizedSourceTree): number =>
  tree.groups.reduce(
    (groupTotal, group) => groupTotal + group.items.reduce((itemTotal, item) => itemTotal + item.parts.length, 0),
    0,
  )

const cloneTreeNode = (node: PageTreeNode): PageTreeNode => ({ ...node, children: node.children?.map(cloneTreeNode) })
const normalizeSearch = (value: string): string => value.trim().toLocaleLowerCase()
const searchableText = (...values: Array<string | number | null | undefined>): string => values
  .filter((value) => value !== null && value !== undefined && value !== '')
  .map((value) => normalizeSearch(String(value)))
  .join(' ')
const maxDuration = (values: Array<number | null>): number | null => {
  const durations = values.filter((value): value is number => typeof value === 'number')
  return durations.length ? Math.max(...durations) : null
}
const partMeta = (
  item: NormalizedSourceTree['groups'][number]['items'][number],
  part: NormalizedSourceTree['groups'][number]['items'][number]['parts'][number],
): string => {
  const duration = displayPartDuration(part.duration_seconds, item.duration_seconds, item.parts.length)
  return duration !== null ? formatDuration(duration) : ''
}
const sameTitle = (left: string, right: string): boolean => left.trim() !== '' && left.trim() === right.trim()
