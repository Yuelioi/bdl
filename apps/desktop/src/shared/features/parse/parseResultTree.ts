import type { NormalizedItem, NormalizedSourceTree, SourceKind } from '../../api/dto'

export interface PageTreeNode {
  id: string
  label: string
  meta?: string
  partIds: string[]
  leafPartId?: string
  children?: PageTreeNode[]
}

export interface ParseResultRow {
  id: string
  title: string
  meta?: string
  partIds: string[]
}

export const sourceKindLabels: Record<SourceKind, string> = {
  video: '视频',
  bangumi: '番剧',
  cheese: '课程',
  watch_later: '稍后再看',
  favorite: '收藏夹',
  collection: '合集',
  series: '系列',
  uploader: 'UP 主',
  unknown: '未知',
}

export const selectsWholePosts = (kind: SourceKind) =>
  kind === 'uploader' || kind === 'watch_later' || kind === 'favorite' || kind === 'collection' || kind === 'series'

export const selectableParts = (item: NormalizedItem, kind: SourceKind) =>
  selectsWholePosts(kind) ? item.parts.slice(0, 1) : item.parts

export const toTreeNodes = (tree: NormalizedSourceTree): PageTreeNode[] =>
  tree.groups.flatMap((group) => {
    const itemNodes = group.items.map((item) => itemNode(item, tree.source.kind)).flat()

    if (tree.source.kind === 'video' && group.items.length === 1) return videoItemNodes(group.items[0])
    if (tree.groups.length === 1 || sameTitle(group.title, tree.source.title)) return itemNodes

    return [{
      id: group.id,
      label: group.title,
      meta: `${group.items.length} 项`,
      partIds: group.items.flatMap((item) => selectableParts(item, tree.source.kind).map((part) => part.id)),
      children: itemNodes,
    }]
  })

const itemNode = (
  item: NormalizedSourceTree['groups'][number]['items'][number],
  sourceKind: SourceKind,
): PageTreeNode[] => {
  if (!item.parts.length) return []
  if (item.parts.length === 1 || selectsWholePosts(sourceKind)) {
    const part = item.parts[0]
    return [{
      id: item.id,
      label: item.title || part.title,
      meta: item.owner_name ?? '',
      partIds: [part.id],
      leafPartId: part.id,
    }]
  }

  if (sourceKind === 'video') return videoItemNodes(item)

  return [{
    id: item.id,
    label: item.title,
    meta: `${item.parts.length} P`,
    partIds: item.parts.map((part) => part.id),
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
  meta: item.owner_name ?? '',
  partIds: [part.id],
  leafPartId: part.id,
})

export const flattenResultRows = (nodes: PageTreeNode[]): ParseResultRow[] =>
  nodes.flatMap((node) => node.children?.length
    ? flattenResultRows(node.children)
    : node.leafPartId
      ? [{ id: node.id, title: node.label, meta: node.meta, partIds: node.partIds }]
      : [])

export const sourcePartCount = (tree: NormalizedSourceTree): number =>
  tree.groups.reduce(
    (groupTotal, group) => groupTotal + group.items.reduce((itemTotal, item) => itemTotal + selectableParts(item, tree.source.kind).length, 0),
    0,
  )

const sameTitle = (left: string, right: string): boolean => left.trim() !== '' && left.trim() === right.trim()
