import type { ParseBatchEntry } from './parse'

export const filterParseBatchEntries = (entries: ParseBatchEntry[], query: string): ParseBatchEntry[] => {
  const normalizedQuery = query.trim().toLowerCase()
  if (!normalizedQuery) return entries
  return entries.filter((entry) => `${entry.title} ${entry.input}`.toLowerCase().includes(normalizedQuery))
}

export const allBatchEntriesSelected = (entries: ParseBatchEntry[], selectedIds: string[]): boolean => {
  if (entries.length === 0) return false
  const selected = new Set(selectedIds)
  return entries.every((entry) => selected.has(entry.id))
}

export const toggleBatchEntrySelection = (
  entries: ParseBatchEntry[],
  selectedIds: string[],
  entryId: string,
): string[] => {
  const selected = new Set(selectedIds)
  if (selected.has(entryId)) {
    selected.delete(entryId)
  } else if (entries.some((entry) => entry.id === entryId)) {
    selected.add(entryId)
  }
  return [...selected]
}

export const selectedBatchSourceIds = (entries: ParseBatchEntry[], selectedIds: string[]): string[] => {
  const selected = new Set(selectedIds)
  return [...new Set(entries.filter((entry) => selected.has(entry.id)).map((entry) => entry.sourceId))]
}

export const buildBatchSelectionBySource = (
  sourceIds: string[],
  entries: ParseBatchEntry[],
  selectedIds: string[],
): Record<string, string[]> => {
  const selection = Object.fromEntries(sourceIds.map((sourceId) => [sourceId, [] as string[]]))
  const selected = new Set(selectedIds)
  for (const entry of entries) {
    if (!selected.has(entry.id)) continue
    const partIds = selection[entry.sourceId] ?? []
    if (!partIds.includes(entry.partId)) partIds.push(entry.partId)
    selection[entry.sourceId] = partIds
  }
  return selection
}
