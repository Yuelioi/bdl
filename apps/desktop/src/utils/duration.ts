export const displayPartDuration = (
  partDurationSeconds: number | null,
  itemDurationSeconds: number | null,
  partCount: number,
): number | null => partDurationSeconds ?? (partCount === 1 ? itemDurationSeconds : null)

export const formatDuration = (seconds: number): string => {
  const wholeSeconds = Math.max(0, Math.floor(seconds))
  const hours = Math.floor(wholeSeconds / 3600)
  const minutes = Math.floor((wholeSeconds % 3600) / 60)
  const rest = wholeSeconds % 60

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${rest.toString().padStart(2, '0')}`
  }
  return `${minutes}:${rest.toString().padStart(2, '0')}`
}
