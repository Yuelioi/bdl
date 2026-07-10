const MIB = 1024 * 1024
const MAX_MIB_PER_SECOND = 10 * 1024

export const speedLimitMibError = (value: string): string | null => {
  const trimmed = value.trim()
  if (!trimmed) {
    return null
  }

  const limit = Number(trimmed)
  if (!Number.isFinite(limit) || limit <= 0) {
    return '限速必须大于 0 MiB/s，留空表示不限速。'
  }
  if (limit * MIB < 1) {
    return '限速至少为 1 B/s（约 0.000001 MiB/s）。'
  }
  if (limit > MAX_MIB_PER_SECOND) {
    return `限速不能超过 ${MAX_MIB_PER_SECOND} MiB/s。`
  }
  return null
}

export const toBytesPerSecond = (value: string): number | undefined => {
  if (!value.trim()) {
    return undefined
  }
  if (speedLimitMibError(value)) {
    return undefined
  }
  return Math.round(Number(value) * MIB)
}

export const toMibPerSecondInput = (value: number | null | undefined): string => {
  if (!value || value <= 0) {
    return ''
  }
  return String(Number((value / MIB).toFixed(7)))
}

export const formatSpeedLimit = (value: number | null | undefined): string => {
  if (!value || value <= 0) {
    return '不限速'
  }
  if (value < 1024) {
    return `${Math.round(value)} B/s`
  }
  if (value < MIB) {
    return `${Number((value / 1024).toFixed(1))} KiB/s`
  }
  return `${Number((value / MIB).toFixed(1))} MiB/s`
}
