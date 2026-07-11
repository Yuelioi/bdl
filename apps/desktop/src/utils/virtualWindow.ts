export const LARGE_LIST_VIRTUALIZATION_THRESHOLD = 200
export const DEFAULT_VIRTUAL_OVERSCAN = 6

export interface VirtualWindow {
  start: number
  end: number
  offset: number
  totalSize: number
  virtualized: boolean
}

export const calculateVirtualWindow = (
  total: number,
  scrollOffset: number,
  viewportSize: number,
  itemSize: number,
  threshold = LARGE_LIST_VIRTUALIZATION_THRESHOLD,
  overscan = DEFAULT_VIRTUAL_OVERSCAN,
): VirtualWindow => {
  const safeTotal = Math.max(0, Math.floor(total))
  const safeItemSize = Math.max(1, itemSize)
  const totalSize = safeTotal * safeItemSize
  if (safeTotal < threshold) {
    return { start: 0, end: safeTotal, offset: 0, totalSize, virtualized: false }
  }

  const safeOffset = Math.max(0, Math.min(scrollOffset, Math.max(0, totalSize - viewportSize)))
  const visibleStart = Math.floor(safeOffset / safeItemSize)
  const visibleCount = Math.max(1, Math.ceil(Math.max(0, viewportSize) / safeItemSize))
  const start = Math.max(0, visibleStart - overscan)
  const end = Math.min(safeTotal, visibleStart + visibleCount + overscan)
  return { start, end, offset: start * safeItemSize, totalSize, virtualized: true }
}
