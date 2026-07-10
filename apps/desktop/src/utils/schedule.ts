export const scheduledLocalError = (value: string, now: number, required = false): string => {
  if (!value) return required ? '请选择开始时间' : ''
  const scheduledAt = new Date(value)
  if (!Number.isFinite(scheduledAt.getTime())) return '请选择有效的开始时间'
  return scheduledAt.getTime() <= now ? '开始时间必须晚于当前时间' : ''
}

export const toDateTimeLocalValue = (
  date: Date,
  timezoneOffsetMinutes = date.getTimezoneOffset(),
): string => {
  const offset = timezoneOffsetMinutes * 60_000
  return new Date(date.getTime() - offset).toISOString().slice(0, 16)
}

export const toScheduledIso = (localValue: string): string => new Date(localValue).toISOString()
